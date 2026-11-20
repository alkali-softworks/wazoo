/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Handle
 *
 * Defines `VideoHandle` lifecycle (construction, placeholder promotion, file loading,
 * teardown) and low-level property forwarding. Playback, rendering, equalizer, and
 * track-control methods are implemented in sibling submodules under `player/`.
 */

use super::mpv_client::MpvClient;
use super::state::{BufferConfig, PlayerState};
use super::tracks::{PlayerId, StartTime};
use crate::pipeline::FrameData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct VideoHandle {
    pub id: PlayerId,
    pub state: PlayerState,
    pub(super) mpv: Option<MpvClient>,
    pub(super) render_width: u32,
    pub(super) render_height: u32,
    pub(super) pixel_buffer: Vec<u8>,
    pub(super) frame: Arc<Mutex<FrameData>>,
    pub(super) alive: Arc<AtomicBool>,
    pub(super) is_eos: bool,
    pub(super) is_failed: bool,
    pub(super) last_seek_time: Option<Instant>,
    pub(super) pending_seek: Option<Duration>,
    pub(super) pending_seek_random: bool,
    pub(super) tracks_loaded: bool,
    pub(super) preferred_audio_language: Option<String>,
    pub(super) preferred_subtitle_language: Option<String>,
    pub(super) i18n_language: Option<String>,
    pub(super) gamma: f64,
    pub(super) contrast: f64,
    pub(super) brightness: f64,
    pub(super) saturation: f64,
    pub(super) speed: f64,
    pub crt_enabled: bool,
    pub wavy_enabled: bool,
    pub fog_enabled: bool,
    pub crt_settings: wazoo_core::CrtSettings,
    pub is_seeking: bool,
    pub(super) seek_restart_received: bool,
    pub(super) crossfade_volume: f64,
    pub(super) cached_dwidth: Option<u32>,
    pub(super) cached_dheight: Option<u32>,
    pub(super) cached_eof_reached: bool,
    pub(super) cached_paused_for_cache: bool,
    pub(super) file_loaded: bool,
    pub(super) needs_render_dim_refresh: bool,
    pub(super) is_network_source: bool,
}

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
        // `self.mpv` (`MpvClient`) is dropped right after, triggering async stop + teardown thread.
    }
}

impl VideoHandle {
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

    /// Creates an idle, pre-warmed `VideoHandle` with libmpv and render context initialized,
    /// ready to load a media file instantly via `load_file()` without blocking the UI thread.
    pub fn new_idle(config: BufferConfig) -> Result<Self, String> {
        Self::with_buffering_and_start(0, "", "", config, StartTime::Beginning)
    }

    /// Creates a lightweight placeholder `VideoHandle` with no libmpv instance or disk/network I/O.
    /// Used while the real libmpv handle is initialized asynchronously on a background thread.
    pub fn new_placeholder(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
        start_time: impl Into<StartTime>,
    ) -> Self {
        Self::from_parts(id, file_path, name, config, start_time.into(), None)
    }

    pub fn with_buffering_and_start(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
        start_time: impl Into<StartTime>,
    ) -> Result<Self, String> {
        let start_time = start_time.into();
        let eq_filter = Self::build_eq_filter_string(
            config.gamma,
            config.contrast,
            config.brightness,
            config.saturation,
        );
        let mpv = MpvClient::create_and_configure(file_path, &config, start_time, &eq_filter)?;
        Ok(Self::from_parts(
            id,
            file_path,
            name,
            config,
            start_time,
            Some(mpv),
        ))
    }

    fn from_parts(
        id: PlayerId,
        file_path: &str,
        name: &str,
        config: BufferConfig,
        start_time: StartTime,
        mpv: Option<MpvClient>,
    ) -> Self {
        let mut state = PlayerState::new(id, file_path.to_string(), name.to_string());
        let (pending_seek, last_seek_time) = Self::initial_seek_tracking(start_time, &mut state);

        Self {
            id,
            state,
            mpv,
            render_width: 0,
            render_height: 0,
            pixel_buffer: Vec::new(),
            frame: Arc::new(Mutex::new(FrameData {
                width: 0,
                height: 0,
                pixels: Vec::new(),
                new_frame: false,
                frame_seq: 0,
            })),
            alive: Arc::new(AtomicBool::new(true)),
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
            crt_settings: config.crt_settings,
            is_seeking: false,
            seek_restart_received: false,
            crossfade_volume: 1.0,
            cached_dwidth: None,
            cached_dheight: None,
            cached_eof_reached: false,
            cached_paused_for_cache: false,
            file_loaded: false,
            needs_render_dim_refresh: false,
            is_network_source: Self::is_network_path(file_path),
        }
    }

    fn initial_seek_tracking(
        start_time: StartTime,
        state: &mut PlayerState,
    ) -> (Option<Duration>, Option<Instant>) {
        match start_time {
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
        }
    }

    #[inline]
    fn is_network_path(path: &str) -> bool {
        let lower = path.trim().to_ascii_lowercase();
        lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("smb://")
            || lower.starts_with("nfs://")
    }

    #[inline]
    pub fn is_placeholder(&self) -> bool {
        self.mpv.is_none()
    }

    #[inline]
    pub fn is_network_stream(&self) -> bool {
        self.is_network_source
    }

    #[inline]
    pub fn is_buffering_for_cache(&self) -> bool {
        self.cached_paused_for_cache || self.is_placeholder()
    }

    pub fn path(&self) -> &str {
        &self.state.path
    }

    pub fn name(&self) -> &str {
        &self.state.name
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
        if self.mpv.is_none() {
            return Err("mpv handle is null".to_string());
        }

        let clean_path = MpvClient::validate_media_path(file_path)?.to_string();
        let start_opt = MpvClient::format_start_option(start_time).unwrap_or_else(|| "0".to_string());
        self.set_property_string("start", &start_opt);

        // Immediately clear frame to solid black so no frozen frame of prior video is visible
        self.clear_frame_opaque_black();

        if let Some(ref mpv) = self.mpv {
            let res = mpv.command_async(&["loadfile", &clean_path, "replace"])?;
            if res < 0 {
                return Err(format!("Failed to load file: {res}"));
            }
            mpv.set_property_async_flag("pause", false);
        }

        self.render_width = 0;
        self.render_height = 0;
        self.state.path = file_path.to_string();
        self.state.name = name.to_string();
        self.state.duration = Duration::ZERO;
        self.state.position = Duration::ZERO;
        self.state.is_playing = true;
        self.is_eos = false;
        self.is_failed = false;
        self.tracks_loaded = false;
        self.cached_dwidth = None;
        self.cached_dheight = None;
        self.cached_eof_reached = false;
        self.cached_paused_for_cache = false;
        self.file_loaded = false;
        self.needs_render_dim_refresh = false;
        self.is_network_source = Self::is_network_path(file_path);
        self.state.audio_tracks.clear();
        self.state.subtitle_tracks.clear();
        self.state.current_audio_track_id = None;
        self.state.current_subtitle_track_id = None;

        let (pending_seek, last_seek_time) =
            Self::initial_seek_tracking(start_time, &mut self.state);
        self.pending_seek = pending_seek;
        self.last_seek_time = Some(last_seek_time.unwrap_or_else(Instant::now));
        self.pending_seek_random = false;
        self.is_seeking = true;
        self.seek_restart_received = false;

        Ok(())
    }

    pub fn get_property_string(&self, name: &str) -> Option<String> {
        self.mpv.as_ref()?.get_property_string(name)
    }

    pub fn set_property_string(&mut self, name: &str, value: &str) -> i32 {
        self.mpv
            .as_ref()
            .map_or(-1, |mpv| mpv.set_property_string(name, value))
    }

    pub fn get_property_i64(&self, name: &str) -> Option<i64> {
        self.mpv.as_ref()?.get_property_i64(name)
    }

    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        self.mpv.as_ref()?.get_property_bool(name)
    }
}
