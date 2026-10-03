/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Playback, Layout & Multi-Player Reducer
 *
 * Implements message handlers for play/pause, seeking, frame stepping, volume/mute,
 * audio track selection, shuffle modes, layout switching, player count changes,
 * scroll/flip mode toggling, and bookmarks.
 */

use crate::app::{
    DEFAULT_TOAST_SECS, FOCUS_BORDER_TICKS, LONG_TOAST_SECS, PLAYER_OVERLAY_FADE_TICKS,
    SHORT_TOAST_SECS, WazooApp,
};
use crate::format;
use crate::message::Message;
use crate::state::AppPlayer;
use iced::Task;
use std::time::Duration;
use wazoo_core::{Bookmark, LayoutMode, PlaybackMode};
use wazoo_media::PlayerId;

impl WazooApp {
    pub(crate) fn update_playback(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayerClicked(id) => {
                if self.open_audio_menu_id.is_some() {
                    self.open_audio_menu_id = None;
                }
                if self.is_modal_or_menu_open() {
                    return Task::none();
                }
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                        if self.drawers.show_transcript {
                            return self.load_transcript_for_focused_player();
                        }
                    }
                    self.trigger_player_overlay();
                }
                Task::none()
            }
            Message::TogglePlay(id) => {
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                if let Some(p) = self.players.player_mut(id) {
                    p.toggle_play();
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::TogglePlayFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlay(id));
                }
                Task::none()
            }
            Message::NextVideo(id) => self.advance_player_to_next_video(id, true),
            Message::AutoAdvanceVideo(id) => self.advance_player_to_next_video(id, false),
            Message::NextVideoFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::NextVideo(id));
                }
                Task::none()
            }
            Message::PrevVideo(id) => self.advance_player_to_prev_video(id, true),
            Message::PrevVideoFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::PrevVideo(id));
                }
                Task::none()
            }
            Message::MarkInFocused => {
                if let Some(p) = self.focused_player_mut() {
                    let pos = p.position();
                    let id = p.id;
                    p.set_mark_in(Some(pos));
                    let time_str = format::format_time_str(pos.as_secs_f64());
                    let id_str = id.to_string();
                    if let (Some(i), Some(o)) = (p.mark_in(), p.mark_out()) {
                        let start_str = format::format_time_str(i.as_secs_f64());
                        let end_str = format::format_time_str(o.as_secs_f64());
                        self.overlay.toast_message = Some(self.t_with(
                            "player.loop_range",
                            &[
                                ("player", &id_str),
                                ("start", &start_str),
                                ("end", &end_str),
                            ],
                        ));
                    } else {
                        self.overlay.toast_message = Some(self.t_with(
                            "player.mark_in_set",
                            &[("player", &id_str), ("time", &time_str)],
                        ));
                    }
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::MarkOutFocused => {
                if let Some(p) = self.focused_player_mut() {
                    let pos = p.position();
                    let id = p.id;
                    p.set_mark_out(Some(pos));
                    let time_str = format::format_time_str(pos.as_secs_f64());
                    let id_str = id.to_string();
                    if let (Some(i), Some(o)) = (p.mark_in(), p.mark_out()) {
                        let start_str = format::format_time_str(i.as_secs_f64());
                        let end_str = format::format_time_str(o.as_secs_f64());
                        self.overlay.toast_message = Some(self.t_with(
                            "player.loop_range",
                            &[
                                ("player", &id_str),
                                ("start", &start_str),
                                ("end", &end_str),
                            ],
                        ));
                    } else {
                        self.overlay.toast_message = Some(self.t_with(
                            "player.mark_out_set",
                            &[("player", &id_str), ("time", &time_str)],
                        ));
                    }
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ClearMarkInFocused => {
                if let Some(p) = self.focused_player_mut() {
                    let id = p.id;
                    let id_str = id.to_string();
                    p.clear_mark_in();
                    if let Some(o) = p.mark_out() {
                        let time_str = format::format_time_str(o.as_secs_f64());
                        self.overlay.toast_message = Some(self.t_with(
                            "player.mark_out_set",
                            &[("player", &id_str), ("time", &time_str)],
                        ));
                    } else {
                        self.overlay.toast_message =
                            Some(self.t_with("player.loop_cleared", &[("player", &id_str)]));
                    }
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ClearMarkOutFocused => {
                if let Some(p) = self.focused_player_mut() {
                    let id = p.id;
                    let id_str = id.to_string();
                    p.clear_mark_out();
                    if let Some(i) = p.mark_in() {
                        let time_str = format::format_time_str(i.as_secs_f64());
                        self.overlay.toast_message = Some(self.t_with(
                            "player.mark_in_set",
                            &[("player", &id_str), ("time", &time_str)],
                        ));
                    } else {
                        self.overlay.toast_message =
                            Some(self.t_with("player.loop_cleared", &[("player", &id_str)]));
                    }
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ClearLoop(id) => {
                if let Some(p) = self.players.player_mut(id) {
                    p.clear_marks();
                    let id_str = id.to_string();
                    self.overlay.toast_message =
                        Some(self.t_with("player.loop_cleared", &[("player", &id_str)]));
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ClearLoopFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::ClearLoop(id));
                }
                Task::none()
            }
            Message::Seek(id, pos) => {
                if let Some(pos_idx) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos_idx;
                    self.focused_idx = pos_idx;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                if let Some(p) = self.players.player_mut(id) {
                    p.seek(pos);
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::SeekRatio(id, ratio) => {
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                if let Some(p) = self.players.player_mut(id) {
                    let dur = p.duration();
                    if dur > Duration::ZERO {
                        let target_secs = dur.as_secs_f64() * (ratio.clamp(0.0, 1.0) as f64);
                        let target = Duration::from_secs_f64(target_secs);
                        p.seek(target);

                        let pos_s = target.as_secs_f64();
                        let dur_s = dur.as_secs_f64();
                        let pos_str = format::format_time_str(pos_s);
                        let dur_str = format::format_time_str(dur_s);
                        self.overlay.toast_message = Some(self.t_with(
                            "player.seek_position",
                            &[("pos", &pos_str), ("dur", &dur_str)],
                        ));
                        self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    }
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::PlayerHovered(id) => {
                if self.window.is_resizing()
                    || self.is_modal_or_menu_open()
                    || (self.titlebar.show
                        && self.is_point_in_titlebar(self.window.cursor_position))
                {
                    if self.hovered_player_id == Some(id) {
                        let current_fade =
                            self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.overlay.ticks = self.overlay.ticks.min(current_fade);
                    }
                    return Task::none();
                }
                self.hovered_player_id = Some(id);
                self.trigger_player_overlay();
                Task::none()
            }
            Message::PlayerUnhovered(id) => {
                if self.window.is_resizing() || self.open_audio_menu_id == Some(id) {
                    return Task::none();
                }
                if self.hovered_player_id == Some(id) {
                    let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                    self.overlay.ticks = self.overlay.ticks.min(current_fade);
                }
                Task::none()
            }
            Message::SeekRelativeFocused(secs) => {
                self.trigger_player_overlay();
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player_mut(id) {
                        p.seek_relative(secs);
                        let pos = p.position();
                        let dur = p.duration();
                        let sign = if secs > 0.0 { "+" } else { "" };
                        let pos_str = format::format_time_str(pos.as_secs_f64());
                        let dur_str = format::format_time_str(dur.as_secs_f64());
                        let secs_str = format!("{:.0}", secs);
                        self.overlay.toast_message = Some(self.t_with(
                            "player.seek_relative",
                            &[
                                ("sign", sign),
                                ("secs", &secs_str),
                                ("pos", &pos_str),
                                ("dur", &dur_str),
                            ],
                        ));
                        self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    }
                }
                Task::none()
            }
            Message::StepFrameForwardFocused => {
                self.trigger_player_overlay();
                if let Some(p) = self.focused_player_mut() {
                    p.step_frame_forward();
                }
                Task::none()
            }
            Message::StepFrameBackwardFocused => {
                self.trigger_player_overlay();
                if let Some(p) = self.focused_player_mut() {
                    p.step_frame_backward();
                }
                Task::none()
            }
            Message::SetVolume(id, vol) => {
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                if let Some(p) = self.players.player_mut(id) {
                    p.set_volume(vol);
                    if vol > 0.0 {
                        if p.state.is_muted {
                            p.set_muted(false);
                        }
                    } else {
                        p.set_muted(true);
                    }
                }
                self.save_session_state();
                self.trigger_player_overlay();
                Task::none()
            }
            Message::AdjustVolumeFocused(delta) => {
                if self.settings.playback_mode == PlaybackMode::Scroll && delta > 0.0 {
                    return self.update(Message::GlobalUnmute);
                }
                if let Some(id) = self.focused_player_id() {
                    let mut vol_display = None;
                    if let Some(p) = self.players.player_mut(id) {
                        p.adjust_volume(delta);
                        if p.state.volume > 0.0 {
                            if p.state.is_muted {
                                p.set_muted(false);
                            }
                        } else {
                            p.set_muted(true);
                        }
                        vol_display = Some(p.state.volume);
                    }
                    if let Some(v) = vol_display {
                        let pct_str = format!("{:.0}", v * 100.0);
                        self.overlay.toast_message =
                            Some(self.t_with("player.volume", &[("percent", &pct_str)]));
                        self.overlay.toast_time_remaining = SHORT_TOAST_SECS;
                        self.save_session_state();
                    }
                }
                Task::none()
            }
            Message::SelectAudioTrack(id, track_id) => {
                self.open_audio_menu_id = None;
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                let mut selected_pref = None;
                let mut track_label = None;
                if let Some(p) = self.players.player_mut(id) {
                    p.set_audio_track(track_id);
                    if let Some(track) = p.state.audio_tracks.iter().find(|t| t.id == track_id) {
                        track_label = Some(wazoo_media::format_audio_track_label(track, 0));
                        selected_pref = Some(wazoo_media::get_track_preference_string(track));
                    }
                }
                if let Some(label) = track_label {
                    self.overlay.toast_message =
                        Some(self.t_with("player.audio_track", &[("label", &label)]));
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                if let Some(pref) = selected_pref {
                    if !pref.starts_with("Track ") {
                        self.settings.preferred_audio_language = Some(pref.clone());
                        let _ = self.config_mgr.save_settings(&self.settings);
                        for player in &mut self.players {
                            player.set_preferred_audio_language(Some(pref.clone()));
                        }
                    }
                }
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ToggleAudioMenu(id) => {
                if self.open_audio_menu_id == Some(id) {
                    self.open_audio_menu_id = None;
                } else {
                    self.open_audio_menu_id = Some(id);
                    self.hovered_player_id = Some(id);
                    self.trigger_player_overlay();
                }
                Task::none()
            }
            Message::CloseAudioMenu => {
                self.open_audio_menu_id = None;
                Task::none()
            }
            Message::TogglePlayerMute(id) => {
                if let Some(pos) = self.players.player_index(id) {
                    let was_already_active = self.focused_idx == pos;
                    self.focused_idx = pos;
                    if !was_already_active {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                }
                if let Some(p) = self.players.player_mut(id) {
                    let muted = !p.state.is_muted;
                    p.set_muted(muted);
                    self.overlay.toast_message = Some(if muted {
                        self.t("player.muted")
                    } else {
                        self.t("player.unmuted")
                    });
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                self.save_session_state();
                self.trigger_player_overlay();
                Task::none()
            }
            Message::ToggleMuteFocused => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    return self.update(Message::ToggleGlobalMute);
                }
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlayerMute(id));
                }
                Task::none()
            }
            Message::ToggleGlobalMute => {
                self.settings.scroll_mode_muted = !self.settings.scroll_mode_muted;
                self.scroll_engine.scroll_mode_muted = self.settings.scroll_mode_muted;
                for p in &mut self.players {
                    p.set_muted(self.settings.scroll_mode_muted);
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }
                }
                if let Some(ref mut preloaded) = self.preloaded_player {
                    preloaded.set_muted(self.settings.scroll_mode_muted);
                }
                self.overlay.toast_message = Some(if self.settings.scroll_mode_muted {
                    self.t("wazoo.global_mode_muted")
                } else {
                    self.t("wazoo.global_mode_unmuted")
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::GlobalUnmute => {
                self.settings.scroll_mode_muted = false;
                self.scroll_engine.scroll_mode_muted = false;
                for p in &mut self.players {
                    p.set_muted(false);
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }
                }
                if let Some(ref mut preloaded) = self.preloaded_player {
                    preloaded.set_muted(false);
                }
                self.overlay.toast_message = Some(self.t("wazoo.global_mode_unmuted"));
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::TogglePlayerShuffle(id) => {
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
                self.overlay.toast_message = Some(if new_mode {
                    self.t("player.switched_shuffle")
                } else {
                    self.t("player.switched_sequential")
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                self.save_session_state();
                Task::none()
            }
            Message::ToggleShuffleMode => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlayerShuffle(id));
                } else {
                    self.default_shuffle_mode = !self.default_shuffle_mode;
                    for p in &mut self.players {
                        p.nav_history.back_stack.clear();
                        p.nav_history.forward_stack.clear();
                    }
                    self.overlay.toast_message = Some(if self.default_shuffle_mode {
                        self.t("player.switched_shuffle")
                    } else {
                        self.t("player.switched_sequential")
                    });
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    self.save_session_state();
                }
                Task::none()
            }
            Message::CycleLayout => {
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                let exit_task = if self.cube.desktop_overlay {
                    self.dismiss_all_cubes()
                } else {
                    Task::none()
                };
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    self.restore_grid_players_to_target(self.settings.player_count);
                }
                self.settings.layout = match self.settings.layout {
                    LayoutMode::Grid => LayoutMode::Row,
                    LayoutMode::Row => LayoutMode::Column,
                    LayoutMode::Column => LayoutMode::Grid,
                };
                self.overlay.toast_message = Some(match self.settings.layout {
                    LayoutMode::Grid => self.t("wazoo.layout_grid"),
                    LayoutMode::Row => self.t("wazoo.layout_row"),
                    LayoutMode::Column => self.t("wazoo.layout_column"),
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                let _ = self.config_mgr.save_settings(&self.settings);
                exit_task
            }
            Message::SetPlayerCount(count) => {
                let exit_task = if self.cube.desktop_overlay {
                    self.dismiss_all_cubes()
                } else {
                    Task::none()
                };
                let target = count.clamp(1, 12);
                let was_scroll = self.settings.playback_mode == PlaybackMode::Scroll;
                if was_scroll {
                    self.cleanup_scroll_mode();
                }

                self.settings.player_count = target;
                self.restore_grid_players_to_target(target);

                if self.settings.playback_mode == PlaybackMode::Flip {
                    self.stagger_flip_countdowns();
                }

                self.overlay.toast_message = Some(self.t_with(
                    "wazoo.set_players_count",
                    &[
                        ("count", &target.to_string()),
                        ("suffix", if target == 1 { "" } else { "s" }),
                    ],
                ));
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                let _ = self.config_mgr.save_settings(&self.settings);
                self.save_session_state();
                exit_task
            }
            Message::ToggleScrollMode => {
                self.modals.menu = false;
                self.titlebar.show_dropdown_menu = false;
                let exit_task = if self.cube.desktop_overlay {
                    self.dismiss_all_cubes()
                } else {
                    Task::none()
                };
                if self.settings.playback_mode != PlaybackMode::Scroll {
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

                    if grid_indices.is_empty() {
                        self.add_player_internal();
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
                        if let Some(id) = self.add_player_internal() {
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
                            p.set_volume(vol);
                        }
                    }

                    self.overlay.toast_message = Some(self.t("wazoo.scroll_mode_enabled"));
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    let _ = self.config_mgr.save_settings(&self.settings);

                    return Task::batch([exit_task, self.trigger_preload_task()]);
                } else {
                    self.cleanup_scroll_mode();
                    self.restore_grid_players_to_target(self.settings.player_count);
                    self.overlay.toast_message = Some(self.t("wazoo.scroll_mode_disabled"));
                }
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                let _ = self.config_mgr.save_settings(&self.settings);
                exit_task
            }
            Message::ToggleFlipMode => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    self.restore_grid_players_to_target(self.settings.player_count);
                }
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Flip => PlaybackMode::Normal,
                    _ => PlaybackMode::Flip,
                };
                if self.settings.playback_mode == PlaybackMode::Flip {
                    self.stagger_flip_countdowns();
                }
                self.overlay.toast_message = Some(match self.settings.playback_mode {
                    PlaybackMode::Flip => self.t("wazoo.flip_mode_enabled"),
                    _ => self.t("wazoo.flip_mode_disabled"),
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                Task::none()
            }
            Message::SetScrollSpeed(speed) => {
                self.settings.scroll_speed = speed.clamp(0.1, 10.0);
                self.scroll_engine.set_speed(self.settings.scroll_speed);
                let speed_str = format!("{:.1}", self.settings.scroll_speed);
                self.overlay.toast_message =
                    Some(self.t_with("player.scroll_speed", &[("speed", &speed_str)]));
                self.overlay.toast_time_remaining = SHORT_TOAST_SECS;
                Task::none()
            }
            Message::AdjustScrollSpeed(delta) => {
                let new_speed = (self.settings.scroll_speed + delta).clamp(0.1, 10.0);
                self.update(Message::SetScrollSpeed(new_speed))
            }
            Message::PreloadedPlayerReady(holder) => {
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
                        if let Some(spawn_y) =
                            self.scroll_engine.needs_new_player_with_margin(margin)
                        {
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
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
            Message::AddNewPlayer => {
                let prev_len = self.players.len();
                let new_count = (prev_len + 1).min(12);
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                let task = self.update(Message::SetPlayerCount(new_count));
                if self.players.len() > prev_len {
                    let new_idx = self.players.len() - 1;
                    let focus_task = self.update(Message::SetFocusedPlayer(new_idx));
                    self.overlay.focus_border_ticks = 0;
                    return Task::batch([task, focus_task]);
                }
                task
            }
            Message::RemoveFocusedPlayer => {
                if let Some(player) = self.focused_player() {
                    if player.is_cube {
                        return self.update(Message::ClearCubes);
                    }
                }
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    self.restore_grid_players_to_target(self.settings.player_count);
                }
                let exit_task = if self.cube.desktop_overlay {
                    self.dismiss_all_cubes()
                } else {
                    Task::none()
                };
                if self.players.iter().filter(|p| !p.is_cube).count() <= 1 {
                    return exit_task;
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
                    self.overlay.toast_message =
                        Some(self.t_with("player.players_count", &[("count", &count_str)]));
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    let _ = self.config_mgr.save_settings(&self.settings);
                    if self.drawers.show_transcript {
                        return Task::batch([exit_task, self.load_transcript_for_focused_player()]);
                    }
                    return exit_task;
                }
                exit_task
            }
            Message::CycleFocusedPlayer => {
                if !self.players.is_empty() {
                    let next_idx = (self.focused_idx + 1) % self.players.len();
                    if next_idx != self.focused_idx {
                        self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
                    }
                    self.focused_idx = next_idx;
                    let target_player = &self.players[self.focused_idx];
                    let toast = if target_player.is_cube {
                        "Focused: 3D Video Cube".to_string()
                    } else {
                        let grid_idx = self
                            .players
                            .iter()
                            .take(self.focused_idx + 1)
                            .filter(|p| !p.is_cube)
                            .count();
                        self.t_with("player.focused_player", &[("index", &grid_idx.to_string())])
                    };
                    self.overlay.toast_message = Some(toast);
                    self.overlay.toast_time_remaining = SHORT_TOAST_SECS;
                    if self.drawers.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::SetFocusedPlayer(idx) => {
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
            Message::AddBookmarkFocused => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player(id) {
                        let path = p.path().to_string();
                        if !path.is_empty() {
                            let name = format::format_descriptive_title(&path);
                            let query = self.search.active_query.clone();
                            let position_secs = p.position().as_secs_f64();
                            let is_shuffle = self.is_player_shuffle(id);

                            if let Some(existing) =
                                self.settings.bookmarks.iter_mut().find(|b| b.path == path)
                            {
                                existing.name = name.clone();
                                existing.query = query;
                                existing.position_secs = position_secs;
                                existing.is_shuffle = is_shuffle;
                                self.overlay.toast_message =
                                    Some(self.t_with("bookmarks.updated", &[("name", &name)]));
                            } else {
                                self.settings.bookmarks.push(Bookmark {
                                    name: name.clone(),
                                    query,
                                    path,
                                    position_secs,
                                    is_shuffle,
                                });
                                self.overlay.toast_message =
                                    Some(self.t_with("bookmarks.added", &[("name", &name)]));
                            }
                            let _ = self.config_mgr.save_settings(&self.settings);
                            self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        }
                    }
                }
                Task::none()
            }
            Message::RemoveBookmarkFocused => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player(id) {
                        let path = p.path();
                        if let Some(pos) =
                            self.settings.bookmarks.iter().position(|b| b.path == path)
                        {
                            let removed = self.settings.bookmarks.remove(pos);
                            let _ = self.config_mgr.save_settings(&self.settings);
                            self.overlay.toast_message =
                                Some(self.t_with("bookmarks.removed", &[("name", &removed.name)]));
                            self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        } else {
                            self.overlay.toast_message = Some(self.t("bookmarks.not_found"));
                            self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                        }
                    }
                }
                Task::none()
            }
            Message::RemoveBookmark(idx) => {
                if idx < self.settings.bookmarks.len() {
                    let removed = self.settings.bookmarks.remove(idx);
                    let _ = self.config_mgr.save_settings(&self.settings);
                    self.overlay.toast_message =
                        Some(self.t_with("bookmarks.removed", &[("name", &removed.name)]));
                    self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                }
                Task::none()
            }
            Message::JumpToBookmark(b) => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player_mut(id) {
                        p.flip.reset(self.settings.flip_interval_secs);
                    }
                }
                // 1. Restore shuffle vs linear mode
                self.default_shuffle_mode = b.is_shuffle;
                if let Some(p) = self.focused_player_mut() {
                    p.shuffle = b.is_shuffle;
                }

                // 2. Update the global search query to match the bookmark's query
                self.search.active_query = b.query.clone();
                self.search.tags = b
                    .query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.search.input.clear();
                self.settings.last_query = b.query.clone();
                self.settings.last_folders = self.search.active_folders.clone();
                let _ = self.config_mgr.save_settings(&self.settings);

                // 3. Query the database using the updated search query
                let folders = self.search.active_folders.clone();

                if let Ok(results) = self.db.search_videos(&self.search.active_query, &folders) {
                    self.last_total_videos = results.len();
                    self.available_videos = results;
                    self.drawers.file_picker_entries.clear();
                    self.apply_file_picker_search();
                }

                // 4. Load the bookmarked video and position in the focused player
                let focused_id = if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player_mut(id) {
                        p.start_loading();
                        p.nav_history.forward_stack.clear();
                    }
                    self.record_current_player_nav_position(id);
                    let title = format::format_video_title(&b.path);
                    let curr_player = self.players.player(id);
                    let prev_muted = curr_player.map(|p| p.state.is_muted);
                    let prev_volume = curr_player.map(|p| p.state.volume);

                    if let Ok(mut handle) = self.create_video_handle_with_start(
                        id,
                        &b.path,
                        &title,
                        Some(b.position_secs),
                    ) {
                        handle.set_muted(prev_muted.unwrap_or(false));
                        if let Some(vol) = prev_volume {
                            handle.set_volume(vol);
                        }
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.push_player_nav_entry(id, b.path.clone(), Some(b.position_secs));
                        if let Some(p) = self.players.player_mut(id) {
                            p.handle = handle;
                        }
                    }
                    Some(id)
                } else {
                    let id = self.next_player_id;
                    self.next_player_id += 1;
                    let title = format::format_video_title(&b.path);
                    if let Ok(mut handle) = self.create_video_handle_with_start(
                        id,
                        &b.path,
                        &title,
                        Some(b.position_secs),
                    ) {
                        handle.set_muted(false);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.push_player_nav_entry(id, b.path.clone(), Some(b.position_secs));
                        self.players.push(AppPlayer::new(handle, b.is_shuffle));
                        self.focused_idx = 0;
                        Some(id)
                    } else {
                        None
                    }
                };

                // 5. Reconcile other players if what they are playing does not exist in the new queried list of files
                self.reconcile_players_with_available_videos(focused_id);
                self.save_session_state();

                let mode_str = if b.is_shuffle {
                    self.t("bookmarks.shuffle")
                } else {
                    self.t("bookmarks.linear")
                };
                let time_str = format::format_time_str(b.position_secs);
                self.overlay.toast_message = Some(self.t_with(
                    "bookmarks.jumped",
                    &[("name", &b.name), ("time", &time_str), ("mode", &mode_str)],
                ));
                self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                self.modals.bookmarks = false;
                if self.drawers.show_transcript {
                    return self.load_transcript_for_focused_player();
                }
                Task::none()
            }
            Message::RandomSeekFocused => {
                self.trigger_player_overlay();
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player_mut(id) {
                        p.seek_random();
                        let pos = p.position();
                        let dur = p.duration();
                        let pos_str = format::format_time_str(pos.as_secs_f64());
                        let dur_str = format::format_time_str(dur.as_secs_f64());
                        self.overlay.toast_message = Some(self.t_with(
                            "player.random_seek",
                            &[("pos", &pos_str), ("dur", &dur_str)],
                        ));
                        self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    }
                }
                Task::none()
            }
            Message::ShowTitleOverlay => {
                if self.overlay.title_pill_ticks > 0 {
                    self.overlay.title_pill_ticks = 0;
                } else {
                    self.overlay.title_pill_ticks = 240; // ~4 seconds
                }
                Task::none()
            }
            Message::ToggleSubtitles => {
                self.subtitles_enabled = !self.subtitles_enabled;
                for p in &mut self.players {
                    p.set_subtitles_visible(self.subtitles_enabled);
                }
                let status = if self.subtitles_enabled {
                    self.t("player.subtitles_on")
                } else {
                    self.t("player.subtitles_off")
                };
                self.overlay.toast_message =
                    Some(self.t_with("player.subtitles", &[("status", &status)]));
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Restores regular (non-cube) player count to the target count without affecting active 3D cube players.
    pub(crate) fn restore_grid_players_to_target(&mut self, target: usize) {
        let target_count = target.clamp(1, 12);
        let grid_count = self.players.iter().filter(|p| !p.is_cube).count();
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
                if self.add_player_internal().is_none() {
                    break;
                }
            }
        }
        if self.focused_idx >= self.players.len() && !self.players.is_empty() {
            self.focused_idx = self.players.len() - 1;
        }
    }
}
