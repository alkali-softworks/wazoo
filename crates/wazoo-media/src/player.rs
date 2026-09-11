use std::ffi::{c_int, c_void, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rand::Rng;

use crate::mpv_ffi;
use crate::pipeline::FrameData;

pub type PlayerId = usize;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub path: String,
    pub name: String,
    pub duration: Duration,
    pub position: Duration,
    pub volume: f64,
    pub is_muted: bool,
    pub is_playing: bool,
    pub last_checked_pos: Duration,
    pub stuck_count: usize,
}

impl PlayerState {
    pub fn new(id: PlayerId, path: String, name: String) -> Self {
        Self {
            id,
            path,
            name,
            duration: Duration::ZERO,
            position: Duration::ZERO,
            volume: 1.0,
            is_muted: false,
            is_playing: true,
            last_checked_pos: Duration::ZERO,
            stuck_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferConfig {
    pub duration_secs: u32,
    pub size_mb: u32,
    pub read_chunk_kb: u32,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            duration_secs: 10,
            size_mb: 32,
            read_chunk_kb: 512,
        }
    }
}

pub struct VideoHandle {
    pub id: PlayerId,
    pub state: PlayerState,
    mpv: *mut mpv_ffi::MpvHandle,
    render_ctx: *mut mpv_ffi::MpvRenderContext,
    render_width: u32,
    render_height: u32,
    pixel_buffer: Vec<u8>,
    current_frame: iced::widget::image::Handle,
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
    is_eos: bool,
    last_seek_time: Option<Instant>,
}

unsafe impl Send for VideoHandle {}
unsafe impl Sync for VideoHandle {}

impl Drop for VideoHandle {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        unsafe {
            if !self.render_ctx.is_null() {
                mpv_ffi::mpv_render_context_free(self.render_ctx);
                self.render_ctx = std::ptr::null_mut();
            }
            if !self.mpv.is_null() {
                mpv_ffi::mpv_terminate_destroy(self.mpv);
                self.mpv = std::ptr::null_mut();
            }
        }
    }
}

impl VideoHandle {
    pub const STUCK_THRESHOLD_SECONDS: usize = 30;
    pub const SEEK_GRACE_PERIOD: Duration = Duration::from_secs(10);

    pub fn new(id: PlayerId, file_path: &str, name: &str) -> Result<Self, String> {
        Self::with_buffering(id, file_path, name, BufferConfig::default())
    }

    pub fn with_buffering(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
    ) -> Result<Self, String> {
        unsafe {
            let mpv = mpv_ffi::mpv_create();
            if mpv.is_null() {
                return Err("Failed to create libmpv instance".to_string());
            }

            let set_opt = |k: &str, v: &str| {
                if let (Ok(ck), Ok(cv)) = (CString::new(k), CString::new(v)) {
                    mpv_ffi::mpv_set_option_string(mpv, ck.as_ptr(), cv.as_ptr());
                }
            };

            // Configure libmpv for optimal ambient media playback (coreaudio on macOS, wasapi on Windows, pulse/pipewire/alsa on Linux)
            set_opt("vo", "libmpv");
            set_opt("ao", "coreaudio,wasapi,pulse,pipewire,alsa,null");
            set_opt("hwdec", "auto-safe");
            set_opt("demuxer-max-bytes", &format!("{}M", config.size_mb.max(16)));
            set_opt("demuxer-readahead-secs", &format!("{}", config.duration_secs.max(2)));
            set_opt("osc", "no");
            set_opt("osd-level", "0");
            set_opt("osd-on-seek", "no");
            set_opt("osd-bar", "no");
            set_opt("sub-auto", "all");
            set_opt("sub-ass", "yes");
            set_opt("embeddedfonts", "yes");
            set_opt("keep-open", "yes");
            set_opt("idle", "yes");
            set_opt("terminal", "no");

            let res = mpv_ffi::mpv_initialize(mpv);
            if res < 0 {
                mpv_ffi::mpv_terminate_destroy(mpv);
                return Err(format!("Failed to initialize libmpv (code {res})"));
            }

            let api_type = CString::new("sw").unwrap();
            let mut params = [
                mpv_ffi::MpvRenderParam {
                    type_: mpv_ffi::MPV_RENDER_PARAM_API_TYPE,
                    data: api_type.as_ptr() as *mut c_void,
                },
                mpv_ffi::MpvRenderParam {
                    type_: mpv_ffi::MPV_RENDER_PARAM_INVALID,
                    data: std::ptr::null_mut(),
                },
            ];

            let mut render_ctx: *mut mpv_ffi::MpvRenderContext = std::ptr::null_mut();
            let res = mpv_ffi::mpv_render_context_create(&mut render_ctx, mpv, params.as_mut_ptr());
            if res < 0 || render_ctx.is_null() {
                mpv_ffi::mpv_terminate_destroy(mpv);
                return Err(format!("Failed to create mpv render context (code {res})"));
            }

            // Load media file (normalize Windows backslashes so mpv command string doesn't treat them as escape characters)
            let clean_path = if let Some(stripped) = file_path.strip_prefix("file://") {
                stripped
            } else {
                file_path
            };
            if !clean_path.starts_with("http://") && !clean_path.starts_with("https://") && !std::path::Path::new(clean_path).exists() {
                mpv_ffi::mpv_render_context_free(render_ctx);
                mpv_ffi::mpv_terminate_destroy(mpv);
                return Err(format!("Media file does not exist: {clean_path}"));
            }
            let normalized_path = clean_path.replace('\\', "/");
            let cmd = CString::new(format!("loadfile \"{}\"", normalized_path.replace('"', "\\\"")))
                .map_err(|e| e.to_string())?;
            mpv_ffi::mpv_command_string(mpv, cmd.as_ptr());

            // Non-blocking pump of initial events so the UI thread is not frozen
            while !mpv.is_null() {
                let event = mpv_ffi::mpv_wait_event(mpv, 0.0);
                if event.is_null() || (*event).event_id == mpv_ffi::MPV_EVENT_NONE {
                    break;
                }
            }

            let mut dur: f64 = 0.0;
            let prop = CString::new("duration").unwrap();
            let res = mpv_ffi::mpv_get_property(
                mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &mut dur as *mut _ as *mut _,
            );
            let initial_duration = if res == 0 && dur > 0.0 {
                Duration::from_secs_f64(dur)
            } else {
                Duration::ZERO
            };

            let render_width = 1280u32;
            let render_height = 720u32;
            let buffer_size = (render_width * render_height * 4) as usize;
            let mut pixel_buffer = vec![0u8; buffer_size];
            for chunk in pixel_buffer.chunks_exact_mut(4) {
                chunk[3] = 255;
            }
            let current_frame = iced::widget::image::Handle::from_rgba(
                render_width,
                render_height,
                pixel_buffer.clone(),
            );

            let alive = Arc::new(AtomicBool::new(true));
            let frame = Arc::new(Mutex::new(FrameData {
                width: render_width,
                height: render_height,
                pixels: pixel_buffer.clone(),
                new_frame: true,
            }));

            let mut state = PlayerState::new(id, file_path.to_string(), name.to_string());
            state.duration = initial_duration;

            let mut handle = Self {
                id,
                state,
                mpv,
                render_ctx,
                render_width,
                render_height,
                pixel_buffer,
                current_frame,
                frame,
                alive,
                is_eos: false,
                last_seek_time: None,
            };

            handle.set_volume(1.0);
            Ok(handle)
        }
    }

    /// Update video frame if mpv has decoded a new presentation frame
    pub fn update_frame(&mut self) -> bool {
        unsafe {
            // Process pending mpv events
            while !self.mpv.is_null() {
                let event = mpv_ffi::mpv_wait_event(self.mpv, 0.0);
                if event.is_null() || (*event).event_id == mpv_ffi::MPV_EVENT_NONE {
                    break;
                }
                if (*event).event_id == mpv_ffi::MPV_EVENT_END_FILE {
                    self.is_eos = true;
                }
            }

            if self.render_ctx.is_null() {
                return false;
            }

            let flags = mpv_ffi::mpv_render_context_update(self.render_ctx);
            if (flags & mpv_ffi::MPV_RENDER_UPDATE_FRAME) != 0 {
                let mut size = [self.render_width as i32, self.render_height as i32];
                let format = CString::new("rgb0").unwrap();
                let mut stride = (self.render_width * 4) as usize;
                let mut block_target_time: c_int = 0;
                let mut render_params = [
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_SW_SIZE,
                        data: size.as_mut_ptr() as *mut c_void,
                    },
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_SW_FORMAT,
                        data: format.as_ptr() as *mut c_void,
                    },
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_SW_STRIDE,
                        data: &mut stride as *mut usize as *mut c_void,
                    },
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_SW_POINTER,
                        data: self.pixel_buffer.as_mut_ptr() as *mut c_void,
                    },
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME,
                        data: &mut block_target_time as *mut c_int as *mut c_void,
                    },
                    mpv_ffi::MpvRenderParam {
                        type_: mpv_ffi::MPV_RENDER_PARAM_INVALID,
                        data: std::ptr::null_mut(),
                    },
                ];

                let err = mpv_ffi::mpv_render_context_render(self.render_ctx, render_params.as_mut_ptr());
                if err == 0 {
                    mpv_ffi::mpv_render_context_report_swap(self.render_ctx);

                    let num_pixels = (self.render_width * self.render_height) as usize;
                    let u32_slice: &mut [u32] = std::slice::from_raw_parts_mut(
                        self.pixel_buffer.as_mut_ptr() as *mut u32,
                        num_pixels,
                    );
                    for p in u32_slice.iter_mut() {
                        *p |= 0xFF000000;
                    }

                    {
                        let mut frame_guard = self.frame.lock().unwrap();
                        frame_guard.width = self.render_width;
                        frame_guard.height = self.render_height;
                        if frame_guard.pixels.len() != self.pixel_buffer.len() {
                            frame_guard.pixels.resize(self.pixel_buffer.len(), 0);
                        }
                        frame_guard.pixels.copy_from_slice(&self.pixel_buffer);
                        frame_guard.new_frame = true;
                    }

                    self.current_frame = iced::widget::image::Handle::from_rgba(
                        self.render_width,
                        self.render_height,
                        self.pixel_buffer.clone(),
                    );
                    self.state.position = self.position();
                    let d = self.duration();
                    if d > Duration::ZERO {
                        self.state.duration = d;
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Render video frame using a persistent GPU texture pipeline (flicker-free)
    pub fn view<'a, Message: 'a>(&'a self, opacity: f32) -> iced::Element<'a, Message> {
        let program = crate::pipeline::VideoProgram::new(
            self.id as u64,
            Arc::clone(&self.frame),
            Arc::clone(&self.alive),
            opacity,
        );
        iced::Element::new(crate::pipeline::video_shader(program))
    }

    /// Retrieve the current decoded video frame for rendering in Iced
    pub fn frame_handle(&self) -> iced::widget::image::Handle {
        self.current_frame.clone()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.render_width, self.render_height)
    }

    pub fn pixel_buffer(&self) -> &[u8] {
        &self.pixel_buffer
    }

    pub fn set_volume(&mut self, volume: f64) {
        let clamped = volume.clamp(0.0, 1.0);
        self.state.volume = clamped;
        let mpv_vol = clamped * 100.0;
        unsafe {
            let prop = CString::new("volume").unwrap();
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &mpv_vol as *const _ as *mut _,
            );
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.state.is_muted = muted;
        let flag: c_int = if muted { 1 } else { 0 };
        unsafe {
            let prop = CString::new("mute").unwrap();
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_FLAG,
                &flag as *const _ as *mut _,
            );
        }
    }

    pub fn toggle_play(&mut self) {
        if self.state.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn play(&mut self) {
        self.set_pause_internal(false);
    }

    pub fn pause(&mut self) {
        self.set_pause_internal(true);
    }

    fn set_pause_internal(&mut self, paused: bool) {
        self.state.is_playing = !paused;
        let flag: c_int = if paused { 1 } else { 0 };
        unsafe {
            let prop = CString::new("pause").unwrap();
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_FLAG,
                &flag as *const _ as *mut _,
            );
        }
    }

    pub fn set_subtitles_visible(&mut self, visible: bool) {
        let flag: c_int = if visible { 1 } else { 0 };
        unsafe {
            let prop = CString::new("sub-visibility").unwrap();
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_FLAG,
                &flag as *const _ as *mut _,
            );
        }
    }

    pub fn seek_random(&mut self) {
        let duration = self.duration();
        if duration > Duration::from_secs(2) {
            let max_secs = duration.as_secs_f64();
            let rand_secs = rand::thread_rng().gen_range(0.0..max_secs);
            self.seek(Duration::from_secs_f64(rand_secs));
        }
    }

    pub fn seek(&mut self, position: Duration) {
        self.seek_internal(position.as_secs_f64(), false, true);
    }

    pub fn seek_relative(&mut self, seconds: f64) {
        self.seek_internal(seconds, true, false);
    }

    fn seek_internal(&mut self, val: f64, relative: bool, accurate: bool) {
        self.last_seek_time = Some(Instant::now());
        self.is_eos = false;
        self.state.stuck_count = 0;

        let current = self.position().as_secs_f64();
        let dur = self.duration().as_secs_f64();
        let max_target = if dur > 0.5 { dur - 0.1 } else { f64::MAX };
        let target = if relative {
            (current + val).clamp(0.0, max_target)
        } else {
            val.clamp(0.0, max_target)
        };
        self.state.position = Duration::from_secs_f64(target);

        let mode = if relative {
            if accurate { "relative+exact" } else { "relative" }
        } else {
            if accurate { "absolute+exact" } else { "absolute" }
        };

        let cmd = format!("no-osd seek {} {}", val, mode);
        if let Ok(c_cmd) = CString::new(cmd) {
            unsafe {
                mpv_ffi::mpv_command_string(self.mpv, c_cmd.as_ptr());
            }
        }
    }

    pub fn adjust_volume(&mut self, delta: f64) {
        self.set_volume(self.state.volume + delta);
    }

    pub fn position(&self) -> Duration {
        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Duration::from_millis(500) {
                return self.state.position;
            }
        }
        unsafe {
            let mut pos: f64 = 0.0;
            let prop = CString::new("time-pos").unwrap();
            let res = mpv_ffi::mpv_get_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &mut pos as *mut _ as *mut _,
            );
            if res == 0 && pos >= 0.0 {
                Duration::from_secs_f64(pos)
            } else {
                self.state.position
            }
        }
    }

    pub fn duration(&self) -> Duration {
        unsafe {
            let mut dur: f64 = 0.0;
            let prop = CString::new("duration").unwrap();
            let res = mpv_ffi::mpv_get_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &mut dur as *mut _ as *mut _,
            );
            if res == 0 && dur > 0.0 {
                Duration::from_secs_f64(dur)
            } else {
                self.state.duration
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.is_eos
    }

    /// Check if video playback is stuck (same position for too long while marked playing)
    pub fn check_stuck(&mut self) -> bool {
        if !self.state.is_playing {
            self.state.stuck_count = 0;
            return false;
        }

        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Self::SEEK_GRACE_PERIOD {
                self.state.stuck_count = 0;
                self.state.last_checked_pos = self.position();
                return false;
            }
        }

        let current_pos = self.position();
        if current_pos == self.state.last_checked_pos && self.duration() > Duration::ZERO {
            self.state.stuck_count += 1;
            if self.state.stuck_count >= Self::STUCK_THRESHOLD_SECONDS {
                return true;
            }
        } else {
            self.state.stuck_count = 0;
            self.state.last_checked_pos = current_pos;
        }
        false
    }
}
