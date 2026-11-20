/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Playback, Layout & Multi-Player Reducer
 *
 * Routes playback, audio, layout/mode, and bookmark messages to focused
 * domain submodules and provides shared focus and toast helpers for `WazooApp`.
 */

mod audio;
mod bookmarks;
mod layout;
mod transport;

use crate::app::{FOCUS_BORDER_TICKS, WazooApp};
use crate::message::Message;
use iced::Task;
use wazoo_media::PlayerId;

impl WazooApp {
    /// Focuses the player matching `id` if present, updating the focus-border flash
    /// when focus changed. Returns `true` if the active focused player changed.
    pub(super) fn focus_player_by_id(&mut self, id: PlayerId) -> bool {
        if let Some(pos) = self.players.player_index(id) {
            let changed = self.focused_idx != pos;
            self.focused_idx = pos;
            if changed {
                self.overlay.focus_border_ticks = FOCUS_BORDER_TICKS;
            }
            changed
        } else {
            false
        }
    }

    pub(super) fn show_toast_for(&mut self, msg: String, duration_secs: usize) {
        self.overlay.toast_message = Some(msg);
        self.overlay.toast_time_remaining = duration_secs;
    }

    pub(crate) fn update_playback(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PlayerClicked(id) => self.handle_player_clicked(id),
            Message::TogglePlay(id) => self.handle_toggle_play(id),
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
            Message::MarkInFocused => self.set_mark_focused(true),
            Message::MarkOutFocused => self.set_mark_focused(false),
            Message::ClearMarkInFocused => self.clear_mark_focused(true),
            Message::ClearMarkOutFocused => self.clear_mark_focused(false),
            Message::ClearLoop(id) => self.handle_clear_loop(id),
            Message::ClearLoopFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::ClearLoop(id));
                }
                Task::none()
            }
            Message::Seek(id, pos) => self.handle_seek(id, pos),
            Message::SeekRatio(id, ratio) => self.handle_seek_ratio(id, ratio),
            Message::PlayerHovered(id) => self.handle_player_hovered(id),
            Message::PlayerUnhovered(id) => self.handle_player_unhovered(id),
            Message::SeekRelativeFocused(secs) => self.handle_seek_relative_focused(secs),
            Message::StepFrameForwardFocused => self.step_frame_focused(true),
            Message::StepFrameBackwardFocused => self.step_frame_focused(false),
            Message::SetVolume(id, vol) => self.handle_set_volume(id, vol),
            Message::AdjustVolumeFocused(delta) => self.handle_adjust_volume_focused(delta),
            Message::SelectAudioTrack(id, track_id) => {
                self.handle_select_audio_track(id, track_id)
            }
            Message::ToggleAudioMenu(id) => self.handle_toggle_audio_menu(id),
            Message::CloseAudioMenu => {
                self.open_audio_menu_id = None;
                Task::none()
            }
            Message::TogglePlayerMute(id) => self.handle_toggle_player_mute(id),
            Message::ToggleMuteFocused => self.handle_toggle_mute_focused(),
            Message::ToggleGlobalMute => self.apply_global_mute(!self.settings.scroll_mode_muted),
            Message::GlobalUnmute => self.apply_global_mute(false),
            Message::TogglePlayerShuffle(id) => self.handle_toggle_player_shuffle(id),
            Message::ToggleShuffleMode => self.handle_toggle_shuffle_mode(),
            Message::CycleLayout => self.handle_cycle_layout(),
            Message::SetPlayerCount(count) => self.handle_set_player_count(count),
            Message::ToggleScrollMode => self.handle_toggle_scroll_mode(),
            Message::ToggleFlipMode => self.handle_toggle_flip_mode(),
            Message::SetScrollSpeed(speed) => self.handle_set_scroll_speed(speed),
            Message::AdjustScrollSpeed(delta) => {
                let new_speed = (self.settings.scroll_speed + delta).clamp(0.1, 10.0);
                self.update(Message::SetScrollSpeed(new_speed))
            }
            Message::PreloadedPlayerReady(holder) => self.handle_preloaded_player_ready(holder),
            Message::AddNewPlayer => self.handle_add_new_player(),
            Message::RemoveFocusedPlayer => self.handle_remove_focused_player(),
            Message::CycleFocusedPlayer => self.handle_cycle_focused_player(),
            Message::SetFocusedPlayer(idx) => self.handle_set_focused_player(idx),
            Message::AddBookmarkFocused => self.handle_add_bookmark_focused(),
            Message::RemoveBookmarkFocused => self.handle_remove_bookmark_focused(),
            Message::RemoveBookmark(idx) => self.handle_remove_bookmark(idx),
            Message::JumpToBookmark(b) => self.handle_jump_to_bookmark(b),
            Message::RandomSeekFocused => self.handle_random_seek_focused(),
            Message::ShowTitleOverlay => self.handle_show_title_overlay(),
            Message::ToggleSubtitles => self.handle_toggle_subtitles(),
            Message::CopyPlayerInfo(id) => self.handle_copy_player_info(id),
            _ => Task::none(),
        }
    }
}
