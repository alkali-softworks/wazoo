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
pub mod plex;
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
            Message::Tick => {
                // Suspend all tick animations, layout recalculations, and physics
                // while the window is actively being resized to eliminate visual tearing and IPC lag.
                if !self.cube.desktop_overlay && self.window.is_resizing() {
                    self.update_player_frames();
                    return Task::none();
                }

                self.tick_overlay_animations();
                self.tick_titlebar_animation();
                self.sync_scroll_item_heights();
                let cube_task = self.tick_cube_screensaver();

                let scroll_task = self.tick_scroll_mode();

                // Selective frame throttling: drop heavy video frame decoding to 30 FPS when
                // unfocused to conserve battery/GPU, while keeping animations, debounces, and physics at 60 Hz.
                if !self.should_throttle_unfocused_frame() {
                    self.update_player_frames();
                }

                Task::batch([scroll_task, cube_task])
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
                    if self.cube.desktop_overlay && !p.is_cube {
                        continue;
                    }
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
                        if self.cube.desktop_overlay && !player.is_cube {
                            continue;
                        }
                        if player.flip.countdown > interval || player.flip.countdown == 0 {
                            player.flip.countdown = interval;
                        }
                        if !player.is_playing() {
                            continue;
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
                    self.update_static_frame();
                }

                // Persist active video paths and playhead positions periodically
                if !self.cube.desktop_overlay {
                    self.save_session_state();
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
            | Message::ToggleFullscreen
            | Message::FitWindow
            | Message::DragWindow
            | Message::DragResize(_)
            | Message::AltDragResize
            | Message::CloseApp
            | Message::ModifiersChanged(_)
            | Message::KeyPressed(_, _)
            | Message::KeyReleased(_)
            | Message::CursorMoved(_, _)
            | Message::CursorLeft
            | Message::TitleBarPressed
            | Message::LeftClickReleased
            | Message::RightClickReleased
            | Message::RightClickPressed(_)
            | Message::ToggleDropdownMenu
            | Message::CloseDropdownMenu
            | Message::SetWindowOpacity(_)
            | Message::ToggleAlwaysOnTop
            | Message::CyclePinMode => self.update_window(message),

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
            | Message::SetCrtScanlineDarkness(_)
            | Message::SetCrtScanlineDensity(_)
            | Message::SetCrtScanlineSpeed(_)
            | Message::SetCrtCurvature(_)
            | Message::SetCrtGamma(_)
            | Message::SetCrtSaturation(_)
            | Message::SetCrtBrightness(_)
            | Message::SetCrtVignette(_)
            | Message::SetCrtChromaticAberration(_)
            | Message::SetCrtNoise(_)
            | Message::SetCrtCornerRounding(_)
            | Message::ApplyCrtPreset(_)
            | Message::ResetCrtSettings
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
            | Message::ToggleStandbyPlayer
            | Message::OpenAlkaliWebsite
            | Message::OpenHelpModal
            | Message::CloseHelpModal
            | Message::HelpSearchChanged(_)
            | Message::ClearHelpSearch
            | Message::ToggleBookmarksModal
            | Message::CloseBookmarksModal
            | Message::ToggleQuickPickerModal
            | Message::OpenQuickPickerModal
            | Message::CloseQuickPickerModal
            | Message::QuickPickerSearchChanged(_)
            | Message::ClearQuickPickerSearch
            | Message::QuickPickerSubmit
            | Message::PlayQuickPickerFile(_)
            | Message::TogglePlexExplorerModal
            | Message::OpenPlexExplorerModal
            | Message::ClosePlexExplorerModal
            | Message::OpenPlexSettingsModal
            | Message::PlexExplorerSearchChanged(_)
            | Message::ClearPlexExplorerSearch
            | Message::PlexExplorerSelectTab(_)
            | Message::PlexExplorerToggleGenre(_)
            | Message::PlexExplorerToggleDecade(_)
            | Message::PlexExplorerSubmit
            | Message::PlayPlexExplorerItem(_, _)
            | Message::PlexExplorerLiveLoaded(_, _)
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
            // Plex Media Server Integration
            // =========================================================================
            Message::SetPlexEnabled(_)
            | Message::PlexServerUrlChanged(_)
            | Message::PlexTokenChanged(_)
            | Message::CopyPlexToken
            | Message::SetPlexAutoSync(_)
            | Message::TestPlexConnection
            | Message::PlexConnectionTested(_)
            | Message::StartPlexSync
            | Message::PlexSyncProgressUpdate(_)
            | Message::PlexSyncFinished(_) => self.update_plex(message),

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

    /// Advances scroll feed physics and handles off-screen player despawns/spawns in Scroll Mode.
    fn tick_scroll_mode(&mut self) -> Task<Message> {
        if self.settings.playback_mode != PlaybackMode::Scroll {
            return Task::none();
        }

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
        while let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(margin) {
            if let Some(mut handle) = self.preloaded_player.take() {
                handle.set_muted(self.settings.scroll_mode_muted);
                handle.set_paused(false);
                let item_h = self.calculate_player_scroll_height(&handle);
                self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                let vol = self.scroll_engine.calculate_player_volume(handle.id);
                handle.set_crossfade_volume(vol);
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
            p.set_crossfade_volume(vol);
        }

        // Trigger next background pre-warm task if the pipeline buffer is empty
        if needs_preload || (self.preloaded_player.is_none() && !self.is_preloading) {
            return self.trigger_preload_task();
        }

        Task::none()
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
                if let Some(ar) = p.handle.aspect_ratio() {
                    p.last_aspect_ratio = Some(ar);
                }
            } else if p.is_failed() {
                p.stop_loading();
            } else if !p.is_loading {
                if let Some(ar) = p.handle.aspect_ratio() {
                    p.last_aspect_ratio = Some(ar);
                }
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
    /// Cropped to match the dimensions and aspect ratio of the player's previous video so
    /// it displays seamlessly with identical letterboxing/pillarboxing and zero distortion.
    pub fn update_static_frame(&mut self) {
        if self.raw_static_frames.is_empty() {
            return;
        }

        let is_needed = self.settings.loading_indicator == wazoo_core::LoadingIndicator::TvStatic
            && self.players.iter().any(|p| !p.is_cube && p.is_loading);

        if !is_needed {
            return;
        }

        let frame_idx = self
            .overlay
            .color_static_frame_index(self.raw_static_frames.len());

        let is_new_gif_frame = frame_idx != self.last_static_frame_idx;
        if is_new_gif_frame {
            self.last_static_frame_idx = frame_idx;
        }

        let raw = &self.raw_static_frames[frame_idx];

        for p in self.players.iter_mut() {
            if !p.is_cube && p.is_loading {
                let target_aspect = p.aspect_ratio().unwrap_or(16.0 / 9.0);
                if let Ok(mut guard) = p.static_frame.lock() {
                    let current_aspect = if guard.height > 0 {
                        guard.width as f32 / guard.height as f32
                    } else {
                        0.0
                    };
                    let aspect_diff = (current_aspect - target_aspect).abs();
                    if is_new_gif_frame || guard.width == 0 || aspect_diff > 0.02 {
                        Self::crop_static_frame(raw, target_aspect, &mut guard);
                    }
                }
            }
        }
    }

    /// Crops the raw 4:3 static noise texture to match a target video aspect ratio.
    /// Centered crop preserves texture pixel squareness without stretching or distortion.
    pub fn crop_static_frame(
        raw: &crate::assets::RawStaticFrame,
        target_aspect: f32,
        guard: &mut wazoo_media::pipeline::FrameData,
    ) {
        let w = raw.width;
        let h = raw.height;
        if w == 0 || h == 0 || raw.pixels.is_empty() {
            return;
        }

        let raw_aspect = w as f32 / h as f32;
        let (crop_w, crop_h, start_x, start_y) = if target_aspect >= raw_aspect {
            // Target is wider than or equal to raw (e.g. 16:9 vs 4:3)
            // Full width, crop top and bottom
            let ch = ((w as f32 / target_aspect).round() as u32).clamp(2, h);
            let sy = (h - ch) / 2;
            (w, ch, 0, sy)
        } else {
            // Target is taller than raw (e.g. 1:1 or 9:16 vertical vs 4:3)
            // Full height, crop left and right
            let cw = ((h as f32 * target_aspect).round() as u32).clamp(2, w);
            let sx = (w - cw) / 2;
            (cw, h, sx, 0)
        };

        guard.width = crop_w;
        guard.height = crop_h;

        let total_bytes = (crop_w * crop_h * 4) as usize;
        if guard.pixels.len() != total_bytes {
            guard.pixels.resize(total_bytes, 0);
        }

        let src_stride = (w * 4) as usize;
        let row_bytes = (crop_w * 4) as usize;
        let src_start_offset = (start_y * w + start_x) as usize * 4;

        for row in 0..crop_h as usize {
            let src_start = src_start_offset + row * src_stride;
            let src_end = src_start + row_bytes;
            let dest_start = row * row_bytes;
            let dest_end = dest_start + row_bytes;
            if src_end <= raw.pixels.len() && dest_end <= guard.pixels.len() {
                guard.pixels[dest_start..dest_end]
                    .copy_from_slice(&raw.pixels[src_start..src_end]);
            }
        }

        guard.new_frame = true;
        guard.frame_seq = guard.frame_seq.wrapping_add(1);
    }
}
