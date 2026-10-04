/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * State Reducer & Event Dispatcher
 *
 * Implements WazooApp::update, processing incoming Message events, executing playback commands,
 * managing modal dialogs, handling window events, and orchestrating asynchronous tasks.
 */

pub mod cube;
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
    /// Primary state reducer for the application. Processes incoming messages,
    /// coordinates engine ticks, routes sub-domain events, and produces asynchronous tasks.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // =========================================================================
            // Engine Ticks & Frame Updates
            // =========================================================================
            Message::AnimationTick => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    // Advance scroll physics and collect player IDs that have moved entirely off-screen
                    let offscreen = self.scroll_engine.tick();
                    if !offscreen.is_empty() {
                        let (keep, despawned): (Vec<_>, Vec<_>) = self
                            .players
                            .drain(..)
                            .partition(|p| !offscreen.contains(&p.id));
                        self.players = PlayerList(keep);
                        // Offload pipeline teardown to a background thread to prevent deallocation
                        // and driver teardown latency (20-50ms) from causing UI frame drops during fast scrolling.
                        if !despawned.is_empty() {
                            std::thread::spawn(move || drop(despawned));
                        }
                    }

                    let margin = self.scroll_engine.default_item_height() * 1.5;
                    let mut needs_preload = false;

                    // Non-blocking spawn: attach preloaded player seamlessly into the feed if ready
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
                            // Preloaded player is still decoding in the background; do not block UI thread
                            break;
                        }
                    }

                    // Dynamically update audio volume falloff based on vertical proximity to viewport center
                    for p in &mut self.players {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }

                    // Trigger next background pre-warm task if the pipeline buffer is empty
                    if needs_preload || (self.preloaded_player.is_none() && !self.is_preloading) {
                        return self.trigger_preload_task();
                    }
                }
                Task::none()
            }

            Message::VideoFrameTick => {
                // Drop non-essential frame processing when window is unfocused to conserve battery/CPU
                if self.should_throttle_unfocused_frame() {
                    return Task::none();
                }

                // Suspend all tick animations, layout recalculations, and physics
                // while the window is actively being resized to eliminate visual tearing and IPC lag.
                if self.window.is_resizing() {
                    self.update_player_frames();
                    return Task::none();
                }

                self.tick_overlay_animations();
                self.tick_titlebar_animation();
                self.update_player_frames();
                self.sync_scroll_item_heights();
                self.tick_cube_screensaver();

                Task::none()
            }

            Message::WatchdogTick => {
                // Debounced persistence: saves window geometry to disk on watchdog intervals
                // rather than thrashing disk I/O on every single pixel of mouse dragging.
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }

                // Detect completed streams, unplayable files, and frozen playback pipelines
                let mut finished_ids = Vec::new();
                let mut stuck_ids = Vec::new();

                for p in &mut self.players {
                    if p.is_failed() {
                        p.stop_loading();
                        stuck_ids.push(p.id);
                        continue;
                    }
                    if p.is_loading {
                        p.loading_ticks += 1;
                        if p.loading_ticks >= 10 {
                            p.stop_loading();
                            stuck_ids.push(p.id);
                        }
                        continue;
                    }
                    if p.is_finished() {
                        finished_ids.push(p.id);
                    } else if p.check_stuck() {
                        stuck_ids.push(p.id);
                    }
                }

                // Automatically advance ended videos to keep the continuous playback experience seamless
                for id in finished_ids {
                    log::info!("Player {id} video reached end, advancing to next video");
                    let _ = self.update(Message::AutoAdvanceVideo(id));
                }

                // Skip stalled pipelines or failed videos that refuse to play
                for id in stuck_ids {
                    log::warn!(
                        "Player {id} playback stuck (>={}s) or failed to play, skipping to next video",
                        VideoHandle::STUCK_THRESHOLD_SECONDS
                    );
                    let _ = self.update(Message::AutoAdvanceVideo(id));
                }

                // Decrement active toast notification countdown
                if self.overlay.toast_message.is_some() {
                    if self.overlay.toast_time_remaining > 0 {
                        self.overlay.toast_time_remaining -= 1;
                    }
                    if self.overlay.toast_time_remaining == 0 {
                        self.overlay.toast_message = None;
                    }
                }

                // In Flip Mode, decrement player flip countdowns and stagger video transitions
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

                // Persist active video paths and playhead positions periodically
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
            | Message::ToggleFullscreen
            | Message::FitWindow
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
            | Message::ClearFilePickerSearch
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
            | Message::ToggleFilters
            | Message::SetFiltersEnabled(_)
            | Message::ToggleCrtFilter
            | Message::SetCrtFilter(_)
            | Message::ToggleWavyFilter
            | Message::SetWavyFilter(_)
            | Message::ToggleFogFilter
            | Message::SetFogFilter(_)
            | Message::SetLoadingIndicator(_)
            | Message::SetBufferDuration(_)
            | Message::SetBufferSize(_)
            | Message::SetFlipInterval(_)
            | Message::ResetPlaybackOptions
            | Message::ToggleDefaultPlayer
            | Message::OpenAlkaliWebsite
            | Message::OpenHelpModal
            | Message::CloseHelpModal
            | Message::HelpSearchChanged(_)
            | Message::ClearHelpSearch
            | Message::ToggleBookmarksModal
            | Message::CloseBookmarksModal
            | Message::EscapePressed
            | Message::DismissToast
            | Message::ModalCardClicked => self.update_modals(message),

            // =========================================================================
            // 3D Video Cube & Cube Overlay Mode
            // =========================================================================
            Message::ToggleCubeScreensaver
            | Message::ToggleDesktopCubeScreensaver
            | Message::ToggleCubeSheen
            | Message::SpawnCube
            | Message::RemoveCube
            | Message::ClearCubes
            | Message::SetCubeSpeed(_)
            | Message::SetCubeSize(_) => self.update_cube(message),

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
        self.update_static_frame();

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

        if self.modals.playback_settings_debounce_ticks > 0 {
            self.modals.playback_settings_debounce_ticks -= 1;
            if self.modals.playback_settings_debounce_ticks == 0 {
                self.apply_playback_settings();
            }
        }

        for p in &mut self.players {
            p.tick_seek_debounce();
        }
    }

    /// Renders new video frames on active player handles, clearing loading state upon completion.
    /// In desktop overlay mode, only the active 3D cube player updates frames to save 100% background CPU.
    fn update_player_frames(&mut self) {
        if self.cube.desktop_overlay {
            for p in &mut self.players {
                if p.is_cube {
                    if p.update_frame() {
                        p.stop_loading();
                    } else if p.is_failed() {
                        p.stop_loading();
                    }
                }
            }
            return;
        }

        for p in &mut self.players {
            if p.update_frame() {
                p.stop_loading();
            } else if p.is_failed() {
                p.stop_loading();
            }
        }
    }

    /// Synchronizes scroll stream layout heights with players' native aspect ratios.
    fn sync_scroll_item_heights(&mut self) {
        if self.cube.desktop_overlay {
            return;
        }
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

    /// Advances the animated TV static noise texture if any active player is currently loading.
    fn update_static_frame(&mut self) {
        if self.raw_static_frames.is_empty() {
            return;
        }

        let is_needed = self.settings.loading_indicator == wazoo_core::LoadingIndicator::TvStatic
            && self.players.iter().any(|p| p.is_loading);

        if !is_needed {
            return;
        }

        let frame_idx = self
            .overlay
            .color_static_frame_index(self.raw_static_frames.len());

        if frame_idx != self.last_static_frame_idx {
            self.last_static_frame_idx = frame_idx;
            let raw = &self.raw_static_frames[frame_idx];
            if let Ok(mut guard) = self.static_frame.lock() {
                guard.width = raw.width;
                guard.height = raw.height;
                if guard.pixels.len() != raw.pixels.len() {
                    guard.pixels.resize(raw.pixels.len(), 0);
                }
                guard.pixels.copy_from_slice(&raw.pixels);
                guard.new_frame = true;
                guard.frame_seq = guard.frame_seq.wrapping_add(1);
            }
        }
    }
}
