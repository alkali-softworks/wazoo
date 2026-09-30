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

use crate::app::{PLAYER_OVERLAY_FADE_TICKS, PLAYER_OVERLAY_HIDE_TICKS, WazooApp};
use crate::message::Message;
use crate::state::{AppPlayer, PlayerList};
use iced::Task;
use wazoo_core::PlaybackMode;
use wazoo_media::VideoHandle;

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
                        self.players = PlayerList(keep);
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
                        if let Some(mut handle) = self.preloaded_player.take() {
                            handle.set_muted(self.settings.scroll_mode_muted);
                            handle.set_paused(false);
                            let item_h = self.calculate_player_scroll_height(&handle);
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.record_play_history(&handle.state.path);
                            self.players
                                .push(AppPlayer::new(handle, self.default_shuffle_mode));
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

                    if needs_preload || (self.preloaded_player.is_none() && !self.is_preloading) {
                        return self.trigger_preload_task();
                    }
                }
                Task::none()
            }

            Message::VideoFrameTick => {
                if self.should_throttle_unfocused_frame() {
                    return Task::none();
                }

                self.tick_overlay_animations();
                self.tick_titlebar_animation();
                self.update_player_frames();
                self.sync_scroll_item_heights();

                Task::none()
            }

            Message::WatchdogTick => {
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                // Auto-clear loading state if it exceeds 10 seconds to avoid indefinite spinner
                for p in &mut self.players {
                    if p.is_loading {
                        p.loading_ticks += 1;
                        if p.loading_ticks >= 10 {
                            p.stop_loading();
                        }
                    }
                }

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
                    let mut to_flip = Vec::new();
                    for player in self.players.iter_mut() {
                        if player.flip.countdown > interval || player.flip.countdown == 0 {
                            player.flip.countdown = interval;
                        }
                        if player.flip.countdown > 1 {
                            player.flip.countdown -= 1;
                        } else {
                            player.flip.reset(interval);
                            to_flip.push(player.id);
                        }
                    }
                    for id in to_flip {
                        if let Some(p) = self.players.player_mut(id) {
                            p.start_loading();
                        }
                        let _ = self.advance_player_to_next_video(id, false);
                    }
                }
                self.save_session_state();
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
            | Message::FolderInputChanged(_)
            | Message::SetActiveQuery(_)
            | Message::TitleOverlayClicked(_) => self.update_search(message),

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

impl WazooApp {
    /// Advances HUD overlays, spinner rotations, and file picker search debouncing.
    fn tick_overlay_animations(&mut self) {
        self.overlay.spinner_ticks = self.overlay.spinner_ticks.wrapping_add(1);

        if self.open_audio_menu_id.is_some() {
            self.overlay.ticks = PLAYER_OVERLAY_HIDE_TICKS;
            self.overlay.fade_in_ticks = PLAYER_OVERLAY_FADE_TICKS;
        } else if self.overlay.ticks > 0 {
            if self.overlay.fade_in_ticks < PLAYER_OVERLAY_FADE_TICKS {
                self.overlay.fade_in_ticks += 1;
            }
            self.overlay.ticks -= 1;
            if self.overlay.ticks == 0 {
                self.hovered_player_id = None;
                self.overlay.fade_in_ticks = 0;
            }
        } else {
            self.overlay.fade_in_ticks = 0;
        }

        self.overlay.title_pill_ticks = self.overlay.title_pill_ticks.saturating_sub(1);
        self.overlay.focus_border_ticks = self.overlay.focus_border_ticks.saturating_sub(1);

        if self.drawers.file_picker_debounce_ticks > 0 {
            self.drawers.file_picker_debounce_ticks -= 1;
            if self.drawers.file_picker_debounce_ticks == 0 {
                self.apply_file_picker_search();
            }
        }
    }

    /// Renders new video frames on all active player handles, clearing loading state upon completion.
    fn update_player_frames(&mut self) {
        for p in &mut self.players {
            if p.update_frame() {
                p.stop_loading();
            }
        }
    }

    /// Synchronizes scroll stream layout heights with players' native aspect ratios.
    fn sync_scroll_item_heights(&mut self) {
        if self.settings.playback_mode == PlaybackMode::Scroll {
            let mut heights_changed = false;
            for p in &self.players {
                let h = self.calculate_player_scroll_height(p);
                if self.scroll_engine.update_height(p.id, h) {
                    heights_changed = true;
                }
            }
            if heights_changed {
                self.scroll_engine.recalculate_positions();
            }
        }
    }
}
