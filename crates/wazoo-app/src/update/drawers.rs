/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Side Drawers Reducer
 *
 * Implements message handlers for File Picker, History Drawer, and Transcript Subtitles Drawer.
 */

use crate::app::{LONG_TOAST_SECS, WazooApp};
use crate::format;
use crate::message::Message;
use iced::Task;
use std::time::Duration;

impl WazooApp {
    pub(crate) fn update_drawers(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleFilePicker => {
                if self.drawers.show_file_picker {
                    self.close_file_picker();
                } else {
                    self.drawers.show_file_picker = true;
                    self.drawers.show_transcript = false;
                    self.drawers.show_history_drawer = false;
                    if self.drawers.file_picker_groups.is_empty()
                        && !self.available_videos.is_empty()
                    {
                        self.apply_file_picker_search();
                    }
                }
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                Task::none()
            }
            Message::ToggleTranscript => {
                self.drawers.show_transcript = !self.drawers.show_transcript;
                if !self.drawers.show_transcript {
                    self.drawers.show_transcript_menu = false;
                }
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                if self.drawers.show_transcript {
                    self.close_file_picker();
                    self.drawers.show_history_drawer = false;
                    return self.load_transcript_for_focused_player();
                }
                Task::none()
            }
            Message::ToggleTranscriptForPlayer(id) => {
                let is_same_focused = self.focused_player_id() == Some(id);
                if is_same_focused && self.drawers.show_transcript {
                    self.drawers.show_transcript = false;
                    self.drawers.show_transcript_menu = false;
                    return Task::none();
                }
                if let Some(idx) = self.players.player_index(id) {
                    self.focused_idx = idx;
                }
                self.drawers.show_transcript = true;
                self.drawers.show_transcript_menu = false;
                self.close_file_picker();
                self.drawers.show_history_drawer = false;
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                self.load_transcript_for_focused_player()
            }
            Message::ToggleHistoryDrawer => {
                self.drawers.show_history_drawer = !self.drawers.show_history_drawer;
                if self.drawers.show_history_drawer {
                    self.close_file_picker();
                    self.drawers.show_transcript = false;
                    self.drawers.show_transcript_menu = false;
                }
                self.titlebar.show_dropdown_menu = false;
                self.modals.menu = false;
                Task::none()
            }
            Message::CloseHistoryDrawer => {
                self.drawers.show_history_drawer = false;
                Task::none()
            }
            Message::HistorySearchChanged(s) => {
                self.drawers.history_search = s;
                Task::none()
            }
            Message::ClearPlayHistory => {
                self.drawers.play_history.clear();
                Task::none()
            }
            Message::CloseTranscript => {
                self.drawers.show_transcript = false;
                self.drawers.show_transcript_menu = false;
                Task::none()
            }
            Message::TranscriptSearchChanged(s) => {
                self.drawers.transcript_search = s;
                Task::none()
            }
            Message::TranscriptLoaded(path, track_idx, cues) => {
                if self.drawers.transcript_video_path.as_deref() == Some(&path)
                    && self.drawers.transcript_track_index == track_idx
                {
                    self.drawers.transcript_loading = false;
                    self.drawers.transcript_cues = cues;
                }
                Task::none()
            }
            Message::SeekToSubtitle(secs) => {
                if let Some(id) = self.focused_player_id() {
                    // Offset by +10ms so playback starts cleanly inside the target cue,
                    // avoiding boundary collision with the preceding cue.
                    return self.update(Message::Seek(
                        id,
                        Duration::from_secs_f64((secs + 0.01).max(0.0)),
                    ));
                }
                Task::none()
            }
            Message::ToggleTranscriptSubtitleMenu => {
                self.drawers.show_transcript_menu = !self.drawers.show_transcript_menu;
                Task::none()
            }
            Message::CloseTranscriptSubtitleMenu => {
                self.drawers.show_transcript_menu = false;
                Task::none()
            }
            Message::SelectTranscriptSubtitleTrack(track_idx, track_id) => {
                self.drawers.show_transcript_menu = false;
                self.drawers.transcript_track_index = track_idx;
                self.subtitles_enabled = true;
                if let Some(player) = self.focused_player_mut() {
                    player.set_subtitles_visible(true);
                    player.set_subtitle_track(track_id);
                    let sub_track = player.subtitle_tracks().get(track_idx).cloned();
                    let path = player.state.path.clone();
                    if !path.is_empty() {
                        self.drawers.transcript_video_path = Some(path.clone());
                        self.drawers.transcript_loading = true;
                        self.drawers.transcript_cues.clear();
                        let path_clone = path.clone();
                        let ext_file = sub_track.as_ref().and_then(|t| t.external_filename.clone());
                        let ff_index = sub_track.as_ref().and_then(|t| t.ff_index);

                        return Task::perform(
                            async move {
                                wazoo_media::load_subtitles_for_track_details(
                                    path, ext_file, ff_index, track_idx,
                                )
                                .await
                            },
                            move |cues| Message::TranscriptLoaded(path_clone, track_idx, cues),
                        );
                    }
                }
                Task::none()
            }
            Message::ToggleFolderCollapse(folder) => {
                if self.drawers.expanded_folders.contains(&folder) {
                    self.drawers.expanded_folders.remove(&folder);
                } else {
                    self.drawers.expanded_folders.insert(folder);
                }
                Task::none()
            }
            Message::FilePickerSearchChanged(s) => {
                self.drawers.file_picker_search = s;
                let trimmed = self.drawers.file_picker_search.trim();
                if trimmed.is_empty() {
                    self.drawers.file_picker_debounce_ticks = 0;
                    self.apply_file_picker_search();
                } else {
                    self.drawers.file_picker_debounce_ticks =
                        crate::app::FILE_PICKER_DEBOUNCE_TICKS;
                }
                Task::none()
            }
            Message::ApplyFilePickerSearch => {
                self.drawers.file_picker_debounce_ticks = 0;
                self.apply_file_picker_search();
                Task::none()
            }
            Message::PlayFileInFocused(path) => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.player_mut(id) {
                        p.flip.reset(self.settings.flip_interval_secs);
                        p.start_loading();
                        p.nav_history.forward_stack.clear();
                    }
                    self.record_current_player_nav_position(id);
                    let title = format::format_video_title(&path);
                    let curr_player = self.players.player(id);
                    let prev_muted = curr_player.map(|p| p.state.is_muted);
                    let prev_volume = curr_player.map(|p| p.state.volume);

                    if let Ok(mut handle) = self.create_video_handle(id, &path, &title) {
                        handle.set_muted(prev_muted.unwrap_or(true));
                        if let Some(vol) = prev_volume {
                            handle.set_volume(vol);
                        }
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.push_player_nav_entry(id, path.clone(), None);
                        if let Some(p) = self.players.player_mut(id) {
                            p.handle = handle;
                        }
                        self.overlay.toast_message =
                            Some(self.t_with("player.playing", &[("title", &title)]));
                        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        if self.drawers.show_transcript {
                            return self.load_transcript_for_focused_player();
                        }
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
