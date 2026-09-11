use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub has_subtitles: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    Column,
    Row,
    Grid,
}

impl Default for LayoutMode {
    fn default() -> Self {
        Self::Grid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackMode {
    Normal,
    Scroll,
    Flip,
}

impl Default for PlaybackMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WazooSettings {
    pub window_bounds: WindowBounds,
    pub window_opacity: f32,
    pub media_folders: Vec<String>,
    pub player_count: usize,
    pub layout: LayoutMode,
    pub playback_mode: PlaybackMode,
    pub scroll_speed: f32,
    pub is_global_muted: bool,
    pub last_query: String,
    pub last_folder: String,
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
            is_global_muted: true,
            last_query: String::new(),
            last_folder: "All".to_string(),
        }
    }
}
