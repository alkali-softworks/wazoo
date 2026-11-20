/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Volume, Mute & Audio Track Selection Handlers
 */

use crate::app::{DEFAULT_TOAST_SECS, SHORT_TOAST_SECS, WazooApp};
use crate::message::Message;
use crate::state::AppPlayer;
use iced::Task;
use wazoo_core::PlaybackMode;
use wazoo_media::PlayerId;

fn sync_mute_for_volume(player: &mut AppPlayer) {
    if player.state.volume > 0.0 {
        if player.state.is_muted {
            player.set_muted(false);
        }
    } else {
        player.set_muted(true);
    }
}

impl WazooApp {
    pub(super) fn handle_set_volume(&mut self, id: PlayerId, vol: f64) -> Task<Message> {
        self.focus_player_by_id(id);
        if let Some(p) = self.players.player_mut(id) {
            p.set_volume(vol);
            sync_mute_for_volume(p);
        }
        self.save_session_state();
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_adjust_volume_focused(&mut self, delta: f64) -> Task<Message> {
        if self.settings.playback_mode == PlaybackMode::Scroll && delta > 0.0 {
            return self.update(Message::GlobalUnmute);
        }
        if let Some(id) = self.focused_player_id() {
            let mut vol_display = None;
            if let Some(p) = self.players.player_mut(id) {
                p.adjust_volume(delta);
                sync_mute_for_volume(p);
                vol_display = Some(p.state.volume);
            }
            if let Some(v) = vol_display {
                let pct_str = format!("{:.0}", v * 100.0);
                let msg = self.t_with("player.volume", &[("percent", &pct_str)]);
                self.show_toast_for(msg, SHORT_TOAST_SECS);
                self.save_session_state();
            }
        }
        Task::none()
    }

    pub(super) fn handle_select_audio_track(
        &mut self,
        id: PlayerId,
        track_id: i64,
    ) -> Task<Message> {
        self.open_audio_menu_id = None;
        self.focus_player_by_id(id);
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
            let msg = self.t_with("player.audio_track", &[("label", &label)]);
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
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

    pub(super) fn handle_toggle_audio_menu(&mut self, id: PlayerId) -> Task<Message> {
        if self.open_audio_menu_id == Some(id) {
            self.open_audio_menu_id = None;
        } else {
            self.open_audio_menu_id = Some(id);
            self.hovered_player_id = Some(id);
            self.trigger_player_overlay();
        }
        Task::none()
    }

    pub(super) fn handle_toggle_player_mute(&mut self, id: PlayerId) -> Task<Message> {
        self.focus_player_by_id(id);
        if let Some(p) = self.players.player_mut(id) {
            let muted = !p.state.is_muted;
            p.set_muted(muted);
            let msg = if muted {
                self.t("player.muted")
            } else {
                self.t("player.unmuted")
            };
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        }
        self.save_session_state();
        self.trigger_player_overlay();
        Task::none()
    }

    pub(super) fn handle_toggle_mute_focused(&mut self) -> Task<Message> {
        if self.settings.playback_mode == PlaybackMode::Scroll {
            return self.update(Message::ToggleGlobalMute);
        }
        if let Some(id) = self.focused_player_id() {
            return self.update(Message::TogglePlayerMute(id));
        }
        Task::none()
    }

    pub(super) fn apply_global_mute(&mut self, muted: bool) -> Task<Message> {
        self.settings.scroll_mode_muted = muted;
        self.scroll_engine.scroll_mode_muted = muted;
        for p in &mut self.players {
            p.set_muted(muted);
            if self.settings.playback_mode == PlaybackMode::Scroll {
                let vol = self.scroll_engine.calculate_player_volume(p.id);
                p.set_crossfade_volume(vol);
            }
        }
        if let Some(ref mut preloaded) = self.preloaded_player {
            preloaded.set_muted(muted);
        }
        let msg = if muted {
            self.t("wazoo.global_mode_muted")
        } else {
            self.t("wazoo.global_mode_unmuted")
        };
        self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        let _ = self.config_mgr.save_settings(&self.settings);
        Task::none()
    }
}
