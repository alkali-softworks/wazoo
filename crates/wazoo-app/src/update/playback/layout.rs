/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Layout, Multi-Player Grid, Shuffle & Scroll/Flip Mode Handlers
 */

use crate::app::{DEFAULT_TOAST_SECS, FOCUS_BORDER_TICKS, SHORT_TOAST_SECS, WazooApp};
use crate::message::Message;
use crate::state::AppPlayer;
use iced::Task;
use std::sync::{Arc, Mutex};
use wazoo_core::{LayoutMode, PlaybackMode};
use wazoo_media::{PlayerId, VideoHandle};

impl WazooApp {
    fn exit_cubes_if_active(&mut self) -> Task<Message> {
        if self.cube.desktop_overlay {
            self.dismiss_all_cubes()
        } else {
            Task::none()
        }
    }

    fn exit_scroll_and_restore_grid(&mut self) -> Task<Message> {
        if self.settings.playback_mode == PlaybackMode::Scroll {
            self.cleanup_scroll_mode();
            self.restore_grid_players_to_target(self.settings.player_count)
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_toggle_player_shuffle(&mut self, id: PlayerId) -> Task<Message> {
        let mut new_mode = self.default_shuffle_mode;
        if let Some(p) = self.players.player_mut(id) {
            p.shuffle = !p.shuffle;
            new_mode = p.shuffle;
            p.nav_history.back_stack.clear();
            p.nav_history.forward_stack.clear();
        }
        if self.focused_player_id() == Some(id) {
            self.default_shuffle_mode = new_mode;
        }
        let msg = if new_mode {
            self.t("player.switched_shuffle")
        } else {
            self.t("player.switched_sequential")
        };
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        self.save_session_state();
        Task::none()
    }

    pub(super) fn handle_toggle_shuffle_mode(&mut self) -> Task<Message> {
        if let Some(id) = self.focused_player_id() {
            return self.update(Message::TogglePlayerShuffle(id));
        }
        self.default_shuffle_mode = !self.default_shuffle_mode;
        for p in &mut self.players {
            p.nav_history.back_stack.clear();
            p.nav_history.forward_stack.clear();
        }
        let msg = if self.default_shuffle_mode {
            self.t("player.switched_shuffle")
        } else {
            self.t("player.switched_sequential")
        };
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        self.save_session_state();
        Task::none()
    }

    pub(super) fn handle_cycle_layout(&mut self) -> Task<Message> {
        self.titlebar.show_dropdown_menu = false;
        self.modals.menu = false;
        let exit_task = self.exit_cubes_if_active();
        let restore_task = self.exit_scroll_and_restore_grid();
        self.settings.layout = match self.settings.layout {
            LayoutMode::Grid => LayoutMode::Row,
            LayoutMode::Row => LayoutMode::Column,
            LayoutMode::Column => LayoutMode::Grid,
        };
        let msg = match self.settings.layout {
            LayoutMode::Grid => self.t("wazoo.layout_grid"),
            LayoutMode::Row => self.t("wazoo.layout_row"),
            LayoutMode::Column => self.t("wazoo.layout_column"),
        };
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        let _ = self.config_mgr.save_settings(&self.settings);
        Task::batch([exit_task, restore_task])
    }

    pub(super) fn handle_set_player_count(&mut self, count: usize) -> Task<Message> {
        let exit_task = self.exit_cubes_if_active();
        let target = count.clamp(1, 12);
        if self.settings.playback_mode == PlaybackMode::Scroll {
            self.cleanup_scroll_mode();
        }

        self.settings.player_count = target;
        let restore_task = self.restore_grid_players_to_target(target);

        if self.settings.playback_mode == PlaybackMode::Flip {
            self.stagger_flip_countdowns();
        }

        let msg = self.t_with(
            "wazoo.set_players_count",
            &[
                ("count", &target.to_string()),
                ("suffix", if target == 1 { "" } else { "s" }),
            ],
        );
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        self.save_session_state();
        Task::batch([exit_task, restore_task])
    }

    pub(super) fn handle_toggle_scroll_mode(&mut self) -> Task<Message> {
        self.modals.menu = false;
        self.titlebar.show_dropdown_menu = false;
        let exit_task = self.exit_cubes_if_active();
        if self.settings.playback_mode != PlaybackMode::Scroll {
            self.enter_scroll_mode(exit_task)
        } else {
            self.cleanup_scroll_mode();
            let restore_task = self.restore_grid_players_to_target(self.settings.player_count);
            self.show_toast_for(self.t("wazoo.scroll_mode_disabled"), DEFAULT_TOAST_SECS);
            let _ = self.config_mgr.save_settings(&self.settings);
            Task::batch([exit_task, restore_task])
        }
    }

    fn enter_scroll_mode(&mut self, exit_task: Task<Message>) -> Task<Message> {
        self.save_session_state();
        self.settings.playback_mode = PlaybackMode::Scroll;
        self.scroll_engine.scroll_mode_muted = self.settings.scroll_mode_muted;
        let window_w = self.settings.window_bounds.width as f32;
        let window_h = self.settings.window_bounds.height as f32;
        self.scroll_engine.set_window_size(window_w, window_h);

        let grid_indices: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| if !p.is_cube { Some(i) } else { None })
            .collect();

        let mut scroll_tasks = Vec::new();
        if grid_indices.is_empty() {
            let (_, task) = self.add_player_internal();
            scroll_tasks.push(task);
        } else {
            let keep_grid_idx = if !self.players.is_empty()
                && self.focused_idx < self.players.len()
                && !self.players[self.focused_idx].is_cube
            {
                self.focused_idx
            } else {
                grid_indices[0]
            };
            let keep_pid = self.players[keep_grid_idx].id;

            let mut i = self.players.len();
            while i > 0 {
                i -= 1;
                if !self.players[i].is_cube && self.players[i].id != keep_pid {
                    let mut p = self.players.remove(i);
                    p.stop();
                }
            }

            if let Some(pos) = self.players.player_index(keep_pid) {
                if self.focused_player().map(|p| !p.is_cube).unwrap_or(true) {
                    self.focused_idx = pos;
                }
            }
        }

        let items_with_heights: Vec<(PlayerId, f32)> = self
            .players
            .iter()
            .filter(|p| !p.is_cube)
            .map(|p| (p.id, self.calculate_player_scroll_height(p)))
            .collect();
        self.scroll_engine
            .init_stack_with_heights(&items_with_heights);

        while let Some(spawn_y) = self.scroll_engine.needs_new_player() {
            let (added, task) = self.add_player_internal();
            if let Some(id) = added {
                scroll_tasks.push(task);
                let item_h = self
                    .players
                    .player(id)
                    .map(|p| self.calculate_player_scroll_height(p))
                    .unwrap_or_else(|| self.scroll_engine.default_item_height());
                self.scroll_engine.add_item(id, spawn_y, item_h);
            } else {
                break;
            }
        }

        for p in &mut self.players {
            if !p.is_cube {
                p.set_muted(self.settings.scroll_mode_muted);
                let vol = self.scroll_engine.calculate_player_volume(p.id);
                p.set_crossfade_volume(vol);
            }
        }

        self.show_toast_for(self.t("wazoo.scroll_mode_enabled"), DEFAULT_TOAST_SECS);
        let _ = self.config_mgr.save_settings(&self.settings);

        scroll_tasks.push(exit_task);
        scroll_tasks.push(self.trigger_preload_task());
        Task::batch(scroll_tasks)
    }

    pub(super) fn handle_toggle_flip_mode(&mut self) -> Task<Message> {
        let restore_task = self.exit_scroll_and_restore_grid();
        self.settings.playback_mode = match self.settings.playback_mode {
            PlaybackMode::Flip => PlaybackMode::Normal,
            _ => PlaybackMode::Flip,
        };
        if self.settings.playback_mode == PlaybackMode::Flip {
            self.stagger_flip_countdowns();
        }
        let msg = match self.settings.playback_mode {
            PlaybackMode::Flip => self.t("wazoo.flip_mode_enabled"),
            _ => self.t("wazoo.flip_mode_disabled"),
        };
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        restore_task
    }

    pub(super) fn handle_set_scroll_speed(&mut self, speed: f32) -> Task<Message> {
        self.settings.scroll_speed = speed.clamp(0.1, 10.0);
        self.scroll_engine.set_speed(self.settings.scroll_speed);
        let speed_str = format!("{:.1}", self.settings.scroll_speed);
        let msg = self.t_with("player.scroll_speed", &[("speed", &speed_str)]);
        self.show_toast_for(msg, SHORT_TOAST_SECS);
        Task::none()
    }

    pub(super) fn handle_preloaded_player_ready(
        &mut self,
        holder: Arc<Mutex<Option<Result<VideoHandle, String>>>>,
    ) -> Task<Message> {
        self.is_preloading = false;
        if self.settings.playback_mode != PlaybackMode::Scroll {
            if let Some(Ok(handle)) = holder.lock().ok().and_then(|mut g| g.take()) {
                std::thread::spawn(move || drop(handle));
            }
            return Task::none();
        }

        let result = match holder.lock().ok().and_then(|mut g| g.take()) {
            Some(r) => r,
            None => return Task::none(),
        };

        match result {
            Ok(mut handle) => {
                handle.set_subtitles_visible(self.subtitles_enabled);
                handle.set_muted(self.settings.scroll_mode_muted);
                let item_h = self.calculate_player_scroll_height(&handle);
                let margin = self.scroll_engine.default_item_height() * 1.5;
                if let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(margin) {
                    self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                    let vol = self.scroll_engine.calculate_player_volume(handle.id);
                    handle.set_crossfade_volume(vol);
                    handle.set_paused(false);
                    self.record_play_history(&handle.state.path);
                    self.players
                        .push(AppPlayer::new(handle, self.default_shuffle_mode));
                    return self.trigger_preload_task();
                } else {
                    self.preloaded_player = Some(handle);
                }
            }
            Err(err) => {
                log::error!("Background player preload failed: {err}");
            }
        }
        Task::none()
    }

    pub(super) fn handle_add_new_player(&mut self) -> Task<Message> {
        self.titlebar.show_dropdown_menu = false;
        self.modals.menu = false;
        let prev_grid_count = self.players.iter().filter(|p| !p.is_cube).count();
        if prev_grid_count >= 12 {
            return Task::none();
        }
        let new_count = prev_grid_count + 1;
        let task = self.update(Message::SetPlayerCount(new_count));
        let new_grid_count = self.players.iter().filter(|p| !p.is_cube).count();
        if new_grid_count > prev_grid_count {
            if let Some(new_idx) = self.players.iter().rposition(|p| !p.is_cube) {
                let focus_task = self.update(Message::SetFocusedPlayer(new_idx));
                self.overlay.focus_border_ticks = 0;
                return Task::batch([task, focus_task]);
            }
        }
        task
    }

    pub(super) fn handle_remove_focused_player(&mut self) -> Task<Message> {
        if let Some(player) = self.focused_player() {
            if player.is_cube {
                return self.update(Message::RemoveCube);
            }
        }
        let restore_task = self.exit_scroll_and_restore_grid();
        let exit_task = self.exit_cubes_if_active();
        if self.players.iter().filter(|p| !p.is_cube).count() <= 1 {
            return Task::batch([exit_task, restore_task]);
        }
        if let Some(id) = self.focused_player_id() {
            self.players.retain(|p| p.id != id);
            if self.hovered_player_id == Some(id) {
                self.hovered_player_id = None;
            }
            if self.open_audio_menu_id == Some(id) {
                self.open_audio_menu_id = None;
            }
            let grid_count = self.players.iter().filter(|p| !p.is_cube).count();
            self.settings.player_count = grid_count;
            if self.focused_idx >= self.players.len() && !self.players.is_empty() {
                self.focused_idx = self.players.len() - 1;
            }
            let count_str = grid_count.to_string();
            let msg = self.t_with("player.players_count", &[("count", &count_str)]);
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
            let _ = self.config_mgr.save_settings(&self.settings);
            if self.drawers.show_transcript {
                return Task::batch([
                    exit_task,
                    restore_task,
                    self.load_transcript_for_focused_player(),
                ]);
            }
            return Task::batch([exit_task, restore_task]);
        }
        Task::batch([exit_task, restore_task])
    }

    pub(super) fn handle_cycle_focused_player(&mut self) -> Task<Message> {
        if !self.players.is_empty() {
            let next_idx = (self.focused_idx + 1) % self.players.len();
            if next_idx != self.focused_idx {
                self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
            }
            self.focused_idx = next_idx;
            let idx_str = (self.focused_idx + 1).to_string();
            let toast = self.t_with("player.focused_player", &[("index", &idx_str)]);
            self.show_toast_for(toast, SHORT_TOAST_SECS);
            if self.drawers.show_transcript {
                return self.load_transcript_for_focused_player();
            }
        }
        Task::none()
    }

    pub(super) fn handle_set_focused_player(&mut self, idx: usize) -> Task<Message> {
        if idx < self.players.len() {
            let was_already_active = self.focused_idx == idx;
            self.focused_idx = idx;
            if !was_already_active {
                self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                if self.drawers.show_transcript {
                    return self.load_transcript_for_focused_player();
                }
            }
        }
        Task::none()
    }

    /// Restores regular (non-cube) player count to the target count without affecting active 3D cube players.
    pub(crate) fn restore_grid_players_to_target(&mut self, target: usize) -> Task<Message> {
        let target_count = target.clamp(1, 12);
        let grid_count = self.players.iter().filter(|p| !p.is_cube).count();
        let mut tasks = Vec::new();
        if grid_count > target_count {
            let mut remove_needed = grid_count - target_count;
            let mut i = self.players.len();
            while i > 0 && remove_needed > 0 {
                i -= 1;
                if !self.players[i].is_cube {
                    let mut p = self.players.remove(i);
                    p.stop();
                    remove_needed -= 1;
                }
            }
        } else if grid_count < target_count {
            while self.players.iter().filter(|p| !p.is_cube).count() < target_count {
                let (added, task) = self.add_player_internal();
                if added.is_none() {
                    break;
                }
                tasks.push(task);
            }
        }
        if self.focused_idx >= self.players.len() && !self.players.is_empty() {
            self.focused_idx = self.players.len() - 1;
        }
        Task::batch(tasks)
    }
}
