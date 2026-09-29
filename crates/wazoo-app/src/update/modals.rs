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
                self.modals.settings = false;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetSettingsTab(tab) => {
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
                Task::none()
            }
            Message::SetGamma(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.gamma = clamped;
                for player in &mut self.players {
                    player.set_gamma(clamped as f64);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetContrast(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.contrast = clamped;
                for player in &mut self.players {
                    player.set_contrast(clamped as f64);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetBrightness(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.brightness = clamped;
                for player in &mut self.players {
                    player.set_brightness(clamped as f64);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetSaturation(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.saturation = clamped;
                for player in &mut self.players {
                    player.set_saturation(clamped as f64);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetPlaybackSpeed(val) => {
                let clamped = val.clamp(0.25, 3.0);
                self.settings.playback_speed = clamped;
                for player in &mut self.players {
                    player.set_speed(clamped as f64);
                }
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
                if self.flip.countdown > clamped {
                    self.flip.countdown = clamped;
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ResetPlaybackOptions => {
                self.settings.gamma = 0.0;
                self.settings.contrast = 0.0;
                self.settings.brightness = 0.0;
                self.settings.saturation = 0.0;
                self.settings.playback_speed = 1.0;
                for player in &mut self.players {
                    player.set_gamma(0.0);
                    player.set_contrast(0.0);
                    player.set_brightness(0.0);
                    player.set_saturation(0.0);
                    player.set_speed(1.0);
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
}
