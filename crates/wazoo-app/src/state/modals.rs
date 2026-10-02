/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Visibility State
 */

use crate::app::SettingsTab;

#[derive(Debug, Clone, Default)]
pub struct ModalState {
    pub search: bool,
    pub settings: bool,
    pub settings_tab: SettingsTab,
    pub help: bool,
    pub menu: bool,
    pub bookmarks: bool,
    pub playback_settings_debounce_ticks: usize,
}

impl ModalState {
    pub fn is_any_open(&self) -> bool {
        self.search || self.settings || self.help || self.bookmarks || self.menu
    }

    pub fn close_all(&mut self) {
        self.search = false;
        self.settings = false;
        self.help = false;
        self.bookmarks = false;
        self.menu = false;
    }
}
