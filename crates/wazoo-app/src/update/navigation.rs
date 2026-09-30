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
            p.handle = new_handle;
        }
    }

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

            let start_time = target
                .position_secs
                .map(StartTime::Seconds)
                .unwrap_or(StartTime::Beginning);

            if let Some(p) = self.players.player_mut(id) {
                if p.handle.load_file(&target.path, &name, start_time).is_ok() {
                    p.handle.set_subtitles_visible(self.subtitles_enabled);
                    self.push_player_nav_entry(id, target.path.clone(), target.position_secs);
                    loaded = true;
                }
            }

            if !loaded {
                if let Ok(new_handle) =
                    self.create_video_handle_with_start(id, &target.path, &name, target.position_secs)
                {
                    self.apply_playback_state_and_replace(id, new_handle, prev_muted, prev_volume);
                    self.push_player_nav_entry(id, target.path.clone(), target.position_secs);
                    loaded = true;
                }
            }
        }

        // 2. If forward_stack had no entries (or loading failed), generate next video (random or sequential)
        if !loaded {
            let start_time = if self.settings.playback_mode == PlaybackMode::Scroll
                || self.settings.playback_mode == PlaybackMode::Flip
            {
                StartTime::Random
            } else {
                StartTime::Beginning
            };
            for _ in 0..MAX_VIDEO_LOAD_RETRIES {
                if let Some(video_rec) = self
                    .get_next_video_rec_with_mode(curr_path.as_deref(), self.is_player_shuffle(id))
                {
                    if let Some(p) = self.players.player_mut(id) {
                        if p.handle.load_file(&video_rec.path, &video_rec.name, start_time).is_ok() {
                            p.handle.set_subtitles_visible(self.subtitles_enabled);
                            self.push_player_nav_entry(id, video_rec.path.clone(), None);
                            break;
                        }
                    }

                    if let Ok(new_handle) = self.create_video_handle_with_start_time(
                        id,
                        &video_rec.path,
                        &video_rec.name,
                        start_time,
                    ) {
                        self.apply_playback_state_and_replace(
                            id,
                            new_handle,
                            prev_muted,
                            prev_volume,
                        );
                        self.push_player_nav_entry(id, video_rec.path.clone(), None);
                        break;
                    }
                }
            }
        }
        if self.drawers.show_transcript && self.focused_player_id() == Some(id) {
            return self.load_transcript_for_focused_player();
        }
        Task::none()
    }

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

            let start_time = target
                .position_secs
                .map(StartTime::Seconds)
                .unwrap_or(StartTime::Beginning);

            if let Some(p) = self.players.player_mut(id) {
                if p.handle.load_file(&target.path, &name, start_time).is_ok() {
                    p.handle.set_subtitles_visible(self.subtitles_enabled);
                    self.record_play_history(&target.path);
                    loaded = true;
                }
            }

            if !loaded {
                if let Ok(new_handle) =
                    self.create_video_handle_with_start(id, &target.path, &name, target.position_secs)
                {
                    self.apply_playback_state_and_replace(id, new_handle, prev_muted, prev_volume);
                    self.record_play_history(&target.path);
                    loaded = true;
                }
            }
        }

        // 3. Fallback if back_stack had no earlier entries (or loading failed)
        if !loaded {
            for _ in 0..MAX_VIDEO_LOAD_RETRIES {
                if let Some(video_rec) = self
                    .get_prev_video_rec_with_mode(curr_path.as_deref(), self.is_player_shuffle(id))
                {
                    if let Some(p) = self.players.player_mut(id) {
                        if p.handle.load_file(&video_rec.path, &video_rec.name, StartTime::Beginning).is_ok() {
                            p.handle.set_subtitles_visible(self.subtitles_enabled);
                            self.push_player_nav_entry(id, video_rec.path.clone(), None);
                            break;
                        }
                    }

                    if let Ok(new_handle) =
                        self.create_video_handle(id, &video_rec.path, &video_rec.name)
                    {
                        self.apply_playback_state_and_replace(
                            id,
                            new_handle,
                            prev_muted,
                            prev_volume,
                        );
                        self.push_player_nav_entry(id, video_rec.path.clone(), None);
                        break;
                    }
                }
            }
        }

        if self.drawers.show_transcript && self.focused_player_id() == Some(id) {
            return self.load_transcript_for_focused_player();
        }
        Task::none()
    }
}
