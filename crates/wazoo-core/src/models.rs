/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Domain Models & Settings
 *
 * Defines core data structures including VideoRecord, VideoSession, Bookmark, LayoutMode,
 * PlaybackMode, and persistent WazooSettings with serialization support.
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub folder: Option<String>,
}

impl VideoRecord {
    pub fn new(id: i64, name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            path: path.into(),
            folder: None,
        }
    }

    pub fn with_folder(mut self, folder: impl Into<String>) -> Self {
        self.folder = Some(folder.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    Column,
    Row,
    #[default]
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackMode {
    #[default]
    Normal,
    Scroll,
    Flip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadingIndicator {
    #[default]
    TvStatic,
    Spinner,
}

impl LoadingIndicator {
    pub const ALL: &'static [LoadingIndicator] =
        &[LoadingIndicator::TvStatic, LoadingIndicator::Spinner];
}

impl std::fmt::Display for LoadingIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TvStatic => write!(f, "TV Static"),
            Self::Spinner => write!(f, "Spinner"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowBounds {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 720,
        }
    }
}

fn default_buffer_duration_secs() -> u32 {
    10
}

fn default_buffer_size_mb() -> u32 {
    32
}

fn default_session_volume() -> f64 {
    1.0
}

fn default_language() -> String {
    "en".to_string()
}

pub const DEFAULT_FLIP_INTERVAL_SECS: u64 = 45;

fn default_flip_interval_secs() -> u64 {
    DEFAULT_FLIP_INTERVAL_SECS
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoSession {
    pub path: String,
    #[serde(default)]
    pub position_secs: f64,
    #[serde(default)]
    pub is_muted: bool,
    #[serde(default = "default_session_volume")]
    pub volume: f64,
    #[serde(default = "default_true")]
    pub is_shuffle: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    pub name: String,
    pub query: String,
    pub path: String,
    pub position_secs: f64,
    #[serde(default = "default_true")]
    pub is_shuffle: bool,
}

use crate::keybinds::KeybindSettings;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WazooSettings {
    pub window_bounds: WindowBounds,
    pub window_opacity: f32,
    pub media_folders: Vec<String>,
    pub player_count: usize,
    pub layout: LayoutMode,
    pub playback_mode: PlaybackMode,
    pub scroll_speed: f32,
    #[serde(default = "default_scroll_mode_muted")]
    pub scroll_mode_muted: bool,
    pub last_query: String,
    #[serde(default)]
    pub last_folders: Vec<String>,
    #[serde(default = "default_buffer_duration_secs")]
    pub buffer_duration_secs: u32,
    #[serde(default = "default_buffer_size_mb")]
    pub buffer_size_mb: u32,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_flip_interval_secs")]
    pub flip_interval_secs: u64,
    #[serde(default)]
    pub session_videos: Vec<VideoSession>,
    #[serde(default)]
    pub bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub preferred_audio_language: Option<String>,
    #[serde(default)]
    pub is_default_player: bool,
    #[serde(default)]
    pub is_always_on_top: bool,
    #[serde(default)]
    pub keybinds: KeybindSettings,
    #[serde(default)]
    pub gamma: f32,
    #[serde(default)]
    pub contrast: f32,
    #[serde(default)]
    pub brightness: f32,
    #[serde(default)]
    pub saturation: f32,
    #[serde(default = "default_playback_speed")]
    pub playback_speed: f32,
    #[serde(default)]
    pub crt_enabled: bool,
    #[serde(default)]
    pub loading_indicator: LoadingIndicator,
}

fn default_playback_speed() -> f32 {
    1.0
}

fn default_scroll_mode_muted() -> bool {
    true
}

impl Default for WazooSettings {
    fn default() -> Self {
        Self {
            window_bounds: WindowBounds::default(),
            window_opacity: 1.0,
            media_folders: Vec::new(),
            player_count: 1,
            layout: LayoutMode::Grid,
            playback_mode: PlaybackMode::Normal,
            scroll_speed: 1.0,
            scroll_mode_muted: true,
            last_query: String::new(),
            last_folders: Vec::new(),
            buffer_duration_secs: 10,
            buffer_size_mb: 32,
            language: "en".to_string(),
            flip_interval_secs: DEFAULT_FLIP_INTERVAL_SECS,
            session_videos: Vec::new(),
            bookmarks: Vec::new(),
            preferred_audio_language: None,
            is_default_player: false,
            is_always_on_top: false,
            keybinds: KeybindSettings::default(),
            gamma: 0.0,
            contrast: 0.0,
            brightness: 0.0,
            saturation: 0.0,
            playback_speed: 1.0,
            crt_enabled: false,
            loading_indicator: LoadingIndicator::TvStatic,
        }
    }
}
