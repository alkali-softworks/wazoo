/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Visibility State
 */

use crate::app::SettingsTab;

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
    pub playback_settings_debounce_ticks: usize,
}

impl ModalState {
    /// Returns true if any modal dialog overlay is currently visible.
    #[inline]
    pub fn is_any_open(&self) -> bool {
        self.search || self.settings || self.help || self.bookmarks || self.menu
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
    }
}
