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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MediaMetadata {
    pub video_id: i64,
    pub plex_rating_key: Option<String>,
    pub title: String,
    pub series_title: Option<String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub year: Option<u32>,
    pub tagline: Option<String>,
    pub summary: Option<String>,
    pub content_rating: Option<String>,
    pub duration_ms: Option<u64>,
    pub rating: Option<f32>,
    pub studio: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub directors: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    pub thumb_url: Option<String>,
    pub art_url: Option<String>,
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlexSettings {
    pub enabled: bool,
    pub server_url: String,
    pub token: String,
    pub auto_sync_on_scan: bool,
    pub last_synced_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlexExplorerTab {
    All,
    RecentlyAdded,
    OnDeck,
    TopRated,
    Decade(u32),
    Genre(String),
    Random,
}

impl Default for PlexExplorerTab {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlexExplorerItem {
    pub video_id: i64,
    pub path: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub year: Option<u32>,
    pub content_rating: Option<String>,
    pub rating: Option<f32>,
    #[serde(default)]
    pub genres: Vec<String>,
    pub duration_ms: Option<u64>,
    pub view_offset_ms: Option<u64>,
    pub tagline: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub directors: Vec<String>,
    #[serde(default)]
    pub actors: Vec<String>,
}

fn default_plex_server_url() -> String {
    "http://192.168.1.100:32400".to_string()
}

impl Default for PlexSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: default_plex_server_url(),
            token: String::new(),
            auto_sync_on_scan: false,
            last_synced_at: None,
        }
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
    Spinner,
    TvStatic,
}

impl LoadingIndicator {
    pub const ALL: &'static [LoadingIndicator] =
        &[LoadingIndicator::Spinner, LoadingIndicator::TvStatic];
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
    #[serde(default = "default_true")]
    pub search_include_metadata: bool,
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
    pub preferred_subtitle_language: Option<String>,
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
    pub filters_enabled: bool,
    #[serde(default = "default_filter_crt")]
    pub filter_crt: bool,
    #[serde(default)]
    pub filter_wavy: bool,
    #[serde(default)]
    pub filter_fog: bool,
    #[serde(default)]
    pub loading_indicator: LoadingIndicator,
    #[serde(default = "default_cube_speed")]
    pub cube_speed: f32,
    #[serde(default = "default_cube_size")]
    pub cube_size: f32,
    #[serde(default)]
    pub cube_sheen: bool,
    #[serde(default)]
    pub use_standby_player: bool,
    #[serde(default)]
    pub crt: CrtSettings,
    #[serde(default)]
    pub plex: PlexSettings,
    #[serde(default)]
    pub theme: ThemeSettings,
}

pub const DEFAULT_THEME_ACCENT_COLOR: &str = "#42b883";
pub const DEFAULT_THEME_SURFACE_COLOR: &str = "#1f1f1f";
pub const DEFAULT_THEME_TITLEBAR_COLOR: &str = "#1f092f";
pub const DEFAULT_THEME_TITLEBAR_BADGE_COLOR: &str = "#000000";
pub const DEFAULT_THEME_HUD_COLOR: &str = "#14141a";
pub const DEFAULT_THEME_TEXT_COLOR: &str = "#ffffff";
pub const DEFAULT_THEME_HUD_OPACITY: f32 = 0.88;
pub const DEFAULT_THEME_CORNER_RADIUS: f32 = 1.0;
pub const DEFAULT_THEME_FOCUS_RING_WIDTH: f32 = 3.0;

fn default_theme_accent_color() -> String {
    DEFAULT_THEME_ACCENT_COLOR.to_string()
}

fn default_theme_surface_color() -> String {
    DEFAULT_THEME_SURFACE_COLOR.to_string()
}

fn default_theme_titlebar_color() -> String {
    DEFAULT_THEME_TITLEBAR_COLOR.to_string()
}

fn default_theme_titlebar_badge_color() -> String {
    DEFAULT_THEME_TITLEBAR_BADGE_COLOR.to_string()
}

fn default_theme_hud_color() -> String {
    DEFAULT_THEME_HUD_COLOR.to_string()
}

fn default_theme_text_color() -> String {
    DEFAULT_THEME_TEXT_COLOR.to_string()
}

fn default_theme_hud_opacity() -> f32 {
    DEFAULT_THEME_HUD_OPACITY
}

fn default_theme_corner_radius() -> f32 {
    DEFAULT_THEME_CORNER_RADIUS
}

fn default_theme_focus_ring_width() -> f32 {
    DEFAULT_THEME_FOCUS_RING_WIDTH
}

/// Parses a `#RRGGBB`, `RRGGBB`, `#RGB`, or `RGB` hex color string into `(r, g, b)`.
pub fn parse_hex_rgb(input: &str) -> Option<(u8, u8, u8)> {
    let hex = input.trim().strip_prefix('#').unwrap_or(input.trim());
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

/// Formats an `(r, g, b)` tuple as canonical lowercase `#rrggbb`.
pub fn format_hex_rgb(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Normalizes a hex color string to `#rrggbb`, or falls back to `default`.
pub fn sanitize_hex_or_default(input: &str, default: &str) -> String {
    match parse_hex_rgb(input) {
        Some((r, g, b)) => format_hex_rgb(r, g, b),
        None => default.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeSettings {
    #[serde(default = "default_theme_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_theme_surface_color")]
    pub surface_color: String,
    #[serde(default = "default_theme_titlebar_color")]
    pub titlebar_color: String,
    #[serde(default = "default_theme_titlebar_badge_color")]
    pub titlebar_badge_color: String,
    #[serde(default = "default_theme_hud_color")]
    pub hud_color: String,
    #[serde(default = "default_theme_text_color")]
    pub text_color: String,
    #[serde(default = "default_theme_hud_opacity")]
    pub hud_opacity: f32,
    #[serde(default = "default_theme_corner_radius")]
    pub corner_radius: f32,
    #[serde(default = "default_theme_focus_ring_width")]
    pub focus_ring_width: f32,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            accent_color: default_theme_accent_color(),
            surface_color: default_theme_surface_color(),
            titlebar_color: default_theme_titlebar_color(),
            titlebar_badge_color: default_theme_titlebar_badge_color(),
            hud_color: default_theme_hud_color(),
            text_color: default_theme_text_color(),
            hud_opacity: DEFAULT_THEME_HUD_OPACITY,
            corner_radius: DEFAULT_THEME_CORNER_RADIUS,
            focus_ring_width: DEFAULT_THEME_FOCUS_RING_WIDTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    Default,
    Midnight,
    Dracula,
    Crimson,
    Amber,
    Nord,
    Synthwave,
    FrostLight,
}

impl ThemePreset {
    pub const ALL: &'static [ThemePreset] = &[
        ThemePreset::Default,
        ThemePreset::Midnight,
        ThemePreset::Dracula,
        ThemePreset::Crimson,
        ThemePreset::Amber,
        ThemePreset::Nord,
        ThemePreset::Synthwave,
        ThemePreset::FrostLight,
    ];
}

impl ThemeSettings {
    pub fn clamp_all(&mut self) {
        self.accent_color = sanitize_hex_or_default(&self.accent_color, DEFAULT_THEME_ACCENT_COLOR);
        self.surface_color =
            sanitize_hex_or_default(&self.surface_color, DEFAULT_THEME_SURFACE_COLOR);
        self.titlebar_color =
            sanitize_hex_or_default(&self.titlebar_color, DEFAULT_THEME_TITLEBAR_COLOR);
        self.titlebar_badge_color = sanitize_hex_or_default(
            &self.titlebar_badge_color,
            DEFAULT_THEME_TITLEBAR_BADGE_COLOR,
        );
        self.hud_color = sanitize_hex_or_default(&self.hud_color, DEFAULT_THEME_HUD_COLOR);
        self.text_color = sanitize_hex_or_default(&self.text_color, DEFAULT_THEME_TEXT_COLOR);
        self.hud_opacity = if self.hud_opacity.is_finite() {
            self.hud_opacity.clamp(0.25, 1.0)
        } else {
            DEFAULT_THEME_HUD_OPACITY
        };
        self.corner_radius = if self.corner_radius.is_finite() {
            self.corner_radius.clamp(0.0, 2.0)
        } else {
            DEFAULT_THEME_CORNER_RADIUS
        };
        self.focus_ring_width = if self.focus_ring_width.is_finite() {
            self.focus_ring_width.clamp(0.0, 8.0)
        } else {
            DEFAULT_THEME_FOCUS_RING_WIDTH
        };
    }

    pub fn apply_preset(&mut self, preset: ThemePreset) {
        match preset {
            ThemePreset::Default => {
                *self = Self::default();
            }
            ThemePreset::Midnight => {
                self.accent_color = "#38bdf8".to_string();
                self.surface_color = "#0f172a".to_string();
                self.titlebar_color = "#0b1120".to_string();
                self.titlebar_badge_color = "#020617".to_string();
                self.hud_color = "#0b1222".to_string();
                self.text_color = "#f1f5f9".to_string();
                self.hud_opacity = 0.88;
                self.corner_radius = 1.0;
                self.focus_ring_width = 3.0;
            }
            ThemePreset::Dracula => {
                self.accent_color = "#bd93f9".to_string();
                self.surface_color = "#282a36".to_string();
                self.titlebar_color = "#1e1f29".to_string();
                self.titlebar_badge_color = "#191a21".to_string();
                self.hud_color = "#21222c".to_string();
                self.text_color = "#f8f8f2".to_string();
                self.hud_opacity = 0.90;
                self.corner_radius = 1.0;
                self.focus_ring_width = 3.0;
            }
            ThemePreset::Crimson => {
                self.accent_color = "#f43f5e".to_string();
                self.surface_color = "#18181b".to_string();
                self.titlebar_color = "#2a0813".to_string();
                self.titlebar_badge_color = "#09090b".to_string();
                self.hud_color = "#141418".to_string();
                self.text_color = "#fafafa".to_string();
                self.hud_opacity = 0.88;
                self.corner_radius = 1.0;
                self.focus_ring_width = 3.0;
            }
            ThemePreset::Amber => {
                self.accent_color = "#f59e0b".to_string();
                self.surface_color = "#1c1917".to_string();
                self.titlebar_color = "#271a0c".to_string();
                self.titlebar_badge_color = "#0c0a09".to_string();
                self.hud_color = "#161210".to_string();
                self.text_color = "#fef3c7".to_string();
                self.hud_opacity = 0.88;
                self.corner_radius = 0.75;
                self.focus_ring_width = 3.0;
            }
            ThemePreset::Nord => {
                self.accent_color = "#88c0d0".to_string();
                self.surface_color = "#2e3440".to_string();
                self.titlebar_color = "#242933".to_string();
                self.titlebar_badge_color = "#1e222a".to_string();
                self.hud_color = "#242933".to_string();
                self.text_color = "#eceff4".to_string();
                self.hud_opacity = 0.90;
                self.corner_radius = 1.0;
                self.focus_ring_width = 3.0;
            }
            ThemePreset::Synthwave => {
                self.accent_color = "#ff2e97".to_string();
                self.surface_color = "#1a1025".to_string();
                self.titlebar_color = "#2b0f4a".to_string();
                self.titlebar_badge_color = "#0d0514".to_string();
                self.hud_color = "#160b24".to_string();
                self.text_color = "#fdf4ff".to_string();
                self.hud_opacity = 0.88;
                self.corner_radius = 1.25;
                self.focus_ring_width = 3.5;
            }
            ThemePreset::FrostLight => {
                self.accent_color = "#10b981".to_string();
                self.surface_color = "#e5e7eb".to_string();
                self.titlebar_color = "#312e81".to_string();
                self.titlebar_badge_color = "#1e1b4b".to_string();
                self.hud_color = "#f3f4f6".to_string();
                self.text_color = "#111827".to_string();
                self.hud_opacity = 0.90;
                self.corner_radius = 1.0;
                self.focus_ring_width = 3.0;
            }
        }
    }

    pub fn matches_preset(&self, preset: ThemePreset) -> bool {
        let mut target = Self::default();
        target.apply_preset(preset);
        self.accent_color.eq_ignore_ascii_case(&target.accent_color)
            && self.surface_color.eq_ignore_ascii_case(&target.surface_color)
            && self.titlebar_color.eq_ignore_ascii_case(&target.titlebar_color)
            && self
                .titlebar_badge_color
                .eq_ignore_ascii_case(&target.titlebar_badge_color)
            && self.hud_color.eq_ignore_ascii_case(&target.hud_color)
            && self.text_color.eq_ignore_ascii_case(&target.text_color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrtSettings {
    #[serde(default = "default_crt_scanline_darkness")]
    pub scanline_darkness: f32,
    #[serde(default = "default_crt_scanline_density")]
    pub scanline_density: f32,
    #[serde(default = "default_crt_scanline_speed")]
    pub scanline_speed: f32,
    #[serde(default = "default_crt_curvature")]
    pub curvature: f32,
    #[serde(default = "default_crt_gamma")]
    pub gamma: f32,
    #[serde(default = "default_crt_saturation")]
    pub saturation: f32,
    #[serde(default = "default_crt_brightness")]
    pub brightness: f32,
    #[serde(default = "default_crt_vignette")]
    pub vignette: f32,
    #[serde(default = "default_crt_chromatic_aberration")]
    pub chromatic_aberration: f32,
    #[serde(default = "default_crt_noise")]
    pub noise: f32,
    #[serde(default = "default_crt_corner_rounding")]
    pub corner_rounding: f32,
}

pub fn default_crt_scanline_darkness() -> f32 {
    0.20
}

pub fn default_crt_scanline_density() -> f32 {
    600.0
}

pub fn default_crt_scanline_speed() -> f32 {
    5.0
}

pub fn default_crt_curvature() -> f32 {
    0.35
}

pub fn default_crt_gamma() -> f32 {
    1.00
}

pub fn default_crt_saturation() -> f32 {
    1.00
}

pub fn default_crt_brightness() -> f32 {
    1.00
}

pub fn default_crt_vignette() -> f32 {
    0.40
}

pub fn default_crt_chromatic_aberration() -> f32 {
    0.20
}

pub fn default_crt_noise() -> f32 {
    0.25
}

pub fn default_crt_corner_rounding() -> f32 {
    0.50
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            scanline_darkness: default_crt_scanline_darkness(),
            scanline_density: default_crt_scanline_density(),
            scanline_speed: default_crt_scanline_speed(),
            curvature: default_crt_curvature(),
            gamma: default_crt_gamma(),
            saturation: default_crt_saturation(),
            brightness: default_crt_brightness(),
            vignette: default_crt_vignette(),
            chromatic_aberration: default_crt_chromatic_aberration(),
            noise: default_crt_noise(),
            corner_rounding: default_crt_corner_rounding(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrtPreset {
    Default,
    Subtle,
    Arcade,
    WarmPhosphor,
    Monochrome,
    Vhs,
}

impl CrtSettings {
    pub fn clamp_all(&mut self) {
        self.scanline_darkness = self.scanline_darkness.clamp(0.0, 1.0);
        self.scanline_density = self.scanline_density.clamp(50.0, 2000.0);
        self.scanline_speed = self.scanline_speed.clamp(0.0, 20.0);
        self.curvature = self.curvature.clamp(0.0, 1.0);
        self.gamma = self.gamma.clamp(0.2, 3.0);
        self.saturation = self.saturation.clamp(0.0, 3.0);
        self.brightness = self.brightness.clamp(0.2, 3.0);
        self.vignette = self.vignette.clamp(0.0, 1.0);
        self.chromatic_aberration = self.chromatic_aberration.clamp(0.0, 1.0);
        self.noise = self.noise.clamp(0.0, 1.0);
        self.corner_rounding = self.corner_rounding.clamp(0.0, 1.0);
    }

    pub fn apply_preset(&mut self, preset: CrtPreset) {
        match preset {
            CrtPreset::Default => {
                *self = Self::default();
            }
            CrtPreset::Subtle => {
                self.scanline_darkness = 0.08;
                self.scanline_density = 720.0;
                self.scanline_speed = 3.0;
                self.curvature = 0.10;
                self.gamma = 1.05;
                self.saturation = 1.05;
                self.brightness = 1.00;
                self.vignette = 0.15;
                self.chromatic_aberration = 0.05;
                self.noise = 0.08;
                self.corner_rounding = 0.20;
            }
            CrtPreset::Arcade => {
                self.scanline_darkness = 0.35;
                self.scanline_density = 480.0;
                self.scanline_speed = 6.0;
                self.curvature = 0.40;
                self.gamma = 1.15;
                self.saturation = 1.30;
                self.brightness = 1.10;
                self.vignette = 0.50;
                self.chromatic_aberration = 0.25;
                self.noise = 0.20;
                self.corner_rounding = 0.55;
            }
            CrtPreset::WarmPhosphor => {
                self.scanline_darkness = 0.15;
                self.scanline_density = 600.0;
                self.scanline_speed = 4.0;
                self.curvature = 0.30;
                self.gamma = 1.25;
                self.saturation = 1.20;
                self.brightness = 1.15;
                self.vignette = 0.35;
                self.chromatic_aberration = 0.15;
                self.noise = 0.15;
                self.corner_rounding = 0.45;
            }
            CrtPreset::Monochrome => {
                self.scanline_darkness = 0.25;
                self.scanline_density = 520.0;
                self.scanline_speed = 4.0;
                self.curvature = 0.45;
                self.gamma = 1.10;
                self.saturation = 0.00;
                self.brightness = 1.05;
                self.vignette = 0.55;
                self.chromatic_aberration = 0.00;
                self.noise = 0.35;
                self.corner_rounding = 0.60;
            }
            CrtPreset::Vhs => {
                self.scanline_darkness = 0.22;
                self.scanline_density = 400.0;
                self.scanline_speed = 7.0;
                self.curvature = 0.25;
                self.gamma = 1.15;
                self.saturation = 1.10;
                self.brightness = 1.05;
                self.vignette = 0.40;
                self.chromatic_aberration = 0.60;
                self.noise = 0.55;
                self.corner_rounding = 0.40;
            }
        }
    }
}

fn default_playback_speed() -> f32 {
    1.0
}

fn default_scroll_mode_muted() -> bool {
    true
}

fn default_filter_crt() -> bool {
    true
}

fn default_cube_speed() -> f32 {
    1.0
}

fn default_cube_size() -> f32 {
    1.0
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
            search_include_metadata: true,
            buffer_duration_secs: 10,
            buffer_size_mb: 32,
            language: "en".to_string(),
            flip_interval_secs: DEFAULT_FLIP_INTERVAL_SECS,
            session_videos: Vec::new(),
            bookmarks: Vec::new(),
            preferred_audio_language: None,
            preferred_subtitle_language: None,
            is_default_player: false,
            is_always_on_top: false,
            keybinds: KeybindSettings::default(),
            gamma: 0.0,
            contrast: 0.0,
            brightness: 0.0,
            saturation: 0.0,
            playback_speed: 1.0,
            crt_enabled: false,
            filters_enabled: false,
            filter_crt: true,
            filter_wavy: false,
            filter_fog: false,
            loading_indicator: LoadingIndicator::Spinner,
            cube_speed: 1.0,
            cube_size: 1.0,
            cube_sheen: false,
            use_standby_player: false,
            crt: CrtSettings::default(),
            plex: PlexSettings::default(),
            theme: ThemeSettings::default(),
        }
    }
}

impl WazooSettings {
    /// Returns the subtitle language preference if explicitly set;
    /// otherwise falls back to the "language" i18n setting.
    pub fn effective_subtitle_language(&self) -> &str {
        self.preferred_subtitle_language
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.language)
    }
}
