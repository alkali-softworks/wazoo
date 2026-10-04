/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Handle
 *
 * Encapsulates an individual video player instance powered by libmpv, providing playback
 * controls (play, pause, seek, volume, mute), texture rendering, and watchdog monitoring.
 */

use super::state::{BufferConfig, PlayerState};
use super::tracks::{
    AudioTrack, PlayerId, StartTime, SubtitleTrack, build_alang_string, build_slang_string,
    build_slang_string_with_fallback, find_matching_audio_track, get_subtitle_track_preference_string,
    get_track_preference_string, is_signs_or_songs_track,
    select_best_subtitle_track_with_fallback, subtitle_track_matches_preference,
    track_matches_preference,
};
use crate::mpv_ffi;
use crate::pipeline::FrameData;
use rand::Rng;
use std::ffi::{CString, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct VideoHandle {
    pub id: PlayerId,
    pub state: PlayerState,
    mpv: *mut mpv_ffi::MpvHandle,
    render_ctx: *mut mpv_ffi::MpvRenderContext,
    render_width: u32,
    render_height: u32,
    pixel_buffer: Vec<u8>,
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
    is_eos: bool,
    is_failed: bool,
    last_seek_time: Option<Instant>,
    pending_seek: Option<Duration>,
    pending_seek_random: bool,
    tracks_loaded: bool,
    preferred_audio_language: Option<String>,
    preferred_subtitle_language: Option<String>,
    i18n_language: Option<String>,
    gamma: f64,
    contrast: f64,
    brightness: f64,
    saturation: f64,
    speed: f64,
    pub crt_enabled: bool,
    pub wavy_enabled: bool,
    pub fog_enabled: bool,
}

unsafe impl Send for VideoHandle {}

impl std::fmt::Debug for VideoHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoHandle")
            .field("id", &self.id)
            .field("state", &self.state)
            .finish()
    }
}

impl Drop for VideoHandle {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        let mpv = self.mpv;
        let render_ctx = self.render_ctx;
        self.mpv = std::ptr::null_mut();
        self.render_ctx = std::ptr::null_mut();

        // Immediately silence audio and halt playback asynchronously without blocking the UI thread.
        // mpv_command_async never blocks the calling thread - it posts to libmpv's internal event queue
        // and returns in < 1 microsecond, avoiding any core lock contention or demuxer stalls.
        if !mpv.is_null() {
            unsafe {
                if let Ok(cmd_stop) = CString::new("stop") {
                    let mut stop_args = [cmd_stop.as_ptr(), std::ptr::null()];
                    mpv_ffi::mpv_command_async(mpv, 0, stop_args.as_mut_ptr());
                }
            }
        }

        let mpv_addr = mpv as usize;
        let render_ctx_addr = render_ctx as usize;
        if mpv_addr != 0 || render_ctx_addr != 0 {
            let _ = std::thread::Builder::new()
                .name("wazoo-mpv-teardown".into())
                .spawn(move || unsafe {
                    let mpv = mpv_addr as *mut mpv_ffi::MpvHandle;
                    let render_ctx = render_ctx_addr as *mut mpv_ffi::MpvRenderContext;
                    if !mpv.is_null() {
                        if let Ok(cmd_stop) = CString::new("stop") {
                            let mut args = [cmd_stop.as_ptr(), std::ptr::null()];
                            mpv_ffi::mpv_command(mpv, args.as_mut_ptr());
                        }
                    }
                    if !render_ctx.is_null() {
                        mpv_ffi::mpv_render_context_free(render_ctx);
                    }
                    if !mpv.is_null() {
                        mpv_ffi::mpv_terminate_destroy(mpv);
                    }
                });
        }
    }
}

impl VideoHandle {
    pub const STUCK_THRESHOLD_SECONDS: usize = 16;
    pub const SEEK_GRACE_PERIOD: Duration = Duration::from_secs(10);

    /// Construct FFmpeg `eq` video filter string for software rendering equalizer adjustments.
    /// Returns an empty string if all equalizer parameters are at their defaults (0.0).
    pub fn build_eq_filter_string(
        gamma: f64,
        contrast: f64,
        brightness: f64,
        saturation: f64,
    ) -> String {
        if gamma.abs() < 0.001
            && contrast.abs() < 0.001
            && brightness.abs() < 0.001
            && saturation.abs() < 0.001
        {
            return String::new();
        }

        // Map UI slider values (-100.0..=100.0) to FFmpeg eq filter parameters:
        // gamma: default 1.0, range 0.1..10.0 (reciprocal symmetry around 1.0)
        let eq_gamma = if gamma >= 0.0 {
            1.0 + (gamma / 100.0) * 1.5
        } else {
            1.0 / (1.0 + (-gamma / 100.0) * 1.5)
        };

        // contrast: default 1.0, range 0.0..2.5
        let eq_contrast = if contrast >= 0.0 {
            1.0 + (contrast / 100.0) * 1.5
        } else {
            (1.0 + (contrast / 100.0)).max(0.0)
        };

        // brightness: default 0.0, range -0.8..0.8
        let eq_brightness = (brightness / 100.0) * 0.8;

        // saturation: default 1.0, range 0.0..2.5
        let eq_saturation = if saturation >= 0.0 {
            1.0 + (saturation / 100.0) * 1.5
        } else {
            (1.0 + (saturation / 100.0)).max(0.0)
        };

        format!(
            "eq=gamma={:.3}:contrast={:.3}:brightness={:.3}:saturation={:.3}",
            eq_gamma, eq_contrast, eq_brightness, eq_saturation
        )
    }

    pub fn new(id: PlayerId, file_path: &str, name: &str) -> Result<Self, String> {
        Self::with_buffering(id, file_path, name, BufferConfig::default())
    }

    pub fn with_buffering(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
    ) -> Result<Self, String> {
        Self::with_buffering_and_start(id, file_path, name, config, None)
    }

    /// Creates an idle, pre-warmed VideoHandle with libmpv and render context initialized,
    /// ready to load a media file instantly via `load_file()` without blocking the UI thread.
    pub fn new_idle(config: BufferConfig) -> Result<Self, String> {
        Self::with_buffering_and_start(0, "", "", config, StartTime::Beginning)
    }

    pub fn with_buffering_and_start(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
        start_time: impl Into<StartTime>,
    ) -> Result<Self, String> {
        let start_time = start_time.into();
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
            // For software rendering context (MPV_RENDER_PARAM_API_TYPE = "sw"), direct hardware decoding
            // (e.g. vaapi, nvdec) is unsupported by libmpv and causes VA-API driver initialization errors
            // or thread deadlocks on X11 when windows are occluded or behind other windows.
            // Default to pure software decoding ("no") which is multi-threaded, robust, and zero-overhead.
            // Allow override via WAZOO_HWDEC environment variable (e.g. "auto-copy") for specialized setups.
            let hwdec = std::env::var("WAZOO_HWDEC").unwrap_or_else(|_| "no".to_string());
            set_opt("hwdec", &hwdec);
            set_opt("cache", "yes");
            let demuxer_mb = config.size_mb.clamp(8, 256);
            set_opt("demuxer-max-bytes", &format!("{}M", demuxer_mb));
            let readahead_secs = config.duration_secs.clamp(2, 5);
            set_opt("demuxer-readahead-secs", &format!("{}", readahead_secs));
            let back_mb = (demuxer_mb / 2).clamp(16, 64);
            set_opt("demuxer-max-back-bytes", &format!("{}M", back_mb));
            set_opt("cache-pause", "no");
            set_opt("hr-seek-framedrop", "yes");
            set_opt("force-seekable", "yes");
            set_opt("osc", "no");
            set_opt("osd-level", "0");
            set_opt("osd-on-seek", "no");
            set_opt("osd-bar", "no");
            set_opt("sub-auto", "fuzzy");
            set_opt("sub-ass", "yes");
            set_opt("embeddedfonts", "yes");
            set_opt("keep-open", "yes");
            set_opt("idle", "yes");
            set_opt("terminal", "no");

            let eq_filter = Self::build_eq_filter_string(
                config.gamma,
                config.contrast,
                config.brightness,
                config.saturation,
            );
            if !eq_filter.is_empty() {
                set_opt("vf", &eq_filter);
            }
            if config.gamma != 0.0 {
                set_opt("gamma", &format!("{}", config.gamma));
            }
            if config.contrast != 0.0 {
                set_opt("contrast", &format!("{}", config.contrast));
            }
            if config.brightness != 0.0 {
                set_opt("brightness", &format!("{}", config.brightness));
            }
            if config.saturation != 0.0 {
                set_opt("saturation", &format!("{}", config.saturation));
            }
            if (config.playback_speed - 1.0).abs() > 0.001 {
                set_opt("speed", &format!("{:.2}", config.playback_speed));
            }

            if let Some(ref pref) = config.preferred_audio_language {
                let alang = build_alang_string(pref);
                if !alang.is_empty() {
                    set_opt("alang", &alang);
                }
            }

            let slang = build_slang_string_with_fallback(
                config.preferred_subtitle_language.as_deref(),
                config.i18n_language.as_deref(),
            );
            if !slang.is_empty() {
                set_opt("slang", &slang);
            }

            match start_time {
                StartTime::Beginning => {}
                StartTime::Seconds(start) => {
                    if start > 0.05 {
                        set_opt("start", &format!("{:.3}", start));
                    }
                }
                StartTime::Percent(pct) => {
                    let clamped = pct.clamp(0.0, 95.0);
                    set_opt("start", &format!("{:.1}%", clamped));
                }
                StartTime::Random => {
                    let rand_pct = rand::thread_rng().gen_range(5.0..85.0);
                    set_opt("start", &format!("{:.1}%", rand_pct));
                }
            }

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

            if !file_path.is_empty() {
                // Load media file (normalize Windows backslashes so mpv command string doesn't treat them as escape characters)
                let clean_path = if let Some(stripped) = file_path.strip_prefix("file://") {
                    stripped
                } else {
                    file_path
                };
                if !clean_path.starts_with("http://")
                    && !clean_path.starts_with("https://")
                    && !std::path::Path::new(clean_path).exists()
                {
                    mpv_ffi::mpv_render_context_free(render_ctx);
                    mpv_ffi::mpv_terminate_destroy(mpv);
                    return Err(format!("Media file does not exist: {clean_path}"));
                }
                let cmd_loadfile = CString::new("loadfile").map_err(|e| e.to_string())?;
                let path_arg = CString::new(clean_path).map_err(|e| e.to_string())?;
                let mut args: [*const std::ffi::c_char; 3] =
                    [cmd_loadfile.as_ptr(), path_arg.as_ptr(), std::ptr::null()];
                mpv_ffi::mpv_command(mpv, args.as_mut_ptr());
            }

            let initial_duration = Duration::ZERO;
            let render_width = 0u32;
            let render_height = 0u32;
            let pixel_buffer = Vec::new();
            let alive = Arc::new(AtomicBool::new(true));
            let frame = Arc::new(Mutex::new(FrameData {
                width: 0,
                height: 0,
                pixels: Vec::new(),
                new_frame: false,
                frame_seq: 0,
            }));

            let mut state = PlayerState::new(id, file_path.to_string(), name.to_string());
            state.duration = initial_duration;

            let (pending_seek, last_seek_time) = match start_time {
                StartTime::Beginning => (None, None),
                StartTime::Seconds(start) => {
                    if start > 0.05 {
                        state.position = Duration::from_secs_f64(start);
                        (Some(Duration::from_secs_f64(start)), Some(Instant::now()))
                    } else {
                        (None, None)
                    }
                }
                StartTime::Percent(_) | StartTime::Random => (None, Some(Instant::now())),
            };

            let mut handle = Self {
                id,
                state,
                mpv,
                render_ctx,
                render_width,
                render_height,
                pixel_buffer,
                frame,
                alive,
                is_eos: false,
                is_failed: false,
                last_seek_time,
                pending_seek,
                pending_seek_random: false,
                tracks_loaded: false,
                preferred_audio_language: config.preferred_audio_language,
                preferred_subtitle_language: config.preferred_subtitle_language,
                i18n_language: config.i18n_language,
                gamma: config.gamma,
                contrast: config.contrast,
                brightness: config.brightness,
                saturation: config.saturation,
                speed: config.playback_speed,
                crt_enabled: config.crt_enabled,
                wavy_enabled: config.wavy_enabled,
                fog_enabled: config.fog_enabled,
            };

            handle.set_volume(1.0);
            handle.refresh_audio_tracks();
            Ok(handle)
        }
    }

    /// Seamlessly load a new media file into the existing libmpv player instance,
    /// avoiding complete teardown and re-creation of libmpv, render contexts, and audio sinks.
    pub fn load_file(
        &mut self,
        file_path: &str,
        name: &str,
        start_time: impl Into<StartTime>,
    ) -> Result<(), String> {
        let start_time = start_time.into();
        unsafe {
            if self.mpv.is_null() {
                return Err("mpv handle is null".to_string());
            }

            let clean_path = if let Some(stripped) = file_path.strip_prefix("file://") {
                stripped
            } else {
                file_path
            };

            if !clean_path.starts_with("http://")
                && !clean_path.starts_with("https://")
                && !std::path::Path::new(clean_path).exists()
            {
                return Err(format!("Media file does not exist: {clean_path}"));
            }

            match start_time {
                StartTime::Beginning => {
                    self.set_property_string("start", "0");
                }
                StartTime::Seconds(start) => {
                    if start > 0.05 {
                        self.set_property_string("start", &format!("{:.3}", start));
                    } else {
                        self.set_property_string("start", "0");
                    }
                }
                StartTime::Percent(pct) => {
                    let clamped = pct.clamp(0.0, 95.0);
                    self.set_property_string("start", &format!("{:.1}%", clamped));
                }
                StartTime::Random => {
                    let rand_pct = rand::thread_rng().gen_range(5.0..85.0);
                    self.set_property_string("start", &format!("{:.1}%", rand_pct));
                }
            }


            // Immediately clear frame to solid black so no frozen frame of prior video is visible
            if let Ok(mut frame) = self.frame.lock() {
                for chunk in frame.pixels.chunks_exact_mut(4) {
                    chunk[0] = 0;
                    chunk[1] = 0;
                    chunk[2] = 0;
                    chunk[3] = 255;
                }
                frame.new_frame = true;
            }
            for chunk in self.pixel_buffer.chunks_exact_mut(4) {
                chunk[0] = 0;
                chunk[1] = 0;
                chunk[2] = 0;
                chunk[3] = 255;
            }

            let cmd_loadfile = CString::new("loadfile").map_err(|e| e.to_string())?;
            let path_arg = CString::new(clean_path).map_err(|e| e.to_string())?;
            let replace_arg = CString::new("replace").map_err(|e| e.to_string())?;
            let mut args: [*const std::ffi::c_char; 4] = [
                cmd_loadfile.as_ptr(),
                path_arg.as_ptr(),
                replace_arg.as_ptr(),
                std::ptr::null(),
            ];
            let res = mpv_ffi::mpv_command(self.mpv, args.as_mut_ptr());
            if res < 0 {
                return Err(format!("Failed to load file: {res}"));
            }

            self.set_property_string("pause", "no");
            self.state.path = file_path.to_string();
            self.state.name = name.to_string();
            self.state.duration = Duration::ZERO;
            self.state.position = Duration::ZERO;
            self.state.is_playing = true;
            self.is_eos = false;
            self.is_failed = false;
            self.tracks_loaded = false;
            self.state.audio_tracks.clear();
            self.state.subtitle_tracks.clear();
            self.state.current_audio_track_id = None;
            self.state.current_subtitle_track_id = None;

            let (pending_seek, last_seek_time) = match start_time {
                StartTime::Beginning => (None, Some(Instant::now())),
                StartTime::Seconds(start) => {
                    if start > 0.05 {
                        self.state.position = Duration::from_secs_f64(start);
                        (Some(Duration::from_secs_f64(start)), Some(Instant::now()))
                    } else {
                        (None, Some(Instant::now()))
                    }
                }
                StartTime::Percent(_) | StartTime::Random => (None, Some(Instant::now())),
            };
            self.pending_seek = pending_seek;
            self.last_seek_time = last_seek_time;
            self.pending_seek_random = matches!(start_time, StartTime::Random);

            Ok(())
        }
    }

    /// Update video frame if mpv has decoded a new presentation frame
    pub fn update_frame(&mut self) -> bool {
        unsafe {
            // Process pending mpv events with a safety limit to prevent UI thread lockups
            let mut event_count = 0;
            let mut needs_refresh_tracks = false;
            while !self.mpv.is_null() && event_count < 64 {
                event_count += 1;
                let event = mpv_ffi::mpv_wait_event(self.mpv, 0.0);
                if event.is_null() || (*event).event_id == mpv_ffi::MPV_EVENT_NONE {
                    break;
                }
                if (*event).event_id == mpv_ffi::MPV_EVENT_START_FILE {
                    self.is_eos = false;
                    self.is_failed = false;
                }
                if (*event).event_id == mpv_ffi::MPV_EVENT_END_FILE {
                    if !(*event).data.is_null() {
                        let end_data = &*((*event).data as *const mpv_ffi::MpvEventEndFile);
                        if end_data.reason == mpv_ffi::MPV_END_FILE_REASON_ERROR
                            || end_data.error != 0
                        {
                            self.is_failed = true;
                            self.is_eos = true;
                        } else if end_data.reason == mpv_ffi::MPV_END_FILE_REASON_EOF {
                            self.is_eos = true;
                            if !self.tracks_loaded && self.duration() == Duration::ZERO {
                                self.is_failed = true;
                            }
                        }
                    } else {
                        self.is_eos = true;
                    }
                }
                if (*event).event_id == mpv_ffi::MPV_EVENT_FILE_LOADED {
                    self.is_eos = false;
                    self.is_failed = false;
                    needs_refresh_tracks = true;
                } else if (*event).event_id == mpv_ffi::MPV_EVENT_TRACKS_CHANGED
                    || (*event).event_id == mpv_ffi::MPV_EVENT_PLAYBACK_RESTART
                    || (*event).event_id == mpv_ffi::MPV_EVENT_VIDEO_RECONFIG
                {
                    self.is_eos = false;
                    self.is_failed = false;
                    needs_refresh_tracks = true;
                }
            }

            if needs_refresh_tracks {
                self.refresh_audio_tracks();
            } else if !self.tracks_loaded {
                let count = self.get_property_i64("track-list/count").unwrap_or(0);
                if count > 0 {
                    self.refresh_audio_tracks();
                }
            }

            if let Some(target) = self.pending_seek {
                if self.duration() > Duration::ZERO {
                    self.pending_seek = None;
                    self.last_seek_time = Some(Instant::now());
                    let cmd = format!("no-osd seek {:.3} absolute+exact", target.as_secs_f64());
                    if let Ok(c_cmd) = CString::new(cmd) {
                        mpv_ffi::mpv_command_string(self.mpv, c_cmd.as_ptr());
                    }
                }
            }

            if self.pending_seek_random {
                let dur = self.duration();
                if dur > Duration::from_secs(2) {
                    self.pending_seek_random = false;
                    self.last_seek_time = Some(Instant::now());
                    let max_secs = dur.as_secs_f64();
                    let min_secs = (max_secs * 0.05).min(5.0);
                    let max_bound = (max_secs * 0.85).max(min_secs);
                    let rand_secs = if max_bound > min_secs {
                        rand::thread_rng().gen_range(min_secs..max_bound)
                    } else {
                        0.0
                    };
                    self.seek_fast(Duration::from_secs_f64(rand_secs));
                }
            }

            self.check_and_apply_loop();

            if self.render_ctx.is_null() {
                return false;
            }

            self.check_update_render_dimensions();

            let flags = mpv_ffi::mpv_render_context_update(self.render_ctx);
            if (flags & mpv_ffi::MPV_RENDER_UPDATE_FRAME) != 0 {
                if self.render_width == 0 || self.render_height == 0 {
                    return false;
                }

                let mut size = [self.render_width as i32, self.render_height as i32];
                let format = c"rgb0";
                let mut stride = (self.render_width * 4) as usize;
                let mut block_target_time: c_int = 0;
                let buf_size = (self.render_width * self.render_height * 4) as usize;
                if self.pixel_buffer.len() != buf_size {
                    self.pixel_buffer.resize(buf_size, 0);
                }
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

                let err =
                    mpv_ffi::mpv_render_context_render(self.render_ctx, render_params.as_mut_ptr());
                if err == 0 {
                    mpv_ffi::mpv_render_context_report_swap(self.render_ctx);

                    {
                        let mut frame_guard = self.frame.lock().unwrap();
                        frame_guard.width = self.render_width;
                        frame_guard.height = self.render_height;
                        std::mem::swap(&mut frame_guard.pixels, &mut self.pixel_buffer);
                        frame_guard.new_frame = true;
                        frame_guard.frame_seq = frame_guard.frame_seq.wrapping_add(1);
                    }

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
        self.view_with_fit(opacity, false)
    }

    /// Render video frame with custom fit mode (fit_cover = true for edge-to-edge ambient fill)
    pub fn view_with_fit<'a, Message: 'a>(
        &'a self,
        opacity: f32,
        fit_cover: bool,
    ) -> iced::Element<'a, Message> {
        self.view_full(
            opacity,
            fit_cover,
            self.crt_enabled,
            self.wavy_enabled,
            self.fog_enabled,
        )
    }

    /// Render video frame with custom fit and explicit GPU shader filter settings
    pub fn view_full<'a, Message: 'a>(
        &'a self,
        opacity: f32,
        fit_cover: bool,
        crt_enabled: bool,
        wavy_enabled: bool,
        fog_enabled: bool,
    ) -> iced::Element<'a, Message> {
        let program = crate::pipeline::VideoProgram::new_full(
            self.id as u64,
            Arc::clone(&self.frame),
            Arc::clone(&self.alive),
            opacity,
            fit_cover,
            crt_enabled,
            wavy_enabled,
            fog_enabled,
        );
        iced::Element::new(crate::pipeline::video_shader(program))
    }

    pub fn set_crt_enabled(&mut self, enabled: bool) {
        self.crt_enabled = enabled;
    }

    pub fn crt_enabled(&self) -> bool {
        self.crt_enabled
    }

    pub fn set_wavy_enabled(&mut self, enabled: bool) {
        self.wavy_enabled = enabled;
    }

    pub fn wavy_enabled(&self) -> bool {
        self.wavy_enabled
    }

    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_enabled = enabled;
    }

    pub fn fog_enabled(&self) -> bool {
        self.fog_enabled
    }

    pub fn set_filters(&mut self, crt: bool, wavy: bool, fog: bool) {
        self.crt_enabled = crt;
        self.wavy_enabled = wavy;
        self.fog_enabled = fog;
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

    pub fn stop(&mut self) {
        self.state.is_playing = false;
        unsafe {
            if !self.mpv.is_null() {
                if let Ok(cmd) = CString::new("stop") {
                    let mut args = [cmd.as_ptr(), std::ptr::null()];
                    mpv_ffi::mpv_command_async(self.mpv, 0, args.as_mut_ptr());
                }
            }
        }
    }

    /// Overwrite pixel and frame buffers with solid black so no previous frame is visible
    pub fn clear_frame_black(&mut self) {
        if let Ok(mut frame) = self.frame.lock() {
            frame.pixels.fill(0);
            frame.new_frame = true;
        }
        self.pixel_buffer.fill(0);
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.set_pause_internal(paused);
    }

    pub fn is_playing(&self) -> bool {
        self.state.is_playing
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

    pub fn update_equalizer(&mut self) {
        let filter = Self::build_eq_filter_string(
            self.gamma,
            self.contrast,
            self.brightness,
            self.saturation,
        );
        self.set_property_string("vf", &filter);

        // Also update standard VO properties for compatibility if hardware VO is used
        let clamped_g = self.gamma.clamp(-100.0, 100.0);
        let clamped_c = self.contrast.clamp(-100.0, 100.0);
        let clamped_b = self.brightness.clamp(-100.0, 100.0);
        let clamped_s = self.saturation.clamp(-100.0, 100.0);
        unsafe {
            if !self.mpv.is_null() {
                if let Ok(p) = CString::new("gamma") {
                    mpv_ffi::mpv_set_property(
                        self.mpv,
                        p.as_ptr(),
                        mpv_ffi::MPV_FORMAT_DOUBLE,
                        &clamped_g as *const _ as *mut _,
                    );
                }
                if let Ok(p) = CString::new("contrast") {
                    mpv_ffi::mpv_set_property(
                        self.mpv,
                        p.as_ptr(),
                        mpv_ffi::MPV_FORMAT_DOUBLE,
                        &clamped_c as *const _ as *mut _,
                    );
                }
                if let Ok(p) = CString::new("brightness") {
                    mpv_ffi::mpv_set_property(
                        self.mpv,
                        p.as_ptr(),
                        mpv_ffi::MPV_FORMAT_DOUBLE,
                        &clamped_b as *const _ as *mut _,
                    );
                }
                if let Ok(p) = CString::new("saturation") {
                    mpv_ffi::mpv_set_property(
                        self.mpv,
                        p.as_ptr(),
                        mpv_ffi::MPV_FORMAT_DOUBLE,
                        &clamped_s as *const _ as *mut _,
                    );
                }
            }
        }

        // If the video is paused, trigger a zero-seek so mpv immediately re-decodes the current paused frame with new filter settings
        if !self.state.is_playing {
            unsafe {
                if !self.mpv.is_null() {
                    let cmd = c"no-osd seek 0 relative exact";
                    mpv_ffi::mpv_command_string(self.mpv, cmd.as_ptr());
                }
            }
        }
    }

    pub fn set_equalizer(&mut self, gamma: f64, contrast: f64, brightness: f64, saturation: f64) {
        let g = gamma.clamp(-100.0, 100.0);
        let c = contrast.clamp(-100.0, 100.0);
        let b = brightness.clamp(-100.0, 100.0);
        let s = saturation.clamp(-100.0, 100.0);
        if (self.gamma - g).abs() < 0.001
            && (self.contrast - c).abs() < 0.001
            && (self.brightness - b).abs() < 0.001
            && (self.saturation - s).abs() < 0.001
        {
            return;
        }
        self.gamma = g;
        self.contrast = c;
        self.brightness = b;
        self.saturation = s;
        self.update_equalizer();
    }

    pub fn set_gamma(&mut self, gamma: f64) {
        let clamped = gamma.clamp(-100.0, 100.0);
        if (self.gamma - clamped).abs() < 0.001 {
            return;
        }
        self.gamma = clamped;
        self.update_equalizer();
    }

    pub fn set_contrast(&mut self, contrast: f64) {
        let clamped = contrast.clamp(-100.0, 100.0);
        if (self.contrast - clamped).abs() < 0.001 {
            return;
        }
        self.contrast = clamped;
        self.update_equalizer();
    }

    pub fn set_brightness(&mut self, brightness: f64) {
        let clamped = brightness.clamp(-100.0, 100.0);
        if (self.brightness - clamped).abs() < 0.001 {
            return;
        }
        self.brightness = clamped;
        self.update_equalizer();
    }

    pub fn set_saturation(&mut self, saturation: f64) {
        let clamped = saturation.clamp(-100.0, 100.0);
        if (self.saturation - clamped).abs() < 0.001 {
            return;
        }
        self.saturation = clamped;
        self.update_equalizer();
    }

    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    pub fn contrast(&self) -> f64 {
        self.contrast
    }

    pub fn brightness(&self) -> f64 {
        self.brightness
    }

    pub fn saturation(&self) -> f64 {
        self.saturation
    }

    pub fn set_speed(&mut self, speed: f64) {
        let clamped = speed.clamp(0.25, 4.0);
        if (self.speed - clamped).abs() < 0.001 {
            return;
        }
        self.speed = clamped;
        unsafe {
            let prop = CString::new("speed").unwrap();
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &clamped as *const _ as *mut _,
            );
        }
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn has_decoded_frame(&self) -> bool {
        if let Ok(guard) = self.frame.lock() {
            !guard.pixels.is_empty() && guard.width > 0 && guard.height > 0
        } else {
            false
        }
    }

    pub fn frame_snapshot(&self) -> Option<Vec<u8>> {
        self.frame.lock().ok().map(|g| g.pixels.clone())
    }

    pub fn frame(&self) -> Arc<Mutex<FrameData>> {
        Arc::clone(&self.frame)
    }

    pub fn alive(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.alive)
    }

    pub fn is_pending_seek_random(&self) -> bool {
        self.pending_seek_random
    }

    pub fn seek_fast(&mut self, position: Duration) {
        self.seek_internal(position.as_secs_f64(), false, false);
    }

    pub fn seek_random(&mut self) {
        let duration = self.duration();
        if duration > Duration::from_secs(2) {
            let max_secs = duration.as_secs_f64();
            let rand_secs = rand::thread_rng().gen_range(0.0..max_secs);
            self.pending_seek_random = false;
            self.seek_fast(Duration::from_secs_f64(rand_secs));
        } else {
            self.pending_seek_random = true;
        }
    }

    pub fn seek(&mut self, position: Duration) {
        self.seek_internal(position.as_secs_f64(), false, true);
    }

    pub fn seek_relative(&mut self, seconds: f64) {
        self.seek_internal(seconds, true, false);
    }

    /// Pause playback and step forward by exactly one video frame
    pub fn step_frame_forward(&mut self) {
        if self.mpv.is_null() {
            return;
        }
        self.pause();
        self.pending_seek = None;
        self.last_seek_time = None;
        if let Ok(cmd) = CString::new("frame-step") {
            unsafe {
                mpv_ffi::mpv_command_string(self.mpv, cmd.as_ptr());
            }
        }
        self.update_frame();
    }

    /// Pause playback and step backward by exactly one video frame
    pub fn step_frame_backward(&mut self) {
        if self.mpv.is_null() {
            return;
        }
        self.pause();
        self.pending_seek = None;
        self.last_seek_time = None;
        if let Ok(cmd) = CString::new("frame-back-step") {
            unsafe {
                mpv_ffi::mpv_command_string(self.mpv, cmd.as_ptr());
            }
        }
        self.update_frame();
    }

    fn seek_internal(&mut self, val: f64, relative: bool, accurate: bool) {
        self.pending_seek_random = false;
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

        if dur == 0.0 {
            self.pending_seek = Some(self.state.position);
        } else {
            self.pending_seek = None;
        }

        let mode = if relative {
            if accurate {
                "relative+exact"
            } else {
                "relative"
            }
        } else {
            if accurate {
                "absolute+exact"
            } else {
                "absolute"
            }
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
        if let Some(target) = self.pending_seek {
            return target;
        }
        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Duration::from_millis(1500) {
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

    pub fn mark_in(&self) -> Option<Duration> {
        self.state.mark_in
    }

    pub fn mark_out(&self) -> Option<Duration> {
        self.state.mark_out
    }

    pub fn has_loop(&self) -> bool {
        self.state.mark_in.is_some() || self.state.mark_out.is_some()
    }

    pub fn set_mark_in(&mut self, pos: Option<Duration>) {
        self.state.mark_in = pos;
        if let (Some(in_pos), Some(out_pos)) = (self.state.mark_in, self.state.mark_out) {
            if in_pos >= out_pos {
                self.state.mark_out = None;
            }
        }
    }

    pub fn set_mark_out(&mut self, pos: Option<Duration>) {
        self.state.mark_out = pos;
        if let (Some(in_pos), Some(out_pos)) = (self.state.mark_in, self.state.mark_out) {
            if out_pos <= in_pos {
                self.state.mark_in = None;
            }
        }
    }

    pub fn clear_mark_in(&mut self) {
        self.state.mark_in = None;
    }

    pub fn clear_mark_out(&mut self) {
        self.state.mark_out = None;
    }

    pub fn clear_marks(&mut self) {
        self.state.mark_in = None;
        self.state.mark_out = None;
    }

    /// Check if the player has an active Mark-In / Mark-Out loop, and when the current playback
    /// position reaches or passes the loop boundary or end-of-stream, seek back to loop start.
    pub fn check_and_apply_loop(&mut self) -> bool {
        if !self.has_loop() {
            return false;
        }

        if let Some(seek_time) = self.last_seek_time {
            if seek_time.elapsed() < Duration::from_millis(300) {
                return false;
            }
        }

        let start = self.state.mark_in.unwrap_or(Duration::ZERO);

        let should_loop = if let Some(out_pos) = self.state.mark_out {
            self.is_eos || self.position() >= out_pos
        } else if self.state.mark_in.is_some() {
            let dur = self.duration();
            let pos = self.position();
            let near_end = dur > Duration::from_millis(500)
                && (pos >= dur || dur.saturating_sub(pos) <= Duration::from_millis(200));
            self.is_eos || near_end
        } else {
            false
        };

        if should_loop {
            self.is_eos = false;
            let was_playing = self.state.is_playing;
            self.seek(start);
            if was_playing {
                self.play();
            }
            return true;
        }

        false
    }

    /// Returns true if video playback encountered an unrecoverable decoding, demuxing, or format error.
    pub fn is_failed(&self) -> bool {
        self.is_failed
    }

    /// Returns true if video reached end of stream (EOS).
    pub fn is_eos(&self) -> bool {
        self.is_eos
    }

    pub fn is_finished(&self) -> bool {
        if self.is_failed {
            return true;
        }
        if self.has_loop() {
            return false;
        }
        if self.is_eos {
            return true;
        }
        if !self.tracks_loaded {
            return false;
        }
        let dur = self.duration();
        if dur < Duration::from_millis(500) {
            return false;
        }
        unsafe {
            if !self.mpv.is_null() {
                let mut eof: std::ffi::c_int = 0;
                if let Ok(prop) = CString::new("eof-reached") {
                    let res = mpv_ffi::mpv_get_property(
                        self.mpv,
                        prop.as_ptr(),
                        mpv_ffi::MPV_FORMAT_FLAG,
                        &mut eof as *mut _ as *mut _,
                    );
                    if res == 0 && eof != 0 {
                        return true;
                    }
                }
            }
        }
        let pos = self.position();
        if pos >= dur || dur.saturating_sub(pos) <= Duration::from_millis(150) {
            if let Some(seek_time) = self.last_seek_time {
                if seek_time.elapsed() < Duration::from_millis(1500) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Check if video playback is stuck (same position for too long while marked playing)
    pub fn check_stuck(&mut self) -> bool {
        if self.is_failed {
            return true;
        }

        if !self.state.is_playing {
            self.state.stuck_count = 0;
            return false;
        }

        if self.has_loop() {
            self.check_and_apply_loop();
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
        if current_pos == self.state.last_checked_pos {
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

    pub fn path(&self) -> &str {
        &self.state.path
    }

    pub fn name(&self) -> &str {
        &self.state.name
    }

    pub fn audio_tracks(&self) -> &[AudioTrack] {
        &self.state.audio_tracks
    }

    pub fn current_audio_track_id(&self) -> Option<i64> {
        self.state.current_audio_track_id
    }

    pub fn set_audio_track(&mut self, track_id: i64) {
        unsafe {
            if self.mpv.is_null() {
                return;
            }
            let prop = CString::new("aid").unwrap();
            let mut id = track_id;
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut id as *mut _ as *mut _,
            );
            if let Ok(c_val) = CString::new(track_id.to_string()) {
                mpv_ffi::mpv_set_property_string(self.mpv, prop.as_ptr(), c_val.as_ptr());
            }
            self.state.current_audio_track_id = Some(track_id);
            for t in &mut self.state.audio_tracks {
                t.is_selected = t.id == track_id;
            }
            if let Some(track) = self.state.audio_tracks.iter().find(|t| t.id == track_id) {
                let pref = get_track_preference_string(track);
                if !pref.starts_with("Track ") {
                    self.preferred_audio_language = Some(pref.clone());
                    let alang = build_alang_string(&pref);
                    if !alang.is_empty() {
                        if let (Ok(c_prop), Ok(c_val)) =
                            (CString::new("alang"), CString::new(alang))
                        {
                            mpv_ffi::mpv_set_property_string(
                                self.mpv,
                                c_prop.as_ptr(),
                                c_val.as_ptr(),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn subtitle_tracks(&self) -> &[SubtitleTrack] {
        &self.state.subtitle_tracks
    }

    pub fn current_subtitle_track_id(&self) -> Option<i64> {
        self.state.current_subtitle_track_id
    }

    pub fn set_subtitle_track(&mut self, track_id: i64) {
        unsafe {
            if self.mpv.is_null() {
                return;
            }
            let prop = CString::new("sid").unwrap();
            let mut id = track_id;
            mpv_ffi::mpv_set_property(
                self.mpv,
                prop.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut id as *mut _ as *mut _,
            );
            if let Ok(c_val) = CString::new(track_id.to_string()) {
                mpv_ffi::mpv_set_property_string(self.mpv, prop.as_ptr(), c_val.as_ptr());
            }
            self.state.current_subtitle_track_id = Some(track_id);
            for t in &mut self.state.subtitle_tracks {
                t.is_selected = t.id == track_id;
            }
            if let Some(track) = self.state.subtitle_tracks.iter().find(|t| t.id == track_id) {
                let pref = get_subtitle_track_preference_string(track);
                if !pref.starts_with("Track ") {
                    self.preferred_subtitle_language = Some(pref.clone());
                    let slang = build_slang_string(&pref);
                    if !slang.is_empty() {
                        if let (Ok(c_prop), Ok(c_val)) = (CString::new("slang"), CString::new(slang)) {
                            mpv_ffi::mpv_set_property_string(self.mpv, c_prop.as_ptr(), c_val.as_ptr());
                        }
                    }
                }
            }
            self.set_subtitles_visible(true);
            if let Ok(cmd) = CString::new("no-osd seek 0 relative+exact") {
                mpv_ffi::mpv_command_string(self.mpv, cmd.as_ptr());
            }
        }
    }

    pub fn refresh_audio_tracks(&mut self) {
        if self.mpv.is_null() {
            return;
        }
        let count = self.get_property_i64("track-list/count").unwrap_or(0);
        let mut audio_tracks = Vec::new();
        let mut sub_tracks = Vec::new();
        for i in 0..count {
            let track_type = self.get_property_string(&format!("track-list/{}/type", i));
            let id = self
                .get_property_i64(&format!("track-list/{}/id", i))
                .unwrap_or(0);
            let title = self.get_property_string(&format!("track-list/{}/title", i));
            let lang = self.get_property_string(&format!("track-list/{}/lang", i));
            let codec = self.get_property_string(&format!("track-list/{}/codec", i));
            let selected = self
                .get_property_bool(&format!("track-list/{}/selected", i))
                .unwrap_or(false);

            if track_type.as_deref() == Some("audio") {
                audio_tracks.push(AudioTrack {
                    id,
                    title,
                    lang,
                    codec,
                    is_selected: selected,
                });
            } else if track_type.as_deref() == Some("sub") {
                let external_filename =
                    self.get_property_string(&format!("track-list/{}/external-filename", i));
                let ff_index = self.get_property_i64(&format!("track-list/{}/ff-index", i));
                let default = self
                    .get_property_bool(&format!("track-list/{}/default", i))
                    .unwrap_or(false);
                let forced = self
                    .get_property_bool(&format!("track-list/{}/forced", i))
                    .unwrap_or(false);
                sub_tracks.push(SubtitleTrack {
                    id,
                    title,
                    lang,
                    codec,
                    is_selected: selected,
                    is_default: default,
                    is_forced: forced,
                    external_filename,
                    ff_index,
                });
            }
        }
        // Query mpv for the currently active audio track.
        // Important: mpv's "aid" property could be set to "no", "auto", or an ID from a previous/different file.
        // We only consider an aid from mpv valid if it actually exists in this file's audio_tracks list.
        let mpv_selected_aid = self
            .get_property_i64("aid")
            .or_else(|| {
                self.get_property_string("aid")
                    .as_deref()
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .filter(|id| audio_tracks.iter().any(|t| t.id == *id))
            .or_else(|| audio_tracks.iter().find(|t| t.is_selected).map(|t| t.id));

        let mut current_aid = mpv_selected_aid.or(self.state.current_audio_track_id);

        // Auto-select preferred audio track on initial load of the file tracks
        if !self.tracks_loaded && !audio_tracks.is_empty() {
            let mut target_aid = None;
            if let Some(ref pref) = self.preferred_audio_language {
                if let Some(matching_id) = find_matching_audio_track(&audio_tracks, pref) {
                    target_aid = Some(matching_id);
                }
            }

            // Fallback: if no preference matched (or no preference configured) and mpv has no valid track selected,
            // default to the container's default audio track or the first available audio track.
            if target_aid.is_none() && current_aid.is_none() {
                target_aid = audio_tracks
                    .iter()
                    .find(|t| t.is_selected)
                    .map(|t| t.id)
                    .or_else(|| audio_tracks.first().map(|t| t.id));
            }

            if let Some(desired_id) = target_aid {
                if current_aid != Some(desired_id) {
                    self.set_audio_track(desired_id);
                    current_aid = Some(desired_id);
                }
            }
        }

        self.state.audio_tracks = audio_tracks;
        self.state.current_audio_track_id = current_aid;

        if let Some(aid) = current_aid {
            for t in &mut self.state.audio_tracks {
                t.is_selected = t.id == aid;
            }
        }

        // Subtitle tracks
        let is_sub_initial_load =
            !self.tracks_loaded || self.state.current_subtitle_track_id.is_none();
        let sid_i64 = self.get_property_i64("sid");
        let sid_str = self.get_property_string("sid");
        let mpv_selected_sid = sid_i64
            .or_else(|| sid_str.as_deref().and_then(|s| s.parse::<i64>().ok()))
            .filter(|id| sub_tracks.iter().any(|t| t.id == *id))
            .or_else(|| sub_tracks.iter().find(|t| t.is_selected).map(|t| t.id));

        let mut current_sid = mpv_selected_sid.or(self.state.current_subtitle_track_id);

        if is_sub_initial_load && !sub_tracks.is_empty() {
            let pref = self.preferred_subtitle_language.as_deref();
            let i18n_lang = self.i18n_language.as_deref();
            let effective_lang = self.effective_subtitle_language();

            let mut target_sid = None;

            if let Some(sid) = mpv_selected_sid {
                let active_track = sub_tracks.iter().find(|t| t.id == sid);
                let is_signs = active_track.map_or(false, is_signs_or_songs_track);
                let matches_pref = active_track.map_or(false, |t| {
                    subtitle_track_matches_preference(t, effective_lang)
                });

                // If mpv selected a Signs/Songs track, or a track that does not match the effective language
                // while an explicit preferred or i18n language match is available, select the matching track instead.
                if is_signs || !matches_pref {
                    if let Some(best) = select_best_subtitle_track_with_fallback(&sub_tracks, pref, i18n_lang) {
                        if best != sid {
                            target_sid = Some(best);
                        }
                    }
                }
            } else if sid_str.as_deref() == Some("no") || current_sid.is_some() {
                // mpv defaulted to no track (e.g. MKV container tracks with disposition 'default: 0')
                // or has no valid track selected. Select the best track according to preference/defaults.
                target_sid = select_best_subtitle_track_with_fallback(&sub_tracks, pref, i18n_lang);
            }

            if let Some(desired_id) = target_sid {
                if current_sid != Some(desired_id) {
                    log::info!(
                        "Auto-selecting subtitle track (id {}) on initial load (effective preference: {})",
                        desired_id,
                        effective_lang
                    );
                    self.set_subtitle_track(desired_id);
                    current_sid = Some(desired_id);
                }
            }
        }

        self.state.subtitle_tracks = sub_tracks;
        self.state.current_subtitle_track_id = current_sid;
        if let Some(sid) = current_sid {
            for t in &mut self.state.subtitle_tracks {
                t.is_selected = t.id == sid;
            }
        }

        if !self.state.audio_tracks.is_empty()
            || (!self.state.subtitle_tracks.is_empty()
                && (current_sid.is_some() || sid_str.as_deref() == Some("no")))
        {
            self.tracks_loaded = true;
        }
    }

    /// Returns the preferred subtitle language if known; otherwise falls back to the i18n language setting
    /// (defaulting to "en" if neither is known).
    pub fn effective_subtitle_language(&self) -> &str {
        self.preferred_subtitle_language
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| self.i18n_language.as_deref().filter(|s| !s.trim().is_empty()))
            .unwrap_or("en")
    }

    pub fn set_preferred_subtitle_language(&mut self, pref: Option<String>) {
        self.preferred_subtitle_language = pref;
        let slang = build_slang_string_with_fallback(
            self.preferred_subtitle_language.as_deref(),
            self.i18n_language.as_deref(),
        );
        if !slang.is_empty() {
            if let (Ok(c_prop), Ok(c_val)) = (CString::new("slang"), CString::new(slang)) {
                unsafe {
                    if !self.mpv.is_null() {
                        mpv_ffi::mpv_set_property_string(
                            self.mpv,
                            c_prop.as_ptr(),
                            c_val.as_ptr(),
                        );
                    }
                }
            }
        }
    }

    pub fn preferred_subtitle_language(&self) -> Option<&str> {
        self.preferred_subtitle_language.as_deref()
    }

    pub fn set_i18n_language(&mut self, lang: Option<String>) {
        self.i18n_language = lang;
        if self.preferred_subtitle_language.is_none() {
            let slang = build_slang_string_with_fallback(None, self.i18n_language.as_deref());
            if !slang.is_empty() {
                if let (Ok(c_prop), Ok(c_val)) = (CString::new("slang"), CString::new(slang)) {
                    unsafe {
                        if !self.mpv.is_null() {
                            mpv_ffi::mpv_set_property_string(
                                self.mpv,
                                c_prop.as_ptr(),
                                c_val.as_ptr(),
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn i18n_language(&self) -> Option<&str> {
        self.i18n_language.as_deref()
    }

    pub fn set_preferred_audio_language(&mut self, pref: Option<String>) {
        self.preferred_audio_language = pref.clone();
        if let Some(ref p) = pref {
            let alang = build_alang_string(p);
            if !alang.is_empty() {
                if let (Ok(c_prop), Ok(c_val)) = (CString::new("alang"), CString::new(alang)) {
                    unsafe {
                        if !self.mpv.is_null() {
                            mpv_ffi::mpv_set_property_string(
                                self.mpv,
                                c_prop.as_ptr(),
                                c_val.as_ptr(),
                            );
                        }
                    }
                }
            }
            let current_already_matches = self.state.current_audio_track_id.map_or(false, |aid| {
                self.state
                    .audio_tracks
                    .iter()
                    .any(|t| t.id == aid && track_matches_preference(t, p))
            });
            if !current_already_matches {
                if let Some(matching_id) = find_matching_audio_track(&self.state.audio_tracks, p) {
                    if self.state.current_audio_track_id != Some(matching_id) {
                        self.set_audio_track(matching_id);
                    }
                }
            }
        }
    }

    pub fn preferred_audio_language(&self) -> Option<&str> {
        self.preferred_audio_language.as_deref()
    }

    pub fn get_property_string(&self, name: &str) -> Option<String> {
        unsafe {
            if self.mpv.is_null() {
                return None;
            }
            let c_name = CString::new(name).ok()?;
            let ptr = mpv_ffi::mpv_get_property_string(self.mpv, c_name.as_ptr());
            if ptr.is_null() {
                None
            } else {
                let s = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
                mpv_ffi::mpv_free(ptr as *mut _);
                Some(s)
            }
        }
    }

    pub fn set_property_string(&mut self, name: &str, value: &str) -> i32 {
        unsafe {
            if self.mpv.is_null() {
                return -1;
            }
            if let (Ok(c_name), Ok(c_val)) = (CString::new(name), CString::new(value)) {
                mpv_ffi::mpv_set_property_string(self.mpv, c_name.as_ptr(), c_val.as_ptr())
            } else {
                -1
            }
        }
    }

    pub fn get_property_i64(&self, name: &str) -> Option<i64> {
        unsafe {
            if self.mpv.is_null() {
                return None;
            }
            let c_name = CString::new(name).ok()?;
            let mut val: i64 = 0;
            let res = mpv_ffi::mpv_get_property(
                self.mpv,
                c_name.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut val as *mut _ as *mut _,
            );
            if res == 0 { Some(val) } else { None }
        }
    }

    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        unsafe {
            if self.mpv.is_null() {
                return None;
            }
            let c_name = CString::new(name).ok()?;
            let mut val: std::ffi::c_int = 0;
            let res = mpv_ffi::mpv_get_property(
                self.mpv,
                c_name.as_ptr(),
                mpv_ffi::MPV_FORMAT_FLAG,
                &mut val as *mut _ as *mut _,
            );
            if res == 0 { Some(val != 0) } else { None }
        }
    }

    pub fn video_dimensions(&self) -> Option<(u32, u32)> {
        if let (Some(w), Some(h)) = (
            self.get_property_i64("dwidth").filter(|&v| v > 0),
            self.get_property_i64("dheight").filter(|&v| v > 0),
        ) {
            return Some((w as u32, h as u32));
        }
        if let (Some(w), Some(h)) = (
            self.get_property_i64("video-params/dw").filter(|&v| v > 0),
            self.get_property_i64("video-params/dh").filter(|&v| v > 0),
        ) {
            return Some((w as u32, h as u32));
        }
        if let (Some(w), Some(h)) = (
            self.get_property_i64("video-params/w").filter(|&v| v > 0),
            self.get_property_i64("video-params/h").filter(|&v| v > 0),
        ) {
            return Some((w as u32, h as u32));
        }
        None
    }

    pub fn aspect_ratio(&self) -> Option<f32> {
        if let Some((w, h)) = self.video_dimensions() {
            if h > 0 {
                return Some(w as f32 / h as f32);
            }
        }
        if let Some(s) = self.get_property_string("video-params/aspect") {
            if let Ok(val) = s.parse::<f32>() {
                if val > 0.05 {
                    return Some(val);
                }
            }
        }
        if let Ok(guard) = self.frame.lock() {
            if guard.width > 0 && guard.height > 0 {
                return Some(guard.width as f32 / guard.height as f32);
            }
        }
        None
    }

    fn check_update_render_dimensions(&mut self) {
        if let Some((w, h)) = self.video_dimensions() {
            if w > 0 && h > 0 {
                let max_dim = 1280.0f32;
                let scale = (max_dim / (w as f32).max(h as f32)).min(1.0);
                let target_w = (((w as f32 * scale).round() as u32).max(16) / 2) * 2;
                let target_h = (((h as f32 * scale).round() as u32).max(16) / 2) * 2;
                if self.render_width != target_w || self.render_height != target_h {
                    self.render_width = target_w;
                    self.render_height = target_h;
                    let buf_size = (target_w * target_h * 4) as usize;
                    self.pixel_buffer.resize(buf_size, 0);
                }
            }
        }
    }
}
