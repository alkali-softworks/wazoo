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
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.player_overlay_fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseMenuModal => {
                self.show_menu_modal = false;
                Task::none()
            }
            Message::OpenSearchModal => {
                self.show_search_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.show_settings_modal = false;
                self.show_help_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.player_overlay_fade_in_ticks = 0;
                self.search_tags = self
                    .active_search_query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.selected_search_folders = self.active_search_folders.clone();
                self.selected_search_folder = self.active_search_folder.clone();
                self.search_input.clear();
                Task::batch([iced::widget::operation::focus("search_input")])
            }
            Message::CloseSearchModal => {
                self.show_search_modal = false;
                Task::none()
            }
            Message::OpenSettingsModal => {
                self.show_settings_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.show_search_modal = false;
                self.show_help_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.player_overlay_fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseSettingsModal => {
                self.show_settings_modal = false;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetSettingsTab(tab) => {
                self.settings_tab = tab;
                Task::none()
            }
            Message::SetLanguage(lang) => {
                let was_all_active = self.active_search_folders.is_empty()
                    || self.is_all_folder(&self.active_search_folder);
                let was_all_selected = self.selected_search_folders.is_empty()
                    || self.is_all_folder(&self.selected_search_folder);
                self.settings.language = lang;
                if was_all_active {
                    self.active_search_folder = self.t("common.all");
                }
                if was_all_selected {
                    self.selected_search_folder = self.t("common.all");
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
                if self.flip_countdown > clamped {
                    self.flip_countdown = clamped;
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
                self.show_help_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.show_search_modal = false;
                self.show_settings_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.player_overlay_fade_in_ticks = 0;
                Task::none()
            }
            Message::CloseHelpModal => {
                self.show_help_modal = false;
                Task::none()
            }
            Message::ToggleBookmarksModal => {
                self.show_bookmarks_modal = !self.show_bookmarks_modal;
                if self.show_bookmarks_modal {
                    self.show_dropdown_menu = false;
                    self.show_menu_modal = false;
                    self.show_search_modal = false;
                    self.show_settings_modal = false;
                    self.show_help_modal = false;
                    self.show_titlebar = false;
                    self.toast_message = None;
                    self.hovered_player_id = None;
                    self.player_overlay_ticks = 0;
                    self.player_overlay_fade_in_ticks = 0;
                }
                Task::none()
            }
            Message::CloseBookmarksModal => {
                self.show_bookmarks_modal = false;
                Task::none()
            }
            Message::EscapePressed => {
                if self.show_transcript_menu {
                    self.show_transcript_menu = false;
                    return Task::none();
                }
                if self.open_audio_menu_player_id.is_some() {
                    self.open_audio_menu_player_id = None;
                    return Task::none();
                }
                if self.is_any_modal_open()
                    || self.show_dropdown_menu
                    || self.show_file_picker
                    || self.show_transcript
                    || self.show_history_drawer
                {
                    self.show_help_modal = false;
                    self.show_search_modal = false;
                    self.show_settings_modal = false;
                    self.show_menu_modal = false;
                    self.show_bookmarks_modal = false;
                    self.show_dropdown_menu = false;
                    self.close_file_picker();
                    self.show_transcript = false;
                    self.show_transcript_menu = false;
                    self.show_history_drawer = false;
                } else {
                    self.show_menu_modal = true;
                    self.show_dropdown_menu = false;
                }
                Task::none()
            }
            Message::DismissToast => {
                self.toast_message = None;
                self.toast_time_remaining = 0;
                Task::none()
            }
            Message::ModalCardClicked => Task::none(),
            _ => Task::none(),
        }
    }
}
