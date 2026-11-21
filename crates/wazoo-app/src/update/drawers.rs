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
    /// Handles messages for side drawer visibility, file picker searching/filtering,
    /// subtitle track selection, transcript cues seeking, and play history navigation.
    pub(crate) fn update_drawers(&mut self, message: Message) -> Task<Message> {
        let exit_task = if matches!(
            message,
            Message::ToggleFilePicker
                | Message::ToggleTranscript
                | Message::ToggleTranscriptForPlayer(_)
                | Message::ToggleHistoryDrawer
        ) && self.cube.desktop_overlay
        {
            self.exit_desktop_cube_overlay()
        } else {
            Task::none()
        };

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
                exit_task
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
                    return Task::batch([exit_task, self.load_transcript_for_focused_player()]);
                }
                exit_task
            }
            Message::ToggleTranscriptForPlayer(id) => {
                let is_same_focused = self.focused_player_id() == Some(id);
                if is_same_focused && self.drawers.show_transcript {
                    self.drawers.show_transcript = false;
                    self.drawers.show_transcript_menu = false;
                    return exit_task;
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
                Task::batch([exit_task, self.load_transcript_for_focused_player()])
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
                exit_task
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
                let mut selected_pref = None;
                let mut task_args = None;
                if let Some(player) = self.focused_player_mut() {
                    player.set_subtitles_visible(true);
                    player.set_subtitle_track(track_id);
                    let sub_track = player.subtitle_tracks().get(track_idx).cloned();
                    if let Some(ref track) = sub_track {
                        selected_pref = Some(wazoo_media::get_subtitle_track_preference_string(track));
                    }
                    let path = player.state.path.clone();
                    if !path.is_empty() {
                        self.drawers.transcript_video_path = Some(path.clone());
                        self.drawers.transcript_loading = true;
                        self.drawers.transcript_cues.clear();
                        let ext_file = sub_track.as_ref().and_then(|t| t.external_filename.clone());
                        let ff_index = sub_track.as_ref().and_then(|t| t.ff_index);
                        task_args = Some((path, ext_file, ff_index));
                    }
                }
                if let Some(pref) = selected_pref {
                    if !pref.starts_with("Track ") {
                        self.settings.preferred_subtitle_language = Some(pref.clone());
                        let _ = self.config_mgr.save_settings(&self.settings);
                        for p in &mut self.players {
                            p.set_preferred_subtitle_language(Some(pref.clone()));
                        }
                    }
                }
                if let Some((path, ext_file, ff_index)) = task_args {
                    let path_clone = path.clone();
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
                Task::none()
            }
            Message::PickSubtitleFile => {
                let Some(player) = self.focused_player() else {
                    return Task::none();
                };
                let video_path = player.state.path.clone();
                if video_path.is_empty() {
                    return Task::none();
                }
                let starting_dir = std::path::Path::new(&video_path)
                    .parent()
                    .filter(|p| p.is_dir())
                    .map(|p| p.to_path_buf());
                let dialog_title = self.t("transcript.browse_dialog_title");
                let filter_name = self.t("transcript.subtitle_files_filter");
                Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title(&dialog_title)
                            .add_filter(&filter_name, &["srt", "vtt", "ass", "ssa"]);
                        if let Some(ref dir) = starting_dir {
                            dialog = dialog.set_directory(dir);
                        }
                        dialog
                            .pick_file()
                            .await
                            .map(|h| h.path().to_string_lossy().to_string())
                    },
                    move |selected| Message::SubtitleFileSelected(video_path, selected),
                )
            }
            Message::SubtitleFileSelected(target_video_path, Some(sub_path)) => {
                let add_result = {
                    let Some(player) = self.focused_player_mut() else {
                        return Task::none();
                    };
                    if player.state.path != target_video_path {
                        return Task::none();
                    }
                    player.add_external_subtitle(&sub_path)
                };
                match add_result {
                    Ok(track_idx) => {
                        self.subtitles_enabled = true;
                        self.drawers.show_transcript_menu = false;
                        self.drawers.transcript_track_index = track_idx;
                        self.drawers.transcript_video_path = Some(target_video_path.clone());
                        self.drawers.transcript_loading = true;
                        self.drawers.transcript_cues.clear();
                        let path_clone = target_video_path.clone();
                        Task::perform(
                            wazoo_media::load_subtitles_from_file(sub_path),
                            move |cues| Message::TranscriptLoaded(path_clone, track_idx, cues),
                        )
                    }
                    Err(err) => {
                        log::warn!("Failed to load external subtitle: {err}");
                        self.overlay.toast_message = Some(err);
                        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        Task::none()
                    }
                }
            }
            Message::SubtitleFileSelected(_, None) => Task::none(),
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
            Message::ClearFilePickerSearch => {
                self.drawers.file_picker_search.clear();
                self.drawers.file_picker_debounce_ticks = 0;
                self.apply_file_picker_search();
                iced::widget::operation::focus("file_picker_search_input")
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
                        p.nav_history.forward_stack.clear();
                        p.load_retry_count = 0;
                    }
                    self.record_current_player_nav_position(id);
                    let title = format::format_video_title(&path);
                    let load_task = self.start_async_player_load(
                        id,
                        &path,
                        &title,
                        wazoo_media::StartTime::Beginning,
                    );
                    self.push_player_nav_entry(id, path, None);
                    self.overlay.toast_message =
                        Some(self.t_with("player.playing", &[("title", &title)]));
                    self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                    if self.drawers.show_transcript {
                        return Task::batch([load_task, self.load_transcript_for_focused_player()]);
                    }
                    return load_task;
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
