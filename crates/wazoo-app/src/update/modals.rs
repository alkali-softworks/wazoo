/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modals & Settings Dialogs Reducer
 *
 * Handles lifecycle, tab selection, and setting updates for Search, Settings, Help, Bookmarks,
 * and Menu modals.
 */

use crate::app::WazooApp;
use crate::message::Message;
use iced::Task;

impl WazooApp {
    pub(crate) fn update_modals(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenMenuModal => {
                self.modals.close_all();
                self.modals.menu = true;
                self.titlebar.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseMenuModal => {
                self.modals.menu = false;
                Task::none()
            }
            Message::OpenSearchModal => {
                self.modals.close_all();
                self.modals.search = true;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                self.search.tags = self
                    .search
                    .active_query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.search.selected_folders = self.search.active_folders.clone();
                self.search.selected_folder = self.search.active_folder.clone();
                self.search.input.clear();
                Task::batch([iced::widget::operation::focus("search_input")])
            }
            Message::CloseSearchModal => {
                self.modals.search = false;
                Task::none()
            }
            Message::OpenSettingsModal => {
                self.modals.close_all();
                self.modals.settings = true;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseSettingsModal => {
                if self.modals.playback_settings_debounce_ticks > 0 {
                    self.apply_playback_settings();
                }
                self.modals.settings = false;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetSettingsTab(tab) => {
                if self.modals.playback_settings_debounce_ticks > 0 {
                    self.apply_playback_settings();
                }
                self.modals.settings_tab = tab;
                Task::none()
            }
            Message::SetLanguage(lang) => {
                let was_all_active = self.search.active_folders.is_empty()
                    || self.is_all_folder(&self.search.active_folder);
                let was_all_selected = self.search.selected_folders.is_empty()
                    || self.is_all_folder(&self.search.selected_folder);
                self.settings.language = lang;
                if was_all_active {
                    self.search.active_folder = self.t("common.all");
                }
                if was_all_selected {
                    self.search.selected_folder = self.t("common.all");
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                for p in &mut self.players {
                    p.set_i18n_language(Some(self.settings.language.clone()));
                }
                Task::none()
            }
            Message::SetGamma(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.gamma = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetContrast(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.contrast = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetBrightness(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.brightness = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetSaturation(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.saturation = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetPlaybackSpeed(val) => {
                let clamped = val.clamp(0.25, 3.0);
                self.settings.playback_speed = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::ToggleCrtFilter => {
                let new_state = !self.settings.crt_enabled;
                self.settings.crt_enabled = new_state;
                for player in &mut self.players {
                    player.set_crt_enabled(new_state);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtFilter(enabled) => {
                self.settings.crt_enabled = enabled;
                for player in &mut self.players {
                    player.set_crt_enabled(enabled);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetLoadingIndicator(indicator) => {
                self.settings.loading_indicator = indicator;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetBufferDuration(secs) => {
                self.settings.buffer_duration_secs = secs.clamp(2, 60);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetBufferSize(mb) => {
                self.settings.buffer_size_mb = mb.clamp(8, 1024);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetFlipInterval(secs) => {
                let clamped = secs.clamp(1, 3600);
                self.settings.flip_interval_secs = clamped;
                self.stagger_flip_countdowns();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ResetPlaybackOptions => {
                self.modals.playback_settings_debounce_ticks = 0;
                self.settings.gamma = 0.0;
                self.settings.contrast = 0.0;
                self.settings.brightness = 0.0;
                self.settings.saturation = 0.0;
                self.settings.playback_speed = 1.0;
                self.settings.crt_enabled = false;
                for player in &mut self.players {
                    player.set_equalizer(0.0, 0.0, 0.0, 0.0);
                    player.set_speed(1.0);
                    player.set_crt_enabled(false);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleDefaultPlayer => {
                self.settings.is_default_player = !self.settings.is_default_player;
                let _ = self.config_mgr.save_settings(&self.settings);
                #[cfg(target_os = "linux")]
                {
                    if self.settings.is_default_player {
                        let _ = crate::platform::set_as_default_video_player();
                    } else {
                        let _ = crate::platform::unset_as_default_video_player();
                    }
                }
                Task::none()
            }
            Message::OpenAlkaliWebsite => {
                crate::platform::open_url("https://alkalisoftworks.com/");
                Task::none()
            }
            Message::OpenHelpModal => {
                self.modals.close_all();
                self.modals.help = true;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseHelpModal => {
                self.modals.help = false;
                Task::none()
            }
            Message::ToggleBookmarksModal => {
                self.modals.bookmarks = !self.modals.bookmarks;
                if self.modals.bookmarks {
                    self.titlebar.show_dropdown_menu = false;
                    self.modals.menu = false;
                    self.modals.search = false;
                    self.modals.settings = false;
                    self.modals.help = false;
                    self.titlebar.show = false;
                    self.overlay.clear_toast();
                    self.hovered_player_id = None;
                    self.overlay.ticks = 0;
                    self.overlay.fade_in_ticks = 0;
                }
                Task::none()
            }
            Message::CloseBookmarksModal => {
                self.modals.bookmarks = false;
                Task::none()
            }
            Message::EscapePressed => {
                if self.cube.desktop_overlay {
                    return self.update(Message::ToggleDesktopCubeScreensaver);
                }
                if self.drawers.show_transcript_menu {
                    self.drawers.show_transcript_menu = false;
                    return Task::none();
                }
                if self.open_audio_menu_id.is_some() {
                    self.open_audio_menu_id = None;
                    return Task::none();
                }
                if self.is_any_modal_open()
                    || self.titlebar.show_dropdown_menu
                    || self.drawers.is_any_open()
                {
                    if self.modals.playback_settings_debounce_ticks > 0 {
                        self.apply_playback_settings();
                    }
                    self.modals.close_all();
                    self.titlebar.show_dropdown_menu = false;
                    self.close_file_picker();
                    self.drawers.close_all();
                } else {
                    self.modals.menu = true;
                    self.titlebar.show_dropdown_menu = false;
                }
                Task::none()
            }
            Message::DismissToast => {
                self.overlay.clear_toast();
                Task::none()
            }
            Message::ModalCardClicked => Task::none(),
            _ => Task::none(),
        }
    }

    /// Applies pending debounced equalizer and playback speed settings across all active video players
    /// and commits the settings to persistent storage.
    pub fn apply_playback_settings(&mut self) {
        self.modals.playback_settings_debounce_ticks = 0;
        let gamma = self.settings.gamma as f64;
        let contrast = self.settings.contrast as f64;
        let brightness = self.settings.brightness as f64;
        let saturation = self.settings.saturation as f64;
        let speed = self.settings.playback_speed as f64;

        for player in &mut self.players {
            player.set_equalizer(gamma, contrast, brightness, saturation);
            player.set_speed(speed);
        }
        let _ = self.config_mgr.save_settings(&self.settings);
    }
}
