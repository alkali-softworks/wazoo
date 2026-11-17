/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Navigation Candidate Selection & Play History
 *
 * Implements sequential/shuffle next-previous video selection and per-player
 * navigation undo/redo stacks alongside global play history tracking.
 */

use super::{
    MAX_PLAYER_NAV_HISTORY_ENTRIES, MAX_PLAY_HISTORY_ENTRIES, PlayHistoryItem,
    PlaybackHistoryEntry, WazooApp,
};
use crate::format;
use wazoo_core::VideoRecord;
use wazoo_media::PlayerId;

impl WazooApp {
    pub fn is_player_shuffle(&self, id: PlayerId) -> bool {
        self.players
            .player(id)
            .map(|p| p.shuffle)
            .unwrap_or(self.default_shuffle_mode)
    }

    pub(crate) fn get_next_video_rec_from_pool(
        pool: &[VideoRecord],
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if pool.is_empty() {
            return None;
        }
        if is_shuffle {
            if pool.len() > 1 {
                if let Some(curr) = current_path {
                    let candidates: Vec<&VideoRecord> =
                        pool.iter().filter(|v| v.path != curr).collect();
                    if !candidates.is_empty() {
                        let idx = rand::random::<usize>() % candidates.len();
                        return Some(candidates[idx].clone());
                    }
                }
            }
            let idx = rand::random::<usize>() % pool.len();
            Some(pool[idx].clone())
        } else {
            if let Some(curr) = current_path {
                if let Some(pos) = pool.iter().position(|v| v.path == curr) {
                    // If we reach the end of the list in sequential mode, start over from the beginning
                    let next_pos = if pos + 1 >= pool.len() { 0 } else { pos + 1 };
                    return Some(pool[next_pos].clone());
                }
            }
            // If current video is not found or not provided, start from beginning of list
            Some(pool[0].clone())
        }
    }

    pub(crate) fn get_prev_video_rec_from_pool(
        pool: &[VideoRecord],
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if pool.is_empty() {
            return None;
        }
        if is_shuffle {
            if pool.len() > 1 {
                if let Some(curr) = current_path {
                    let candidates: Vec<&VideoRecord> =
                        pool.iter().filter(|v| v.path != curr).collect();
                    if !candidates.is_empty() {
                        let idx = rand::random::<usize>() % candidates.len();
                        return Some(candidates[idx].clone());
                    }
                }
            }
            let idx = rand::random::<usize>() % pool.len();
            Some(pool[idx].clone())
        } else {
            if let Some(curr) = current_path {
                if let Some(pos) = pool.iter().position(|v| v.path == curr) {
                    let prev_pos = if pos == 0 {
                        pool.len().saturating_sub(1)
                    } else {
                        pos - 1
                    };
                    return Some(pool[prev_pos].clone());
                }
            }
            pool.last().cloned()
        }
    }

    pub(crate) fn get_next_video_rec_with_mode(
        &self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        Self::get_next_video_rec_from_pool(&self.available_videos, current_path, is_shuffle)
    }

    pub fn get_next_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
        self.get_next_video_rec_with_mode(current_path, self.default_shuffle_mode)
    }

    pub fn get_prev_video_rec_with_mode(
        &self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        Self::get_prev_video_rec_from_pool(&self.available_videos, current_path, is_shuffle)
    }

    pub fn get_next_video_rec_for_navigation(
        &mut self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if let Some(filtered) = self.file_picker_filtered_videos() {
            return Self::get_next_video_rec_from_pool(&filtered, current_path, is_shuffle);
        }
        self.get_next_video_rec_with_mode(current_path, is_shuffle)
    }

    pub fn get_prev_video_rec_for_navigation(
        &mut self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if let Some(filtered) = self.file_picker_filtered_videos() {
            return Self::get_prev_video_rec_from_pool(&filtered, current_path, is_shuffle);
        }
        self.get_prev_video_rec_with_mode(current_path, is_shuffle)
    }

    pub(crate) fn record_current_player_nav_position(&mut self, id: PlayerId) {
        if let Some(p) = self.players.player_mut(id) {
            let pos_secs = p.state.position.as_secs_f64();
            let dur_secs = p.state.duration.as_secs_f64();
            let saved_pos = if dur_secs > 10.0 && (dur_secs - pos_secs) < 3.0 {
                None
            } else if pos_secs > 0.5 {
                Some(pos_secs)
            } else {
                None
            };
            let curr_path = p.state.path.clone();
            if let Some(top) = p.nav_history.back_stack.last_mut() {
                if top.path == curr_path {
                    top.position_secs = saved_pos;
                }
            }
        }
    }

    pub fn record_play_history(&mut self, path: &str) {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.drawers.play_history.last().map(|e| e.path.as_str()) == Some(trimmed) {
            return;
        }
        let title = format::format_video_title(trimmed);
        let folder = format::format_video_folder(trimmed);
        self.drawers.play_history.push(PlayHistoryItem {
            path: trimmed.to_string(),
            title,
            folder,
        });
        if self.drawers.play_history.len() > MAX_PLAY_HISTORY_ENTRIES {
            self.drawers.play_history.remove(0);
        }
    }

    pub fn push_player_nav_entry(&mut self, id: PlayerId, path: String, pos: Option<f64>) {
        self.record_play_history(&path);
        if let Some(player) = self.players.player_mut(id) {
            let hist = &mut player.nav_history;
            if hist.back_stack.last().map(|e| &e.path) != Some(&path) {
                hist.back_stack.push(PlaybackHistoryEntry {
                    path,
                    position_secs: pos,
                });
                if hist.back_stack.len() > MAX_PLAYER_NAV_HISTORY_ENTRIES {
                    hist.back_stack.remove(0);
                }
            }
        }
    }
}
