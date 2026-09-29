/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * State Reducer & Event Dispatcher
 *
 * Implements WazooApp::update, processing incoming Message events, executing playback commands,
 * managing modal dialogs, handling window events, and orchestrating asynchronous tasks.
 */

pub mod drawers;
pub mod input;
pub mod modals;
pub mod navigation;
pub mod playback;
pub mod scanner;
pub mod search;
pub mod window;

use crate::app::{
    PLAYER_OVERLAY_FADE_TICKS, PLAYER_OVERLAY_HIDE_TICKS, TITLEBAR_FADE_TICKS,
    TITLEBAR_HIDE_TICKS, TITLEBAR_SHOW_DELAY_TICKS, WazooApp,
};
use crate::message::Message;
use iced::Task;
use std::time::Duration;
use wazoo_core::PlaybackMode;
use wazoo_media::{PlayerId, VideoHandle};

impl WazooApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // =========================================================================
            // Engine Ticks & Frame Updates
            // =========================================================================
            Message::AnimationTick => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    let offscreen = self.scroll_engine.tick();
                    if !offscreen.is_empty() {
                        let (keep, despawned): (Vec<_>, Vec<_>) = self
                            .players
                            .drain(..)
                            .partition(|p| !offscreen.contains(&p.id));
                        self.players = keep;
                        for &id in &offscreen {
                            self.loading.stop(&id);
                        }
                        if !despawned.is_empty() {
                            std::thread::spawn(move || drop(despawned));
                        }
                    }

                    let margin = self.scroll_engine.default_item_height() * 1.5;
                    let mut needs_preload = false;

                    // Non-blocking spawn: attach preloaded player seamlessly if ready
                    while let Some(spawn_y) =
                        self.scroll_engine.needs_new_player_with_margin(margin)
                    {
                        if let Some(mut handle) = self.loading.preloaded_player.take() {
                            let item_h = self.calculate_player_scroll_height(&handle);
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.record_play_history(&handle.state.path);
                            self.players.push(handle);
                            needs_preload = true;
                        } else {
                            // Preloaded player still preparing in background - do NOT block!
                            break;
                        }
                    }

                    for p in &mut self.players {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }

                    if needs_preload || (self.loading.preloaded_player.is_none() && !self.loading.is_preloading) {
                        return self.trigger_preload_task();
                    }
                }
                Task::none()
            }

            Message::VideoFrameTick => {
                if !self.window.is_focused {
                    self.window.unfocused_frame_ticks = self.window.unfocused_frame_ticks.wrapping_add(1);
                    // When the window is behind another window or unfocused, throttle frame updates
                    // to ~30 FPS (every 2nd tick) so background playback remains smooth (movie standard)
                    // without hammering GPU presentation swapchains.
                    if !self.window.unfocused_frame_ticks.is_multiple_of(2) {
                        return Task::none();
                    }
                }
                self.loading.spinner_ticks = self.loading.spinner_ticks.wrapping_add(1);
                if self.playback.open_audio_menu_id.is_some() {
                    self.overlay.ticks = PLAYER_OVERLAY_HIDE_TICKS;
                    self.overlay.fade_in_ticks = PLAYER_OVERLAY_FADE_TICKS;
                } else if self.overlay.ticks > 0 {
                    if self.overlay.fade_in_ticks < PLAYER_OVERLAY_FADE_TICKS {
                        self.overlay.fade_in_ticks += 1;
                    }
                    self.overlay.ticks -= 1;
                    if self.overlay.ticks == 0 {
                        self.playback.hovered_id = None;
                        self.overlay.fade_in_ticks = 0;
                    }
                } else {
                    self.overlay.fade_in_ticks = 0;
                }
                if self.overlay.title_pill_ticks > 0 {
                    self.overlay.title_pill_ticks -= 1;
                }
                if self.overlay.focus_border_ticks > 0 {
                    self.overlay.focus_border_ticks -= 1;
                }
                if self.drawers.file_picker_debounce_ticks > 0 {
                    self.drawers.file_picker_debounce_ticks -= 1;
                    if self.drawers.file_picker_debounce_ticks == 0 {
                        self.apply_file_picker_search();
                    }
                }
                if self.window.is_dragging
                    || self.titlebar.drag_pending
                    || self.is_point_in_titlebar(self.window.cursor_position)
                    || self.titlebar.show_dropdown_menu
                {
                    if self.window.is_dragging || self.titlebar.drag_pending {
                        let is_still_moving = self
                            .window
                            .last_drag_move
                            .map(|t| t.elapsed() < Duration::from_millis(300))
                            .unwrap_or(false);
                        if !is_still_moving
                            && !self.titlebar.drag_pending
                            && !self.is_point_in_titlebar(self.window.cursor_position)
                        {
                            self.window.is_dragging = false;
                            self.window.last_drag_move = None;
                            if self.titlebar.hide_ticks > TITLEBAR_FADE_TICKS {
                                self.titlebar.hide_ticks = TITLEBAR_FADE_TICKS;
                            }
                        } else {
                            self.titlebar.show = true;
                            self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                            self.titlebar.hover_ticks = 0;
                            self.titlebar.slide_ticks = crate::app::TITLEBAR_SLIDE_TICKS;
                        }
                    } else if self.titlebar.show_dropdown_menu {
                        self.titlebar.show = true;
                        self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                        self.titlebar.hover_ticks = 0;
                        self.titlebar.slide_ticks = crate::app::TITLEBAR_SLIDE_TICKS;
                        if self.titlebar.dropdown_menu_slide_ticks < crate::app::DROPDOWN_MENU_SLIDE_TICKS {
                            self.titlebar.dropdown_menu_slide_ticks += 1;
                        }
                    } else if self.titlebar.show {
                        self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                        if self.titlebar.slide_ticks < crate::app::TITLEBAR_SLIDE_TICKS {
                            self.titlebar.slide_ticks += 1;
                        }
                    } else {
                        self.titlebar.hover_ticks += 1;
                        if self.titlebar.hover_ticks >= TITLEBAR_SHOW_DELAY_TICKS {
                            self.titlebar.show = true;
                            self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                            self.titlebar.hover_ticks = 0;
                            self.titlebar.slide_ticks = 0;
                            if self.playback.hovered_id.is_some() {
                                let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                                self.overlay.ticks =
                                    self.overlay.ticks.min(current_fade);
                            }
                        }
                    }
                } else {
                    self.titlebar.hover_ticks = 0;
                    self.titlebar.dropdown_menu_slide_ticks = 0;
                    if self.titlebar.hide_ticks > 0 {
                        self.titlebar.hide_ticks -= 1;
                        if self.titlebar.hide_ticks == 0 {
                            self.titlebar.show = false;
                            self.titlebar.slide_ticks = 0;
                        }
                    }
                }
                for p in &mut self.players {
                    if p.update_frame() {
                        self.loading.stop(&p.id);
                    }
                }
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    let mut heights_changed = false;
                    for p in &self.players {
                        let h = match p.aspect_ratio() {
                            Some(ar) => self.scroll_engine.item_height_for_aspect_ratio(ar),
                            None => self.scroll_engine.default_item_height(),
                        };
                        if self.scroll_engine.update_height(p.id, h) {
                            heights_changed = true;
                        }
                    }
                    if heights_changed {
                        self.scroll_engine.recalculate_positions();
                    }
                }
                Task::none()
            }

            Message::WatchdogTick => {
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                // Auto-clear loading state if it exceeds 10 seconds to avoid indefinite spinner
                let stale_loading: Vec<PlayerId> = self
                    .loading
                    .player_ticks
                    .iter_mut()
                    .filter_map(|(&id, ticks)| {
                        *ticks += 1;
                        if *ticks >= 10 { Some(id) } else { None }
                    })
                    .collect();
                for id in stale_loading {
                    self.loading.stop(&id);
                }
                self.loading
                    .player_ids
                    .retain(|id| self.players.iter().any(|p| p.id == *id));
                self.loading
                    .player_ticks
                    .retain(|id, _| self.players.iter().any(|p| p.id == *id));

                let mut finished_ids = Vec::new();
                let mut stuck_ids = Vec::new();

                for p in &mut self.players {
                    if p.is_finished() {
                        finished_ids.push(p.id);
                    } else if p.check_stuck() {
                        stuck_ids.push(p.id);
                    }
                }

                for id in finished_ids {
                    log::info!("Player {id} video reached end, advancing to next video");
                    let _ = self.update(Message::AutoAdvanceVideo(id));
                }

                for id in stuck_ids {
                    log::warn!(
                        "Player {id} playback stuck for {}s, skipping to next video",
                        VideoHandle::STUCK_THRESHOLD_SECONDS
                    );
                    let _ = self.update(Message::AutoAdvanceVideo(id));
                }

                if self.overlay.toast_message.is_some() {
                    if self.overlay.toast_time_remaining > 0 {
                        self.overlay.toast_time_remaining -= 1;
                    } else {
                        self.overlay.toast_message = None;
                    }
                }
                if self.settings.playback_mode == PlaybackMode::Flip {
                    let interval = self.settings.flip_interval_secs.max(1);
                    if self.flip.countdown > interval || self.flip.countdown == 0 {
                        self.flip.countdown = interval;
                    }
                    if self.flip.countdown > 1 {
                        self.flip.countdown -= 1;
                    } else {
                        self.flip.countdown = interval;
                        let _ = self.update(Message::FlipModeTick);
                    }
                }
                self.save_session_state();
                Task::none()
            }

            Message::FlipModeTick => {
                self.flip.reset(self.settings.flip_interval_secs);
                if self.settings.playback_mode == PlaybackMode::Flip && !self.players.is_empty() {
                    let rand_id = self.players[rand::random::<usize>() % self.players.len()].id;
                    self.loading.start(rand_id);
                    let _ = self.update(Message::NextVideo(rand_id));
                }
                Task::none()
            }

            // =========================================================================
            // Window Lifecycle, Geometry & Input
            // =========================================================================
            Message::WindowIdReceived(_)
            | Message::GainWindowFocus
            | Message::WindowFocused
            | Message::WindowUnfocused
            | Message::WindowMoved(_, _)
            | Message::WindowResized(_, _)
            | Message::MinimizeWindow
            | Message::MaximizeWindow
            | Message::DragWindow
            | Message::DragResize(_)
            | Message::CloseApp
            | Message::ModifiersChanged(_)
            | Message::KeyPressed(_, _)
            | Message::KeyReleased(_)
            | Message::CursorMoved(_, _)
            | Message::CursorLeft
            | Message::TitleBarPressed
            | Message::LeftClickReleased
            | Message::RightClickPressed(_)
            | Message::ToggleDropdownMenu
            | Message::CloseDropdownMenu
            | Message::SetWindowOpacity(_)
            | Message::ToggleAlwaysOnTop => self.update_window(message),

            // =========================================================================
            // Drawers (File Picker, Transcript, History)
            // =========================================================================
            Message::ToggleFilePicker
            | Message::ToggleTranscript
            | Message::ToggleTranscriptForPlayer(_)
            | Message::ToggleHistoryDrawer
            | Message::CloseHistoryDrawer
            | Message::HistorySearchChanged(_)
            | Message::ClearPlayHistory
            | Message::CloseTranscript
            | Message::TranscriptSearchChanged(_)
            | Message::TranscriptLoaded(_, _, _)
            | Message::SeekToSubtitle(_)
            | Message::ToggleTranscriptSubtitleMenu
            | Message::CloseTranscriptSubtitleMenu
            | Message::SelectTranscriptSubtitleTrack(_, _)
            | Message::ToggleFolderCollapse(_)
            | Message::FilePickerSearchChanged(_)
            | Message::ApplyFilePickerSearch
            | Message::PlayFileInFocused(_) => self.update_drawers(message),

            // =========================================================================
            // Search & Folder Filtering
            // =========================================================================
            Message::SelectSearchFolder(_)
            | Message::ToggleSearchFolder(_)
            | Message::ResetSearchFolder
            | Message::RemoveActiveSearchFolder(_)
            | Message::SearchInputChanged(_)
            | Message::RemoveSearchTag(_)
            | Message::PerformSearch
            | Message::ClearSearch
            | Message::FolderInputChanged(_) => self.update_search(message),

            // =========================================================================
            // Scanner & Library Folders
            // =========================================================================
            Message::PickFolders
            | Message::FoldersSelected(_)
            | Message::AddMediaFolder
            | Message::RemoveMediaFolder(_)
            | Message::ClearMiscVideos
            | Message::StartScan
            | Message::ScanProgressUpdate(_, _)
            | Message::ScanFinished(_, _) => self.update_scanner(message),

            // =========================================================================
            // Modals & Settings Dialogs
            // =========================================================================
            Message::OpenMenuModal
            | Message::CloseMenuModal
            | Message::OpenSearchModal
            | Message::CloseSearchModal
            | Message::OpenSettingsModal
            | Message::CloseSettingsModal
            | Message::SetSettingsTab(_)
            | Message::SetLanguage(_)
            | Message::SetGamma(_)
            | Message::SetContrast(_)
            | Message::SetBrightness(_)
            | Message::SetSaturation(_)
            | Message::SetPlaybackSpeed(_)
            | Message::SetBufferDuration(_)
            | Message::SetBufferSize(_)
            | Message::SetFlipInterval(_)
            | Message::ResetPlaybackOptions
            | Message::ToggleDefaultPlayer
            | Message::OpenAlkaliWebsite
            | Message::OpenHelpModal
            | Message::CloseHelpModal
            | Message::ToggleBookmarksModal
            | Message::CloseBookmarksModal
            | Message::EscapePressed
            | Message::DismissToast
            | Message::ModalCardClicked => self.update_modals(message),

            // =========================================================================
            // Playback, Layout & Multi-Player Navigation
            // =========================================================================
            _ => self.update_playback(message),
        }
    }
}
