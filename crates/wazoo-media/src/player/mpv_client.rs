/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * libmpv FFI Client Wrapper
 *
 * Encapsulates raw `libmpv` handles, software render context setup/rendering,
 * property/command helpers, event translation, and non-blocking background teardown.
 */

use super::state::BufferConfig;
use super::tracks::{StartTime, build_alang_string, build_slang_string_with_fallback};
use crate::mpv_ffi;
use rand::Rng;
use std::ffi::{CStr, CString, c_int, c_void};

/// Observed mpv property changes translated from raw FFI event payloads.
pub(super) enum MpvProperty {
    DWidth(u32),
    DHeight(u32),
    Duration(f64),
    TimePos(f64),
    EofReached(bool),
    PausedForCache(bool),
}

/// Safe Rust representation of libmpv events processed by `VideoHandle::update_frame`.
pub(super) enum MpvEvent {
    StartFile,
    Seek,
    EndFile { is_error: bool, is_eof: bool },
    FileLoaded,
    TracksOrVideoReconfig,
    PlaybackRestart,
    PropertyChange(MpvProperty),
    Ignored,
}

/// Owns a `libmpv` instance and its software render context, guaranteeing safe
/// asynchronous teardown on drop without blocking the UI thread.
pub(super) struct MpvClient {
    mpv: *mut mpv_ffi::MpvHandle,
    render_ctx: *mut mpv_ffi::MpvRenderContext,
}

unsafe impl Send for MpvClient {}

impl Drop for MpvClient {
    fn drop(&mut self) {
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

impl MpvClient {
    /// Validate a media path/URL and strip any `file://` URI prefix.
    pub(super) fn validate_media_path(file_path: &str) -> Result<&str, String> {
        let clean_path = file_path.strip_prefix("file://").unwrap_or(file_path);
        if !clean_path.starts_with("http://")
            && !clean_path.starts_with("https://")
            && !std::path::Path::new(clean_path).exists()
        {
            return Err(format!("Media file does not exist: {clean_path}"));
        }
        Ok(clean_path)
    }

    /// Format a `StartTime` into an mpv `start` option value (`None` means no option needed at init).
    pub(super) fn format_start_option(start_time: StartTime) -> Option<String> {
        match start_time {
            StartTime::Beginning => None,
            StartTime::Seconds(start) => {
                if start > 0.05 {
                    Some(format!("{start:.3}"))
                } else {
                    None
                }
            }
            StartTime::Percent(pct) => {
                let clamped = pct.clamp(0.0, 95.0);
                Some(format!("{clamped:.1}%"))
            }
            StartTime::Random => {
                let rand_pct = rand::thread_rng().gen_range(5.0..85.0);
                Some(format!("{rand_pct:.1}%"))
            }
        }
    }

    /// Create, configure, and initialize a libmpv instance with a software render context,
    /// optionally loading `file_path` if non-empty.
    pub(super) fn create_and_configure(
        file_path: &str,
        config: &BufferConfig,
        start_time: StartTime,
        eq_filter: &str,
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
            // For software rendering context (MPV_RENDER_PARAM_API_TYPE = "sw"), direct hardware decoding
            // (e.g. vaapi, nvdec) is unsupported by libmpv and causes VA-API driver initialization errors
            // or thread deadlocks on X11 when windows are occluded or behind other windows.
            // Default to pure software decoding ("no") which is multi-threaded, robust, and zero-overhead.
            // Allow override via WAZOO_HWDEC environment variable (e.g. "auto-copy") for specialized setups.
            let hwdec = std::env::var("WAZOO_HWDEC").unwrap_or_else(|_| "no".to_string());
            set_opt("hwdec", &hwdec);
            // With BLOCK_FOR_TARGET_TIME=0, video-timing-offset must be set to 0.
            // By default, libmpv schedules video rendering 50ms (0.05s) ahead of the audio clock,
            // expecting mpv_render_context_render() to block until the target display time.
            // When blocking is disabled (BLOCK_FOR_TARGET_TIME=0), leaving timing-offset at 50ms causes
            // libmpv to perceive video as running ahead of audio, resulting in repeat frames,
            // timing recalculation stalls, or dropped frames during smooth camera pans.
            set_opt("video-timing-offset", "0");
            set_opt("correct-pts", "yes");
            set_opt("cache", "yes");
            let demuxer_mb = config.size_mb.clamp(8, 256);
            set_opt("demuxer-max-bytes", &format!("{demuxer_mb}M"));
            let readahead_secs = config.duration_secs.clamp(2, 5);
            set_opt("demuxer-readahead-secs", &format!("{readahead_secs}"));
            let back_mb = (demuxer_mb / 2).clamp(16, 64);
            set_opt("demuxer-max-back-bytes", &format!("{back_mb}M"));
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
            set_opt("volume", "100");

            if !eq_filter.is_empty() {
                set_opt("vf", eq_filter);
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

            if let Some(start_opt) = Self::format_start_option(start_time) {
                set_opt("start", &start_opt);
            }

            let res = mpv_ffi::mpv_initialize(mpv);
            if res < 0 {
                mpv_ffi::mpv_terminate_destroy(mpv);
                return Err(format!("Failed to initialize libmpv (code {res})"));
            }

            Self::observe_core_properties(mpv);

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
                let clean_path = match Self::validate_media_path(file_path) {
                    Ok(p) => p,
                    Err(err) => {
                        mpv_ffi::mpv_render_context_free(render_ctx);
                        mpv_ffi::mpv_terminate_destroy(mpv);
                        return Err(err);
                    }
                };
                let cmd_loadfile = match CString::new("loadfile") {
                    Ok(c) => c,
                    Err(e) => {
                        mpv_ffi::mpv_render_context_free(render_ctx);
                        mpv_ffi::mpv_terminate_destroy(mpv);
                        return Err(e.to_string());
                    }
                };
                let path_arg = match CString::new(clean_path) {
                    Ok(c) => c,
                    Err(e) => {
                        mpv_ffi::mpv_render_context_free(render_ctx);
                        mpv_ffi::mpv_terminate_destroy(mpv);
                        return Err(e.to_string());
                    }
                };
                let mut args: [*const std::ffi::c_char; 3] =
                    [cmd_loadfile.as_ptr(), path_arg.as_ptr(), std::ptr::null()];
                mpv_ffi::mpv_command_async(mpv, 0, args.as_mut_ptr());
            }

            Ok(Self { mpv, render_ctx })
        }
    }

    unsafe fn observe_core_properties(mpv: *mut mpv_ffi::MpvHandle) {
        let observed = [
            ("dwidth", mpv_ffi::MPV_FORMAT_INT64),
            ("dheight", mpv_ffi::MPV_FORMAT_INT64),
            ("time-pos", mpv_ffi::MPV_FORMAT_DOUBLE),
            ("duration", mpv_ffi::MPV_FORMAT_DOUBLE),
            ("eof-reached", mpv_ffi::MPV_FORMAT_FLAG),
            ("paused-for-cache", mpv_ffi::MPV_FORMAT_FLAG),
        ];
        for (idx, (name, fmt)) in observed.iter().enumerate() {
            if let Ok(c_name) = CString::new(*name) {
                unsafe {
                    mpv_ffi::mpv_observe_property(mpv, (idx + 1) as u64, c_name.as_ptr(), *fmt);
                }
            }
        }
    }

    /// Poll a single non-blocking event from libmpv; returns `None` when the queue is empty.
    pub(super) fn poll_event(&self) -> Option<MpvEvent> {
        if self.mpv.is_null() {
            return None;
        }
        unsafe {
            let event = mpv_ffi::mpv_wait_event(self.mpv, 0.0);
            if event.is_null() || (*event).event_id == mpv_ffi::MPV_EVENT_NONE {
                return None;
            }
            let event_id = (*event).event_id;
            let mapped = match event_id {
                mpv_ffi::MPV_EVENT_START_FILE => MpvEvent::StartFile,
                mpv_ffi::MPV_EVENT_SEEK => MpvEvent::Seek,
                mpv_ffi::MPV_EVENT_END_FILE => {
                    if !(*event).data.is_null() {
                        let end_data = &*((*event).data as *const mpv_ffi::MpvEventEndFile);
                        let is_error =
                            end_data.reason == mpv_ffi::MPV_END_FILE_REASON_ERROR || end_data.error != 0;
                        let is_eof = end_data.reason == mpv_ffi::MPV_END_FILE_REASON_EOF;
                        MpvEvent::EndFile { is_error, is_eof }
                    } else {
                        MpvEvent::EndFile {
                            is_error: false,
                            is_eof: true,
                        }
                    }
                }
                mpv_ffi::MPV_EVENT_FILE_LOADED => MpvEvent::FileLoaded,
                mpv_ffi::MPV_EVENT_TRACKS_CHANGED | mpv_ffi::MPV_EVENT_VIDEO_RECONFIG => {
                    MpvEvent::TracksOrVideoReconfig
                }
                mpv_ffi::MPV_EVENT_PLAYBACK_RESTART => MpvEvent::PlaybackRestart,
                mpv_ffi::MPV_EVENT_PROPERTY_CHANGE if !(*event).data.is_null() => {
                    let prop = &*((*event).data as *const mpv_ffi::MpvEventProperty);
                    if !prop.name.is_null() && !prop.data.is_null() {
                        let name = CStr::from_ptr(prop.name).to_string_lossy();
                        match (name.as_ref(), prop.format) {
                            ("dwidth", mpv_ffi::MPV_FORMAT_INT64) => {
                                let v = *(prop.data as *const i64);
                                if v > 0 {
                                    MpvEvent::PropertyChange(MpvProperty::DWidth(v as u32))
                                } else {
                                    MpvEvent::Ignored
                                }
                            }
                            ("dheight", mpv_ffi::MPV_FORMAT_INT64) => {
                                let v = *(prop.data as *const i64);
                                if v > 0 {
                                    MpvEvent::PropertyChange(MpvProperty::DHeight(v as u32))
                                } else {
                                    MpvEvent::Ignored
                                }
                            }
                            ("duration", mpv_ffi::MPV_FORMAT_DOUBLE) => {
                                let dur = *(prop.data as *const f64);
                                if dur > 0.0 && dur.is_finite() {
                                    MpvEvent::PropertyChange(MpvProperty::Duration(dur))
                                } else {
                                    MpvEvent::Ignored
                                }
                            }
                            ("time-pos", mpv_ffi::MPV_FORMAT_DOUBLE) => {
                                let pos = *(prop.data as *const f64);
                                if pos >= 0.0 && pos.is_finite() {
                                    MpvEvent::PropertyChange(MpvProperty::TimePos(pos))
                                } else {
                                    MpvEvent::Ignored
                                }
                            }
                            ("eof-reached", mpv_ffi::MPV_FORMAT_FLAG) => {
                                let flag = *(prop.data as *const c_int);
                                MpvEvent::PropertyChange(MpvProperty::EofReached(flag != 0))
                            }
                            ("paused-for-cache", mpv_ffi::MPV_FORMAT_FLAG) => {
                                let flag = *(prop.data as *const c_int);
                                MpvEvent::PropertyChange(MpvProperty::PausedForCache(flag != 0))
                            }
                            _ => MpvEvent::Ignored,
                        }
                    } else {
                        MpvEvent::Ignored
                    }
                }
                _ => MpvEvent::Ignored,
            };
            Some(mapped)
        }
    }

    /// Returns `true` when libmpv signals a new frame is ready for software rendering.
    pub(super) fn has_frame_update(&self) -> bool {
        if self.render_ctx.is_null() {
            return false;
        }
        let flags = unsafe { mpv_ffi::mpv_render_context_update(self.render_ctx) };
        (flags & mpv_ffi::MPV_RENDER_UPDATE_FRAME) != 0
    }

    /// Render the current video frame into `pixel_buffer` using `MPV_RENDER_API_TYPE_SW`.
    pub(super) fn render_sw_frame(
        &self,
        width: u32,
        height: u32,
        pixel_buffer: &mut Vec<u8>,
    ) -> bool {
        if self.render_ctx.is_null() || width == 0 || height == 0 {
            return false;
        }
        let mut size = [width as i32, height as i32];
        let format = c"rgb0";
        let mut stride = (width * 4) as usize;
        let mut block_target_time: c_int = 0;
        let buf_size = (width * height * 4) as usize;
        if pixel_buffer.len() != buf_size {
            pixel_buffer.resize(buf_size, 0);
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
                data: pixel_buffer.as_mut_ptr() as *mut c_void,
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

        // Do NOT call mpv_render_context_report_swap() here.
        // In software rendering mode (MPV_RENDER_API_TYPE_SW), the decoded frame is rendered
        // directly into an in-memory pixel buffer rather than flipped to a hardware display
        // swapchain managed by libmpv. Calling report_swap informs libmpv that the host is
        // tracking hardware presentation timings, which activates vo_libmpv's 200ms
        // (MP_TIME_MS_TO_NS(200)) swap-wait loop in flip_page(). When called prematurely on CPU
        // (prior to WGPU texture upload and VSync presentation) or during timer drift, libmpv
        // encounters 200ms flip timeouts, stalls the video clock, and forces drop/repeat recovery,
        // producing a transient 1/4 second backward frame jump or motion hitch during panning shots.
        // When report_swap is omitted (flip_count == 0), vo_libmpv skips the swap-wait loop entirely,
        // delivering glitch-free, sequential frame delivery.
        let err = unsafe {
            mpv_ffi::mpv_render_context_render(self.render_ctx, render_params.as_mut_ptr())
        };
        err == 0
    }

    pub(super) fn command_string(&self, cmd: &str) {
        if self.mpv.is_null() {
            return;
        }
        if let Ok(c_cmd) = CString::new(cmd) {
            unsafe {
                mpv_ffi::mpv_command_string(self.mpv, c_cmd.as_ptr());
            }
        }
    }

    pub(super) fn command_async(&self, args: &[&str]) -> Result<i32, String> {
        if self.mpv.is_null() {
            return Err("mpv handle is null".to_string());
        }
        let c_args: Vec<CString> = args
            .iter()
            .map(|a| CString::new(*a).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let mut ptrs: Vec<*const std::ffi::c_char> =
            c_args.iter().map(|c| c.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        let res = unsafe { mpv_ffi::mpv_command_async(self.mpv, 0, ptrs.as_mut_ptr()) };
        Ok(res)
    }

    pub(super) fn set_property_string(&self, name: &str, value: &str) -> i32 {
        if self.mpv.is_null() {
            return -1;
        }
        if let (Ok(c_name), Ok(c_val)) = (CString::new(name), CString::new(value)) {
            unsafe { mpv_ffi::mpv_set_property_string(self.mpv, c_name.as_ptr(), c_val.as_ptr()) }
        } else {
            -1
        }
    }

    pub(super) fn set_property_i64(&self, name: &str, mut value: i64) {
        if self.mpv.is_null() {
            return;
        }
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                mpv_ffi::mpv_set_property(
                    self.mpv,
                    c_name.as_ptr(),
                    mpv_ffi::MPV_FORMAT_INT64,
                    &mut value as *mut _ as *mut _,
                );
            }
        }
    }

    pub(super) fn set_property_double(&self, name: &str, value: f64) {
        if self.mpv.is_null() {
            return;
        }
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                mpv_ffi::mpv_set_property(
                    self.mpv,
                    c_name.as_ptr(),
                    mpv_ffi::MPV_FORMAT_DOUBLE,
                    &value as *const _ as *mut _,
                );
            }
        }
    }

    pub(super) fn set_property_async_flag(&self, name: &str, value: bool) {
        if self.mpv.is_null() {
            return;
        }
        if let Ok(c_name) = CString::new(name) {
            let mut flag: c_int = if value { 1 } else { 0 };
            unsafe {
                mpv_ffi::mpv_set_property_async(
                    self.mpv,
                    0,
                    c_name.as_ptr(),
                    mpv_ffi::MPV_FORMAT_FLAG,
                    &mut flag as *mut _ as *mut _,
                );
            }
        }
    }

    pub(super) fn set_property_async_double(&self, name: &str, mut value: f64) {
        if self.mpv.is_null() {
            return;
        }
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                mpv_ffi::mpv_set_property_async(
                    self.mpv,
                    0,
                    c_name.as_ptr(),
                    mpv_ffi::MPV_FORMAT_DOUBLE,
                    &mut value as *mut _ as *mut _,
                );
            }
        }
    }

    pub(super) fn get_property_string(&self, name: &str) -> Option<String> {
        if self.mpv.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        unsafe {
            let ptr = mpv_ffi::mpv_get_property_string(self.mpv, c_name.as_ptr());
            if ptr.is_null() {
                None
            } else {
                let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                mpv_ffi::mpv_free(ptr as *mut _);
                Some(s)
            }
        }
    }

    pub(super) fn get_property_i64(&self, name: &str) -> Option<i64> {
        if self.mpv.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        let mut val: i64 = 0;
        let res = unsafe {
            mpv_ffi::mpv_get_property(
                self.mpv,
                c_name.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut val as *mut _ as *mut _,
            )
        };
        if res == 0 { Some(val) } else { None }
    }

    pub(super) fn get_property_bool(&self, name: &str) -> Option<bool> {
        if self.mpv.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        let mut val: c_int = 0;
        let res = unsafe {
            mpv_ffi::mpv_get_property(
                self.mpv,
                c_name.as_ptr(),
                mpv_ffi::MPV_FORMAT_FLAG,
                &mut val as *mut _ as *mut _,
            )
        };
        if res == 0 { Some(val != 0) } else { None }
    }

    pub(super) fn get_property_double(&self, name: &str) -> Option<f64> {
        if self.mpv.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        let mut val: f64 = 0.0;
        let res = unsafe {
            mpv_ffi::mpv_get_property(
                self.mpv,
                c_name.as_ptr(),
                mpv_ffi::MPV_FORMAT_DOUBLE,
                &mut val as *mut _ as *mut _,
            )
        };
        if res == 0 { Some(val) } else { None }
    }
}
