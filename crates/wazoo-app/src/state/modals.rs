/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Visibility State
 */

use crate::app::SettingsTab;
use crate::color_picker::rgb_to_hsv;
use wazoo_core::{
    DEFAULT_THEME_ACCENT_COLOR, DEFAULT_THEME_HUD_COLOR, DEFAULT_THEME_SURFACE_COLOR,
    DEFAULT_THEME_TEXT_COLOR, DEFAULT_THEME_TITLEBAR_BADGE_COLOR, DEFAULT_THEME_TITLEBAR_COLOR,
    PlexExplorerTab, ThemeSettings, parse_hex_rgb,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeColorTarget {
    #[default]
    Accent,
    Surface,
    Titlebar,
    TitlebarBadge,
    Hud,
    Text,
}

impl ThemeColorTarget {
    pub const ALL: &'static [ThemeColorTarget] = &[
        ThemeColorTarget::Accent,
        ThemeColorTarget::Surface,
        ThemeColorTarget::Titlebar,
        ThemeColorTarget::TitlebarBadge,
        ThemeColorTarget::Hud,
        ThemeColorTarget::Text,
    ];

    pub fn get_hex<'a>(self, theme: &'a ThemeSettings) -> &'a str {
        match self {
            Self::Accent => &theme.accent_color,
            Self::Surface => &theme.surface_color,
            Self::Titlebar => &theme.titlebar_color,
            Self::TitlebarBadge => &theme.titlebar_badge_color,
            Self::Hud => &theme.hud_color,
            Self::Text => &theme.text_color,
        }
    }

    pub fn set_hex(self, theme: &mut ThemeSettings, hex: String) {
        match self {
            Self::Accent => theme.accent_color = hex,
            Self::Surface => theme.surface_color = hex,
            Self::Titlebar => theme.titlebar_color = hex,
            Self::TitlebarBadge => theme.titlebar_badge_color = hex,
            Self::Hud => theme.hud_color = hex,
            Self::Text => theme.text_color = hex,
        }
    }

    pub fn default_hex(self) -> &'static str {
        match self {
            Self::Accent => DEFAULT_THEME_ACCENT_COLOR,
            Self::Surface => DEFAULT_THEME_SURFACE_COLOR,
            Self::Titlebar => DEFAULT_THEME_TITLEBAR_COLOR,
            Self::TitlebarBadge => DEFAULT_THEME_TITLEBAR_BADGE_COLOR,
            Self::Hud => DEFAULT_THEME_HUD_COLOR,
            Self::Text => DEFAULT_THEME_TEXT_COLOR,
        }
    }
}

/// Tracks the visibility state of centered modal dialog overlays.
#[derive(Debug, Clone)]
pub struct ModalState {
    pub search: bool,
    pub settings: bool,
    pub settings_tab: SettingsTab,
    pub help: bool,
    pub help_search: String,
    pub menu: bool,
    pub bookmarks: bool,
    pub quick_picker: bool,
    pub quick_picker_search: String,
    pub plex_explorer: bool,
    pub plex_explorer_search: String,
    pub plex_explorer_tab: PlexExplorerTab,
    pub plex_explorer_genre: Option<String>,
    pub plex_explorer_decade: Option<u32>,
    pub plex_explorer_random_seed: u64,
    pub playback_settings_debounce_ticks: usize,
    pub active_color_picker: Option<ThemeColorTarget>,
    pub theme_picker_hsv: (f32, f32, f32),
    pub theme_hex_drafts: [String; 6],
}

impl Default for ModalState {
    fn default() -> Self {
        let mut s = Self {
            search: false,
            settings: false,
            settings_tab: SettingsTab::default(),
            help: false,
            help_search: String::new(),
            menu: false,
            bookmarks: false,
            quick_picker: false,
            quick_picker_search: String::new(),
            plex_explorer: false,
            plex_explorer_search: String::new(),
            plex_explorer_tab: PlexExplorerTab::default(),
            plex_explorer_genre: None,
            plex_explorer_decade: None,
            plex_explorer_random_seed: 0,
            playback_settings_debounce_ticks: 0,
            active_color_picker: None,
            theme_picker_hsv: (153.0, 0.64, 0.72),
            theme_hex_drafts: Default::default(),
        };
        s.sync_picker_from_theme(&ThemeSettings::default());
        s
    }
}

impl ModalState {
    pub fn target_index(target: ThemeColorTarget) -> usize {
        match target {
            ThemeColorTarget::Accent => 0,
            ThemeColorTarget::Surface => 1,
            ThemeColorTarget::Titlebar => 2,
            ThemeColorTarget::TitlebarBadge => 3,
            ThemeColorTarget::Hud => 4,
            ThemeColorTarget::Text => 5,
        }
    }

    pub fn sync_picker_from_theme(&mut self, theme: &ThemeSettings) {
        for &t in ThemeColorTarget::ALL {
            self.theme_hex_drafts[Self::target_index(t)] = t.get_hex(theme).to_string();
        }
        if let Some(active) = self.active_color_picker {
            let hex = active.get_hex(theme);
            if let Some((r, g, b)) = parse_hex_rgb(hex) {
                self.theme_picker_hsv = rgb_to_hsv(r, g, b);
            }
        }
    }

    /// Returns true if any modal dialog overlay is currently visible.
    #[inline]
    pub fn is_any_open(&self) -> bool {
        self.search
            || self.settings
            || self.help
            || self.bookmarks
            || self.menu
            || self.quick_picker
            || self.plex_explorer
    }

    /// Closes all active modal dialog overlays.
    #[inline]
    pub fn close_all(&mut self) {
        self.search = false;
        self.settings = false;
        self.help = false;
        self.help_search.clear();
        self.bookmarks = false;
        self.menu = false;
        self.quick_picker = false;
        self.quick_picker_search.clear();
        self.plex_explorer = false;
        self.plex_explorer_search.clear();
        self.plex_explorer_genre = None;
        self.plex_explorer_decade = None;
        self.plex_explorer_random_seed = 0;
        self.active_color_picker = None;
    }
}
