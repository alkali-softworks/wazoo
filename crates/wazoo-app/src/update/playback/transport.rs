/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Playback Transport, Seeking, Marks & Overlay Handlers
 */

use crate::app::{
    DEFAULT_TOAST_SECS, PLAYER_OVERLAY_FADE_TICKS, SEEK_DEBOUNCE_TICKS, WazooApp,
};
use crate::format;
use crate::message::Message;
use crate::state::SeekDebounce;
use iced::Task;
use std::time::Duration;
use wazoo_media::PlayerId;

impl WazooApp {
    pub(super) fn handle_player_clicked(&mut self, id: PlayerId) -> Task<Message> {
        if self.open_audio_menu_id.is_some() {
            self.open_audio_menu_id = None;
        }
        if self.is_modal_or_menu_open() {
            return Task::none();
        }
        if self.players.player_index(id).is_some() {
            let changed = self.focus_player_by_id(id);
            if changed && self.drawers.show_transcript {
                return self.load_transcript_for_focused_player();
            }
            self.trigger_player_overlay();
        }
        Task::none()
    }

    pub(super) fn handle_toggle_play(&mut self, id: PlayerId) -> Task<Message> {
        self.focus_player_by_id(id);
        if let Some(p) = self.players.player_mut(id) {
            p.flush_seek_debounce();
            p.toggle_play();
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn set_mark_focused(&mut self, is_in: bool) -> Task<Message> {
        if let Some(p) = self.focused_player_mut() {
            let pos = p.position();
            let id = p.id;
            if is_in {
                p.set_mark_in(Some(pos));
            } else {
                p.set_mark_out(Some(pos));
            }
            let time_str = format::format_time_str(pos.as_secs_f64());
            let id_str = id.to_string();
            let msg = if let (Some(i), Some(o)) = (p.mark_in(), p.mark_out()) {
                let start_str = format::format_time_str(i.as_secs_f64());
                let end_str = format::format_time_str(o.as_secs_f64());
                self.t_with(
                    "player.loop_range",
                    &[
                        ("player", &id_str),
                        ("start", &start_str),
                        ("end", &end_str),
                    ],
                )
            } else {
                let key = if is_in {
                    "player.mark_in_set"
                } else {
                    "player.mark_out_set"
                };
                self.t_with(key, &[("player", &id_str), ("time", &time_str)])
            };
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn clear_mark_focused(&mut self, is_in: bool) -> Task<Message> {
        if let Some(p) = self.focused_player_mut() {
            let id_str = p.id.to_string();
            let remaining = if is_in {
                p.clear_mark_in();
                p.mark_out().map(|pos| ("player.mark_out_set", pos))
            } else {
                p.clear_mark_out();
                p.mark_in().map(|pos| ("player.mark_in_set", pos))
            };
            let msg = if let Some((key, pos)) = remaining {
                let time_str = format::format_time_str(pos.as_secs_f64());
                self.t_with(key, &[("player", &id_str), ("time", &time_str)])
            } else {
                self.t_with("player.loop_cleared", &[("player", &id_str)])
            };
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_clear_loop(&mut self, id: PlayerId) -> Task<Message> {
        if let Some(p) = self.players.player_mut(id) {
            p.clear_marks();
            let id_str = id.to_string();
            let msg = self.t_with("player.loop_cleared", &[("player", &id_str)]);
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_seek(&mut self, id: PlayerId, pos: Duration) -> Task<Message> {
        self.focus_player_by_id(id);
        if let Some(p) = self.players.player_mut(id) {
            p.cancel_seek_debounce();
            p.seek(pos);
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_seek_ratio(&mut self, id: PlayerId, ratio: f32) -> Task<Message> {
        self.focus_player_by_id(id);
        if let Some(p) = self.players.player_mut(id) {
            p.cancel_seek_debounce();
            let dur = p.duration();
            if dur > Duration::ZERO {
                let target_secs = dur.as_secs_f64() * (ratio.clamp(0.0, 1.0) as f64);
                let target = Duration::from_secs_f64(target_secs);
                p.seek(target);

                let pos_str = format::format_time_str(target.as_secs_f64());
                let dur_str = format::format_time_str(dur.as_secs_f64());
                let msg = self.t_with(
                    "player.seek_position",
                    &[("pos", &pos_str), ("dur", &dur_str)],
                );
                self.show_toast_for(msg, DEFAULT_TOAST_SECS);
            }
        }
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_player_hovered(&mut self, id: PlayerId) -> Task<Message> {
        if self.window.is_resizing()
            || self.is_modal_or_menu_open()
            || (self.titlebar.show && self.is_point_in_titlebar(self.window.cursor_position))
        {
            if self.hovered_player_id == Some(id) {
                let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                self.overlay.ticks = self.overlay.ticks.min(current_fade);
            }
            return Task::none();
        }
        self.hovered_player_id = Some(id);
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_player_unhovered(&mut self, id: PlayerId) -> Task<Message> {
        if self.window.is_resizing() || self.open_audio_menu_id == Some(id) {
            return Task::none();
        }
        if self.hovered_player_id == Some(id) {
            let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
            self.overlay.ticks = self.overlay.ticks.min(current_fade);
        }
        Task::none()
    }

    pub(super) fn handle_seek_relative_focused(&mut self, secs: f64) -> Task<Message> {
        self.trigger_player_overlay();
        if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player_mut(id) {
                let dur_secs = p.duration().as_secs_f64();
                let max_target = if dur_secs > 0.5 {
                    dur_secs - 0.1
                } else {
                    f64::MAX
                };

                let (accumulated, target) = if let Some(ref mut deb) = p.seek_debounce {
                    deb.target_secs = (deb.target_secs + secs).clamp(0.0, max_target);
                    deb.ticks_remaining = SEEK_DEBOUNCE_TICKS;
                    (deb.accumulated_secs(), deb.target_secs)
                } else {
                    let current = p.handle.position().as_secs_f64();
                    let target = (current + secs).clamp(0.0, max_target);
                    let deb = SeekDebounce {
                        start_secs: current,
                        target_secs: target,
                        ticks_remaining: SEEK_DEBOUNCE_TICKS,
                    };
                    let accumulated = deb.accumulated_secs();
                    p.seek_debounce = Some(deb);
                    (accumulated, target)
                };

                let sign = if accumulated > 0.0 { "+" } else { "" };
                let pos_str = format::format_time_str(target);
                let dur_str = format::format_time_str(dur_secs);
                let secs_str = format!("{accumulated:.0}");
                let msg = self.t_with(
                    "player.seek_relative",
                    &[
                        ("sign", sign),
                        ("secs", &secs_str),
                        ("pos", &pos_str),
                        ("dur", &dur_str),
                    ],
                );
                self.show_toast_for(msg, DEFAULT_TOAST_SECS);
            }
        }
        Task::none()
    }

    pub(super) fn step_frame_focused(&mut self, forward: bool) -> Task<Message> {
        self.trigger_player_overlay();
        if let Some(p) = self.focused_player_mut() {
            p.flush_seek_debounce();
            if forward {
                p.step_frame_forward();
            } else {
                p.step_frame_backward();
            }
        }
        Task::none()
    }

    pub(super) fn handle_random_seek_focused(&mut self) -> Task<Message> {
        self.trigger_player_overlay();
        if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player_mut(id) {
                p.cancel_seek_debounce();
                p.seek_random();
                let pos_str = format::format_time_str(p.position().as_secs_f64());
                let dur_str = format::format_time_str(p.duration().as_secs_f64());
                let msg = self.t_with(
                    "player.random_seek",
                    &[("pos", &pos_str), ("dur", &dur_str)],
                );
                self.show_toast_for(msg, DEFAULT_TOAST_SECS);
            }
        }
        Task::none()
    }

    pub(super) fn handle_show_title_overlay(&mut self) -> Task<Message> {
        if self.overlay.title_pill_ticks > 0 {
            self.overlay.title_pill_ticks = 0;
        } else {
            self.overlay.title_pill_ticks = 240; // ~4 seconds
        }
        Task::none()
    }

    pub(super) fn handle_toggle_subtitles(&mut self) -> Task<Message> {
        self.subtitles_enabled = !self.subtitles_enabled;
        for p in &mut self.players {
            p.set_subtitles_visible(self.subtitles_enabled);
        }
        let msg = if self.subtitles_enabled {
            self.t("player.subtitles_enabled")
        } else {
            self.t("player.subtitles_disabled")
        };
        self.overlay.show_toast(msg);
        Task::none()
    }

    pub(super) fn handle_copy_player_info(&mut self, id: PlayerId) -> Task<Message> {
        if let Some(info) = self.format_player_copy_info(id) {
            self.overlay.show_toast(self.t("player.info_copied"));
            iced::clipboard::write(info)
        } else {
            Task::none()
        }
    }
}
