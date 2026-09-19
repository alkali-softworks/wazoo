/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Handle
 *
 * Encapsulates an individual video player instance powered by libmpv, providing playback
 * controls (play, pause, seek, volume, mute), texture rendering, and watchdog monitoring.
 */

use rand::Rng;
use std::ffi::{CString, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::mpv_ffi;
use crate::pipeline::FrameData;

pub type PlayerId = usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StartTime {
    Beginning,
    Seconds(f64),
    Percent(f64),
    Random,
}

impl From<Option<f64>> for StartTime {
    fn from(opt: Option<f64>) -> Self {
        match opt {
            Some(s) => StartTime::Seconds(s),
            None => StartTime::Beginning,
        }
    }
}

impl From<f64> for StartTime {
    fn from(s: f64) -> Self {
        StartTime::Seconds(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioTrack {
    pub id: i64,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubtitleTrack {
    pub id: i64,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    pub is_selected: bool,
    pub external_filename: Option<String>,
    pub ff_index: Option<i64>,
}

pub fn language_display_name(code: &str) -> &'static str {
    let lower = code.trim().to_ascii_lowercase();
    match lower.as_str() {
        "en" | "eng" => "English",
        "ja" | "jpn" => "Japanese",
        "es" | "spa" => "Spanish",
        "fr" | "fra" | "fre" => "French",
        "de" | "deu" | "ger" => "German",
        "it" | "ita" => "Italian",
        "pt" | "por" => "Portuguese",
        "ru" | "rus" => "Russian",
        "zh" | "zho" | "chi" => "Chinese",
        "ko" | "kor" => "Korean",
        "ar" | "ara" => "Arabic",
        "he" | "heb" => "Hebrew",
        "hi" | "hin" => "Hindi",
        "bn" | "ben" => "Bengali",
        "id" | "ind" => "Indonesian",
        "th" | "tha" => "Thai",
        "vi" | "vie" => "Vietnamese",
        "nl" | "nld" | "dut" => "Dutch",
        "pl" | "pol" => "Polish",
        "tr" | "tur" => "Turkish",
        "uk" | "ukr" => "Ukrainian",
        "sv" | "swe" => "Swedish",
        "no" | "nor" => "Norwegian",
        "da" | "dan" => "Danish",
        "fi" | "fin" => "Finnish",
        "el" | "ell" | "gre" => "Greek",
        "cs" | "ces" | "cze" => "Czech",
        "hu" | "hun" => "Hungarian",
        "ro" | "ron" | "rum" => "Romanian",
        _ => "",
    }
}

pub fn format_audio_track_label(track: &AudioTrack, index: usize) -> String {
    let cleaned_title = track.title.as_ref().and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            return None;
        }
        let first_seg = if trimmed.contains(" / ") {
            trimmed.split(" / ").next().unwrap_or(trimmed).trim()
        } else {
            trimmed.split('/').next().unwrap_or(trimmed).trim()
        };
        if first_seg.is_empty() {
            Some(trimmed.to_string())
        } else if first_seg.chars().count() > 65 {
            Some(format!(
                "{}...",
                first_seg.chars().take(62).collect::<String>()
            ))
        } else {
            Some(first_seg.to_string())
        }
    });

    let lang_display = track.lang.as_ref().and_then(|l| {
        let display = language_display_name(l);
        if !display.is_empty() {
            Some(display.to_string())
        } else {
            let trimmed = l.trim().to_ascii_uppercase();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None
            }
        }
    });

    match (cleaned_title, lang_display) {
        (Some(title), Some(lang)) => {
            if title
                .to_ascii_lowercase()
                .contains(&lang.to_ascii_lowercase())
            {
                title
            } else {
                format!("{lang} ({title})")
            }
        }
        (Some(title), None) => title,
        (None, Some(lang)) => lang,
        (None, None) => format!("Track {}", index + 1),
    }
}

pub fn format_subtitle_track_label(track: &SubtitleTrack, index: usize) -> String {
    let lang_display = track.lang.as_ref().and_then(|l| {
        let display = language_display_name(l);
        if !display.is_empty() {
            Some(display.to_string())
        } else if l.eq_ignore_ascii_case("enm") {
            Some("English".to_string())
        } else {
            let trimmed = l.trim().to_ascii_uppercase();
            if !trimmed.is_empty() {
                Some(trimmed)
            } else {
                None
            }
        }
    });

    let cleaned_title = track.title.as_ref().and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            return None;
        }
        let lower = trimmed.to_ascii_lowercase();
        let qualifier = if lower.contains("honorific") || lower.contains("honorofic") {
            Some("Honorifics")
        } else if lower.contains("sign") || lower.contains("song") {
            Some("Signs & Songs")
        } else if lower.contains("sdh") {
            Some("SDH")
        } else if lower.contains("forced") {
            Some("Forced")
        } else {
            None
        };

        let first_seg = if trimmed.contains(" / ") {
            trimmed.split(" / ").next().unwrap_or(trimmed).trim()
        } else {
            trimmed.split('/').next().unwrap_or(trimmed).trim()
        };
        let first_lower = first_seg.to_ascii_lowercase();
        if let Some(q) = qualifier {
            let already_has_qualifier = match q {
                "Honorifics" => {
                    first_lower.contains("honorific") || first_lower.contains("honorofic")
                }
                "Signs & Songs" => first_lower.contains("sign") || first_lower.contains("song"),
                "SDH" => first_lower.contains("sdh"),
                "Forced" => first_lower.contains("forced"),
                _ => first_lower.contains(&q.to_ascii_lowercase()),
            };
            if already_has_qualifier {
                Some(first_seg.to_string())
            } else {
                Some(format!("{first_seg} - {q}"))
            }
        } else if first_seg.is_empty() {
            Some(trimmed.to_string())
        } else if first_seg.chars().count() > 65 {
            Some(format!(
                "{}...",
                first_seg.chars().take(62).collect::<String>()
            ))
        } else {
            Some(first_seg.to_string())
        }
    });

    match (cleaned_title, lang_display) {
        (Some(title), Some(lang)) => {
            if title
                .to_ascii_lowercase()
                .contains(&lang.to_ascii_lowercase())
            {
                title
            } else {
                format!("{lang} ({title})")
            }
        }
        (Some(title), None) => title,
        (None, Some(lang)) => lang,
        (None, None) => format!("Track {}", index + 1),
    }
}

pub fn language_aliases(name_or_code: &str) -> Vec<String> {
    let lower = name_or_code.trim().to_ascii_lowercase();
    let mut aliases = Vec::new();
    match lower.as_str() {
        "ja" | "jpn" | "jp" | "japanese" => {
            aliases.extend(vec![
                "ja".into(),
                "jpn".into(),
                "jp".into(),
                "japanese".into(),
            ]);
        }
        "en" | "eng" | "english" => {
            aliases.extend(vec!["en".into(), "eng".into(), "english".into()]);
        }
        "es" | "spa" | "spanish" => {
            aliases.extend(vec!["es".into(), "spa".into(), "spanish".into()]);
        }
        "fr" | "fra" | "fre" | "french" => {
            aliases.extend(vec![
                "fr".into(),
                "fra".into(),
                "fre".into(),
                "french".into(),
            ]);
        }
        "de" | "deu" | "ger" | "german" => {
            aliases.extend(vec![
                "de".into(),
                "deu".into(),
                "ger".into(),
                "german".into(),
            ]);
        }
        "it" | "ita" | "italian" => {
            aliases.extend(vec!["it".into(), "ita".into(), "italian".into()]);
        }
        "pt" | "por" | "portuguese" => {
            aliases.extend(vec!["pt".into(), "por".into(), "portuguese".into()]);
        }
        "ru" | "rus" | "russian" => {
            aliases.extend(vec!["ru".into(), "rus".into(), "russian".into()]);
        }
        "zh" | "zho" | "chi" | "chinese" => {
            aliases.extend(vec![
                "zh".into(),
                "zho".into(),
                "chi".into(),
                "chinese".into(),
            ]);
        }
        "ko" | "kor" | "korean" => {
            aliases.extend(vec!["ko".into(), "kor".into(), "korean".into()]);
        }
        _ => {
            let display = language_display_name(&lower);
            if !display.is_empty() {
                aliases.push(display.to_ascii_lowercase());
            }
            if !lower.is_empty() {
                aliases.push(lower);
            }
        }
    }
    aliases
}

pub fn build_alang_string(preferred: &str) -> String {
    let aliases = language_aliases(preferred);
    aliases.join(",")
}

pub fn get_track_preference_string(track: &AudioTrack) -> String {
    if let Some(ref l) = track.lang {
        let display = language_display_name(l);
        if !display.is_empty() {
            return display.to_string();
        }
        let trimmed = l.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(ref _t) = track.title {
        let cleaned = format_audio_track_label(track, 0);
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    format!("Track {}", track.id)
}

pub fn find_matching_audio_track(tracks: &[AudioTrack], preferred: &str) -> Option<i64> {
    if preferred.trim().is_empty() {
        return None;
    }
    let aliases = language_aliases(preferred);

    // Pass 1: exact match on track.lang against any alias
    for t in tracks {
        if let Some(ref l) = t.lang {
            let lower_lang = l.trim().to_ascii_lowercase();
            if aliases.iter().any(|a| a == &lower_lang) {
                return Some(t.id);
            }
            let display = language_display_name(&lower_lang).to_ascii_lowercase();
            if !display.is_empty() && aliases.iter().any(|a| a == &display) {
                return Some(t.id);
            }
        }
    }

    // Pass 2: match on track.title containing any alias as a substring
    for t in tracks {
        if let Some(ref title) = t.title {
            let lower_title = title.to_ascii_lowercase();
            if aliases.iter().any(|a| lower_title.contains(a)) {
                return Some(t.id);
            }
        }
    }

    // Pass 3: match formatted label
    for (i, t) in tracks.iter().enumerate() {
        let label = format_audio_track_label(t, i).to_ascii_lowercase();
        if aliases.iter().any(|a| label.contains(a)) {
            return Some(t.id);
        }
    }

    None
}

pub fn track_matches_preference(track: &AudioTrack, preferred: &str) -> bool {
    if preferred.trim().is_empty() {
        return false;
    }
    let aliases = language_aliases(preferred);
    if let Some(ref l) = track.lang {
        let lower_lang = l.trim().to_ascii_lowercase();
        if aliases.iter().any(|a| a == &lower_lang) {
            return true;
        }
        let display = language_display_name(&lower_lang).to_ascii_lowercase();
        if !display.is_empty() && aliases.iter().any(|a| a == &display) {
            return true;
        }
    }
    if let Some(ref title) = track.title {
        let lower_title = title.to_ascii_lowercase();
        if aliases.iter().any(|a| lower_title.contains(a)) {
            return true;
        }
    }
    let label = format_audio_track_label(track, 0).to_ascii_lowercase();
    if aliases.iter().any(|a| label.contains(a)) {
        return true;
    }
    false
}

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
    pub audio_tracks: Vec<AudioTrack>,
    pub current_audio_track_id: Option<i64>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub current_subtitle_track_id: Option<i64>,
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
            audio_tracks: Vec::new(),
            current_audio_track_id: None,
            subtitle_tracks: Vec::new(),
            current_subtitle_track_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub duration_secs: u32,
    pub size_mb: u32,
    pub read_chunk_kb: u32,
    pub preferred_audio_language: Option<String>,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            duration_secs: 10,
            size_mb: 32,
            read_chunk_kb: 512,
            preferred_audio_language: None,
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
    frame: Arc<Mutex<FrameData>>,
    alive: Arc<AtomicBool>,
    is_eos: bool,
    last_seek_time: Option<Instant>,
    pending_seek: Option<Duration>,
    pending_seek_random: bool,
    tracks_loaded: bool,
    preferred_audio_language: Option<String>,
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
        Self::with_buffering_and_start(id, file_path, name, config, None)
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
            set_opt("demuxer-max-bytes", &format!("{}M", config.size_mb.max(32)));
            set_opt(
                "demuxer-readahead-secs",
                &format!("{}", config.duration_secs.max(10)),
            );
            set_opt("demuxer-max-back-bytes", "32M");
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

            if let Some(ref pref) = config.preferred_audio_language {
                let alang = build_alang_string(pref);
                if !alang.is_empty() {
                    set_opt("alang", &alang);
                }
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

            // Non-blocking pump of initial events so the UI thread is not frozen (bounded)
            let mut init_events = 0;
            while !mpv.is_null() && init_events < 64 {
                init_events += 1;
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

            let mut render_width = 1280u32;
            let mut render_height = 720u32;
            let mut init_w: i64 = 0;
            let mut init_h: i64 = 0;
            let prop_w = CString::new("dwidth").unwrap();
            let prop_h = CString::new("dheight").unwrap();
            let res_w = mpv_ffi::mpv_get_property(
                mpv,
                prop_w.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut init_w as *mut _ as *mut _,
            );
            let res_h = mpv_ffi::mpv_get_property(
                mpv,
                prop_h.as_ptr(),
                mpv_ffi::MPV_FORMAT_INT64,
                &mut init_h as *mut _ as *mut _,
            );
            if res_w == 0 && res_h == 0 && init_w > 0 && init_h > 0 {
                let max_dim = 1280.0f32;
                let scale = (max_dim / (init_w as f32).max(init_h as f32)).min(1.0);
                render_width = (((init_w as f32 * scale).round() as u32).max(16) / 2) * 2;
                render_height = (((init_h as f32 * scale).round() as u32).max(16) / 2) * 2;
            }
            let buffer_size = (render_width * render_height * 4) as usize;
            let mut pixel_buffer = vec![0u8; buffer_size];
            for chunk in pixel_buffer.chunks_exact_mut(4) {
                chunk[3] = 255;
            }
            let alive = Arc::new(AtomicBool::new(true));
            let frame = Arc::new(Mutex::new(FrameData {
                width: render_width,
                height: render_height,
                pixels: pixel_buffer.clone(),
                new_frame: true,
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
                last_seek_time,
                pending_seek,
                pending_seek_random: false,
                tracks_loaded: false,
                preferred_audio_language: config.preferred_audio_language,
            };

            handle.set_volume(1.0);
            handle.refresh_audio_tracks();
            Ok(handle)
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
                if (*event).event_id == mpv_ffi::MPV_EVENT_END_FILE {
                    self.is_eos = true;
                }
                if (*event).event_id == mpv_ffi::MPV_EVENT_FILE_LOADED
                    || (*event).event_id == mpv_ffi::MPV_EVENT_TRACKS_CHANGED
                    || (*event).event_id == mpv_ffi::MPV_EVENT_PLAYBACK_RESTART
                    || (*event).event_id == mpv_ffi::MPV_EVENT_VIDEO_RECONFIG
                {
                    needs_refresh_tracks = true;
                    self.tracks_loaded = true;
                    if let Some(target) = self.pending_seek.take() {
                        self.last_seek_time = Some(Instant::now());
                        let cmd = format!("no-osd seek {:.3} absolute+exact", target.as_secs_f64());
                        if let Ok(c_cmd) = CString::new(cmd) {
                            mpv_ffi::mpv_command_string(self.mpv, c_cmd.as_ptr());
                        }
                    } else if self.pending_seek_random {
                        let dur = self.duration();
                        if dur > Duration::from_secs(2) {
                            self.pending_seek_random = false;
                            let max_secs = dur.as_secs_f64();
                            let rand_secs = rand::thread_rng().gen_range(0.0..max_secs);
                            self.seek_fast(Duration::from_secs_f64(rand_secs));
                        }
                    }
                }
            }

            if needs_refresh_tracks {
                self.refresh_audio_tracks();
            } else if !self.tracks_loaded {
                let count = self.get_property_i64("track-list/count").unwrap_or(0);
                if count > 0 {
                    self.refresh_audio_tracks();
                    self.tracks_loaded = true;
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
                    let max_secs = dur.as_secs_f64();
                    let rand_secs = rand::thread_rng().gen_range(0.0..max_secs);
                    self.seek_fast(Duration::from_secs_f64(rand_secs));
                }
            }

            if self.render_ctx.is_null() {
                return false;
            }

            self.check_update_render_dimensions();

            let flags = mpv_ffi::mpv_render_context_update(self.render_ctx);
            if (flags & mpv_ffi::MPV_RENDER_UPDATE_FRAME) != 0 {
                let mut size = [self.render_width as i32, self.render_height as i32];
                let format = c"rgb0";
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

                let err =
                    mpv_ffi::mpv_render_context_render(self.render_ctx, render_params.as_mut_ptr());
                if err == 0 {
                    mpv_ffi::mpv_render_context_report_swap(self.render_ctx);

                    for chunk in self.pixel_buffer.chunks_exact_mut(4) {
                        chunk[3] = 0xFF;
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
        let program = crate::pipeline::VideoProgram::new_with_fit(
            self.id as u64,
            Arc::clone(&self.frame),
            Arc::clone(&self.alive),
            opacity,
            fit_cover,
        );
        iced::Element::new(crate::pipeline::video_shader(program))
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

    pub fn has_decoded_frame(&self) -> bool {
        if let Ok(guard) = self.frame.lock() {
            !guard.pixels.is_empty() && guard.width > 0 && guard.height > 0
        } else {
            false
        }
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

    pub fn is_finished(&self) -> bool {
        if self.is_eos {
            return true;
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
        let dur = self.duration();
        if dur > Duration::from_millis(500) {
            let pos = self.position();
            if pos >= dur || dur.saturating_sub(pos) <= Duration::from_millis(150) {
                if let Some(seek_time) = self.last_seek_time {
                    if seek_time.elapsed() < Duration::from_millis(1000) {
                        return false;
                    }
                }
                return true;
            }
        }
        false
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
                sub_tracks.push(SubtitleTrack {
                    id,
                    title,
                    lang,
                    codec,
                    is_selected: selected,
                    external_filename,
                    ff_index,
                });
            }
        }
        let aid_i64 = self.get_property_i64("aid");
        let aid_str = self.get_property_string("aid");
        let mut current_aid = aid_i64
            .or_else(|| aid_str.as_deref().and_then(|s| s.parse::<i64>().ok()))
            .or_else(|| audio_tracks.iter().find(|t| t.is_selected).map(|t| t.id))
            .or_else(|| audio_tracks.first().map(|t| t.id));
        self.state.audio_tracks = audio_tracks;
        self.state.current_audio_track_id = current_aid;

        // Auto-select preferred audio track on initial load if configured and not already matching
        let is_initial_load = !self.tracks_loaded || self.state.current_audio_track_id.is_none();
        if is_initial_load {
            if let Some(ref pref) = self.preferred_audio_language {
                let current_already_matches = current_aid.map_or(false, |aid| {
                    self.state
                        .audio_tracks
                        .iter()
                        .any(|t| t.id == aid && track_matches_preference(t, pref))
                });

                if !current_already_matches {
                    if let Some(matching_id) = find_matching_audio_track(&self.state.audio_tracks, pref) {
                        if current_aid != Some(matching_id) {
                            self.set_audio_track(matching_id);
                            current_aid = Some(matching_id);
                        }
                    }
                }
            }
        }

        if let Some(aid) = current_aid {
            for t in &mut self.state.audio_tracks {
                t.is_selected = t.id == aid;
            }
        }

        // Subtitle tracks
        let is_initial_load = !self.tracks_loaded || self.state.current_subtitle_track_id.is_none();
        let sid_i64 = self.get_property_i64("sid");
        let sid_str = self.get_property_string("sid");
        let mpv_selected_sid = sid_i64
            .or_else(|| sid_str.as_deref().and_then(|s| s.parse::<i64>().ok()));
        let mut current_sid = mpv_selected_sid
            .or_else(|| self.state.current_subtitle_track_id)
            .or_else(|| sub_tracks.iter().find(|t| t.is_selected).map(|t| t.id))
            .or_else(|| sub_tracks.first().map(|t| t.id));

        if is_initial_load && !sub_tracks.is_empty() {
            if let Some(sid) = mpv_selected_sid {
                // If mpv defaulted to a Signs/Songs track (which only contains signs/lyrics, no dialogue),
                // and a full dialogue subtitle track exists (or external subtitle sidecar), automatically promote to the full track on initial load.
                if let Some(active_track) = sub_tracks.iter().find(|t| t.id == sid) {
                    let is_signs = active_track
                        .title
                        .as_ref()
                        .map(|t| {
                            let l = t.to_ascii_lowercase();
                            l.contains("sign") || l.contains("song")
                        })
                        .unwrap_or(false);

                    if is_signs {
                        let preferred_full_track = sub_tracks
                            .iter()
                            .find(|t| {
                                if t.id == sid {
                                    return false;
                                }
                                if let Some(ref title) = t.title {
                                    let l = title.to_ascii_lowercase();
                                    if l.contains("sign") || l.contains("song") {
                                        return false;
                                    }
                                    if l.contains("full") {
                                        return true;
                                    }
                                }
                                t.external_filename.is_some()
                            })
                            .or_else(|| {
                                sub_tracks.iter().find(|t| {
                                    if t.id == sid {
                                        return false;
                                    }
                                    if let Some(ref title) = t.title {
                                        let l = title.to_ascii_lowercase();
                                        if l.contains("sign") || l.contains("song") {
                                            return false;
                                        }
                                    }
                                    true
                                })
                            });

                        if let Some(full_track) = preferred_full_track {
                            log::info!(
                                "Auto-promoting subtitle track from Signs/Songs (id {}) to full dialogue track (id {})",
                                sid,
                                full_track.id
                            );
                            self.set_subtitle_track(full_track.id);
                            current_sid = Some(full_track.id);
                        }
                    }
                }
            } else if let Some(target_id) = current_sid {
                // mpv did not automatically select a subtitle track (e.g. MKV container tracks with disposition 'default: 0').
                // Select the preferred full dialogue track (or first available track) and explicitly configure mpv.
                let best_track = sub_tracks
                    .iter()
                    .find(|t| {
                        if let Some(ref title) = t.title {
                            let l = title.to_ascii_lowercase();
                            if l.contains("full") {
                                return true;
                            }
                        }
                        t.external_filename.is_some()
                    })
                    .or_else(|| {
                        sub_tracks.iter().find(|t| {
                            if let Some(ref title) = t.title {
                                let l = title.to_ascii_lowercase();
                                if l.contains("sign") || l.contains("song") {
                                    return false;
                                }
                            }
                            true
                        })
                    })
                    .map(|t| t.id)
                    .unwrap_or(target_id);

                log::info!(
                    "Auto-selecting subtitle track (id {}) on initial load (mpv defaulted to no track)",
                    best_track
                );
                self.set_subtitle_track(best_track);
                current_sid = Some(best_track);
            }
        }

        self.state.subtitle_tracks = sub_tracks;
        self.state.current_subtitle_track_id = current_sid;
        if let Some(sid) = current_sid {
            for t in &mut self.state.subtitle_tracks {
                t.is_selected = t.id == sid;
            }
        }
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
                    for chunk in self.pixel_buffer.chunks_exact_mut(4) {
                        chunk[3] = 255;
                    }
                }
            }
        }
    }
}

