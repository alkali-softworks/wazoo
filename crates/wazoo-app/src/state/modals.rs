/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Visibility State
 */

use crate::app::SettingsTab;
use wazoo_core::PlexExplorerTab;

/// Tracks the visibility state of centered modal dialog overlays.
#[derive(Debug, Clone, Default)]
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
}

impl ModalState {
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
    }
}
