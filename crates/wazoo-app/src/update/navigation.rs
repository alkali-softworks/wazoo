/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Player Navigation & Video Replacement
 *
 * Implements forward/back video advance routines, history stack recording,
 * and seamless player state replacement.
 */

use crate::app::{
    FOCUS_BORDER_TICKS, MAX_PLAYER_NAV_HISTORY_ENTRIES, MAX_VIDEO_LOAD_RETRIES, WazooApp,
};
use crate::format;
use crate::message::Message;
use iced::Task;
use wazoo_core::PlaybackMode;
use wazoo_media::{PlayerId, StartTime, VideoHandle};

impl WazooApp {
    /// Focuses the specified player, triggers the green outline flash, activates loading state,
    /// and logs the current playback playhead to the player's navigation history stack.
    pub(crate) fn focus_player_for_navigation(&mut self, id: PlayerId, request_focus: bool) {
        if request_focus {
            if let Some(pos) = self.players.player_index(id) {
                let was_already_active = self.focused_idx == pos;
                self.focused_idx = pos;
                if !was_already_active {
                    self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                }
            }
            self.trigger_player_overlay();
        }
        if let Some(p) = self.players.player_mut(id) {
            p.start_loading();
        }
        self.record_current_player_nav_position(id);
    }

    /// Replaces a player's underlying video handle with a newly initialized one while preserving
    /// user audio preferences (volume and mute status) and global subtitle visibility.
    pub(crate) fn apply_playback_state_and_replace(
        &mut self,
        id: PlayerId,
        mut new_handle: VideoHandle,
        prev_muted: Option<bool>,
        prev_volume: Option<f64>,
    ) {
        new_handle.set_muted(prev_muted.unwrap_or(false));
        if let Some(vol) = prev_volume {
            new_handle.set_volume(vol);
        }
        new_handle.set_subtitles_visible(self.subtitles_enabled);
        if let Some(p) = self.players.player_mut(id) {
            p.cancel_seek_debounce();
            p.handle = new_handle;
            p.start_loading();
        }
    }

    /// Advances a player to the next video, checking the forward navigation history stack before
    /// generating a sequential or shuffled video from the library with automated load retries.
    pub(crate) fn advance_player_to_next_video(
        &mut self,
        id: PlayerId,
        request_focus: bool,
    ) -> Task<Message> {
        if request_focus {
            if let Some(p) = self.players.player_mut(id) {
                p.flip.reset(self.settings.flip_interval_secs);
            }
        }
        self.focus_player_for_navigation(id, request_focus);
        if let Some(p) = self.players.player_mut(id) {
            p.clear_marks();
            p.stop();
        }

        let curr_player = self.players.player(id);
        let curr_path = curr_player.map(|p| p.state.path.clone());
        let prev_muted = curr_player.map(|p| p.state.is_muted);
        let prev_volume = curr_player.map(|p| p.state.volume);

        // 1. Check forward_stack for undone videos from previous navigation
        let forward_candidate = self
            .players
            .player_mut(id)
            .and_then(|p| p.nav_history.forward_stack.pop());

        let mut loaded = false;
        if let Some(target) = forward_candidate {
            let name = self
                .available_videos
                .iter()
                .find(|v| v.path == target.path)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format::format_video_title(&target.path));

            if let Ok(new_handle) =
                self.create_video_handle_with_start(id, &target.path, &name, target.position_secs)
            {
                let path_clone = target.path.clone();
                self.apply_playback_state_and_replace(id, new_handle, prev_muted, prev_volume);
                self.push_player_nav_entry(id, path_clone.clone(), target.position_secs);
                if let Some(p) = self.players.player(id) {
                    if p.is_cube {
                        let (primary, _) = format::format_title_lines(&path_clone);
                        self.overlay
                            .show_toast(self.t_with("toast.cube_playing", &[("title", &primary)]));
                    }
                }
                loaded = true;
            }
        }

        // 2. If forward_stack had no entries (or loading failed), generate next video (random or sequential)
        if !loaded {
            let start_time = if self.settings.playback_mode == PlaybackMode::Scroll
                || self.settings.playback_mode == PlaybackMode::Flip
                || self.players.player(id).map(|p| p.is_cube).unwrap_or(false)
            {
                StartTime::Random
            } else {
                StartTime::Beginning
            };
            let filtered = self.file_picker_filtered_videos();
            let standby_candidate = self.take_standby_player_if(|h| {
                let is_same = curr_path.as_deref() == Some(&h.state.path);
                let allowed_by_filter = filtered
                    .as_ref()
                    .map(|f| f.iter().any(|v| v.path == h.state.path))
                    .unwrap_or(true);
                (!is_same || self.available_videos.len() <= 1) && allowed_by_filter
            });

            let handle_and_path = if let Some(mut handle) = standby_candidate {
                handle.id = id;
                handle.state.id = id;
                handle.set_paused(false);
                let path = handle.state.path.clone();
                Some((handle, path, true))
            } else {
                let mut candidate_curr_path = curr_path;
                let mut generated = None;
                for _ in 0..MAX_VIDEO_LOAD_RETRIES {
                    if let Some(video_rec) = self.get_next_video_rec_for_navigation(
                        candidate_curr_path.as_deref(),
                        self.is_player_shuffle(id),
                    ) {
                        if let Ok(new_handle) = self.create_video_handle_with_start_time(
                            id,
                            &video_rec.path,
                            &video_rec.name,
                            start_time,
                        ) {
                            generated = Some((new_handle, video_rec.path, false));
                            break;
                        }
                        candidate_curr_path = Some(video_rec.path);
                    }
                }
                generated
            };

            if let Some((new_handle, path, from_standby)) = handle_and_path {
                self.apply_playback_state_and_replace(id, new_handle, prev_muted, prev_volume);
                if from_standby {
                    if let Some(p) = self.players.player_mut(id) {
                        p.stop_loading();
                    }
                }
                self.push_player_nav_entry(id, path.clone(), None);
                if let Some(p) = self.players.player(id) {
                    if p.is_cube {
                        let (primary, _) = format::format_title_lines(&path);
                        self.overlay.show_toast(self.t_with(
                            "toast.cube_playing",
                            &[("title", &primary)],
                        ));
                    }
                }
            }
        }
        if self.drawers.show_transcript && self.focused_player_id() == Some(id) {
            return self.load_transcript_for_focused_player();
        }
        Task::none()
    }

    /// Steps a player back to the previously played video, popping from the back navigation history
    /// stack and preserving the forward stack for subsequent redo navigation.
    pub(crate) fn advance_player_to_prev_video(
        &mut self,
        id: PlayerId,
        request_focus: bool,
    ) -> Task<Message> {
        if request_focus {
            if let Some(p) = self.players.player_mut(id) {
                p.flip.reset(self.settings.flip_interval_secs);
            }
        }
        self.focus_player_for_navigation(id, request_focus);
        if let Some(p) = self.players.player_mut(id) {
            p.clear_marks();
            p.stop();
        }

        let curr_player = self.players.player(id);
        let curr_path = curr_player.map(|p| p.state.path.clone());
        let prev_muted = curr_player.map(|p| p.state.is_muted);
        let prev_volume = curr_player.map(|p| p.state.volume);

        // 1. Adjust navigation history:
        // Pop the current video from back_stack and push onto forward_stack
        let mut target_candidate = None;
        if let Some(p) = self.players.player_mut(id) {
            let hist = &mut p.nav_history;
            if let Some(curr) = curr_path.as_deref() {
                if hist.back_stack.last().map(|e| e.path.as_str()) == Some(curr) {
                    let current_entry = hist.back_stack.pop().unwrap();
                    hist.forward_stack.push(current_entry);
                    if hist.forward_stack.len() > MAX_PLAYER_NAV_HISTORY_ENTRIES {
                        hist.forward_stack.remove(0);
                    }
                }
            }
            if let Some(prev_entry) = hist.back_stack.last() {
                target_candidate = Some(prev_entry.clone());
            }
        }

        // 2. Try playing target from back_stack if available
        let mut loaded = false;
        if let Some(target) = target_candidate {
            let name = self
                .available_videos
                .iter()
                .find(|v| v.path == target.path)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format::format_video_title(&target.path));

            if let Ok(new_handle) =
                self.create_video_handle_with_start(id, &target.path, &name, target.position_secs)
            {
                let path_clone = target.path.clone();
                self.apply_playback_state_and_replace(id, new_handle, prev_muted, prev_volume);
                self.record_play_history(&path_clone);
                if let Some(p) = self.players.player(id) {
                    if p.is_cube {
                        let (primary, _) = format::format_title_lines(&path_clone);
                        self.overlay
                            .show_toast(self.t_with("toast.cube_playing", &[("title", &primary)]));
                    }
                }
                loaded = true;
            }
        }

        // 3. Fallback if back_stack had no earlier entries (or loading failed)
        if !loaded {
            let mut candidate_curr_path = curr_path;
            for _ in 0..MAX_VIDEO_LOAD_RETRIES {
                if let Some(video_rec) = self.get_prev_video_rec_for_navigation(
                    candidate_curr_path.as_deref(),
                    self.is_player_shuffle(id),
                ) {
                    if let Ok(new_handle) =
                        self.create_video_handle(id, &video_rec.path, &video_rec.name)
                    {
                        let path_clone = video_rec.path.clone();
                        self.apply_playback_state_and_replace(
                            id,
                            new_handle,
                            prev_muted,
                            prev_volume,
                        );
                        self.push_player_nav_entry(id, path_clone.clone(), None);
                        if let Some(p) = self.players.player(id) {
                            if p.is_cube {
                                let (primary, _) = format::format_title_lines(&path_clone);
                                self.overlay.show_toast(self.t_with(
                                    "toast.cube_playing",
                                    &[("title", &primary)],
                                ));
                            }
                        }
                        break;
                    }
                    candidate_curr_path = Some(video_rec.path);
                }
            }
        }

        if self.drawers.show_transcript && self.focused_player_id() == Some(id) {
            return self.load_transcript_for_focused_player();
        }
        Task::none()
    }
}
