/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Search Queries & Folder Filters Reducer
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use iced::Task;

impl WazooApp {
    /// Handles messages for full-text search input, tag filtering, folder selection/toggling,
    /// and video database query re-evaluation.
    pub(crate) fn update_search(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectSearchFolder(folder) => {
                if self.is_all_folder(&folder) {
                    self.reset_search_folder_selection();
                } else {
                    self.search.selected_folders = vec![folder.clone()];
                    self.search.selected_folder = folder;
                }
                Task::none()
            }
            Message::ToggleSearchFolder(folder) => {
                self.toggle_search_folder(&folder);
                Task::none()
            }
            Message::ResetSearchFolder => {
                let all_label = self.t("common.all");
                self.search.active_folders.clear();
                self.search.selected_folders.clear();
                self.search.active_folder = all_label.clone();
                self.search.selected_folder = all_label.clone();
                self.settings.last_folders.clear();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders: Vec<String> = Vec::new();
                if let Ok(results) = self.db.search_videos(&self.search.active_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.drawers.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    self.show_video_totals_notice(total, &all_label);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.drawers.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::RemoveActiveSearchFolder(folder) => {
                self.search.active_folders.retain(|f| f != &folder);
                self.search.selected_folders.retain(|f| f != &folder);
                self.settings.last_folders.retain(|f| f != &folder);

                let all_label = self.t("common.all");
                if self.search.active_folders.is_empty() {
                    self.search.active_folder = all_label.clone();
                    self.search.selected_folder = all_label.clone();
                } else if self.search.active_folders.len() == 1 {
                    self.search.active_folder = self.search.active_folders[0].clone();
                    self.search.selected_folder = self.search.active_folders[0].clone();
                } else {
                    self.search.active_folder = self.search.active_folders.join(", ");
                    self.search.selected_folder = self.search.active_folders.join(", ");
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders = self.search.active_folders.clone();
                if let Ok(results) = self.db.search_videos(&self.search.active_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.drawers.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    let notice_folder = if self.search.active_folders.is_empty() {
                        all_label
                    } else if self.search.active_folders.len() == 1 {
                        self.search.active_folders[0].clone()
                    } else {
                        let names: Vec<String> = self
                            .search
                            .active_folders
                            .iter()
                            .map(|f| format::folder_basename(f).to_string())
                            .collect();
                        names.join(", ")
                    };
                    self.show_video_totals_notice(total, &notice_folder);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.drawers.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::SearchInputChanged(val) => {
                if val.contains(',') {
                    let mut parts: Vec<&str> = val.split(',').collect();
                    let remainder = parts.pop().unwrap_or("").to_string();
                    for part in parts {
                        let tag = part.trim();
                        if !tag.is_empty()
                            && !self.search.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                        {
                            self.search.tags.push(tag.to_string());
                        }
                    }
                    self.search.input = remainder.trim_start().to_string();
                    iced::widget::operation::focus("search_input")
                } else {
                    self.search.input = val;
                    Task::none()
                }
            }
            Message::RemoveSearchTag(idx) => {
                if idx < self.search.tags.len() {
                    self.search.tags.remove(idx);
                }
                iced::widget::operation::focus("search_input")
            }
            Message::PerformSearch => {
                let pending = self.search.input.trim();
                if !pending.is_empty()
                    && !self
                        .search
                        .tags
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(pending))
                {
                    self.search.tags.push(pending.to_string());
                }
                self.search.input.clear();
                self.search.active_query = self.search.tags.join(", ");
                self.search.active_folders = self.search.selected_folders.clone();
                if self.search.active_folders.is_empty() {
                    self.search.active_folder = self.t("common.all");
                } else if self.search.active_folders.len() == 1 {
                    self.search.active_folder = self.search.active_folders[0].clone();
                } else {
                    self.search.active_folder = self.search.active_folders.join(", ");
                }
                self.settings.last_query = self.search.active_query.clone();
                self.settings.last_folders = self.search.active_folders.clone();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders = self.search.active_folders.clone();

                if let Ok(results) = self.db.search_videos(&self.search.active_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.drawers.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    let notice_folder = if self.search.active_folders.is_empty() {
                        self.t("common.all")
                    } else if self.search.active_folders.len() == 1 {
                        self.search.active_folders[0].clone()
                    } else {
                        let names: Vec<String> = self
                            .search
                            .active_folders
                            .iter()
                            .map(|f| format::folder_basename(f).to_string())
                            .collect();
                        names.join(", ")
                    };
                    self.show_video_totals_notice(total, &notice_folder);

                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.drawers.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                self.modals.search = false;
                Task::none()
            }
            Message::ClearSearch => {
                let all_label = self.t("common.all");
                self.search.input.clear();
                self.search.tags.clear();
                self.search.active_query.clear();
                self.search.active_folders.clear();
                self.search.selected_folders.clear();
                self.search.active_folder = all_label.clone();
                self.search.selected_folder = all_label.clone();
                self.settings.last_query.clear();
                self.settings.last_folders.clear();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders: Vec<String> = Vec::new();
                if let Ok(results) = self.db.search_videos("", &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.drawers.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    self.show_video_totals_notice(total, &all_label);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.drawers.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::FolderInputChanged(val) => {
                self.search.folder_input = val;
                Task::none()
            }
            Message::SetActiveQuery(query) | Message::TitleOverlayClicked(query) => {
                self.set_active_query(&query)
            }
            _ => Task::none(),
        }
    }

    pub fn set_active_query(&mut self, query: &str) -> Task<Message> {
        let clean_query = query.trim().to_string();
        self.search.input.clear();
        self.search.tags = if clean_query.is_empty() {
            Vec::new()
        } else {
            clean_query
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        self.search.active_query = clean_query.clone();
        self.settings.last_query = clean_query.clone();
        let _ = self.config_mgr.save_settings(&self.settings);

        let skip_id = if !clean_query.is_empty() {
            if let Some(pos) = self.players.iter().position(|p| {
                let (prim, _) = format::format_title_lines(&p.state.path);
                prim.eq_ignore_ascii_case(&clean_query)
            }) {
                self.focused_idx = pos;
                self.overlay.focus_border_ticks = crate::app::FOCUS_BORDER_TICKS;
                self.players.get(pos).map(|p| p.id)
            } else {
                self.focused_player_id()
            }
        } else {
            self.focused_player_id()
        };

        let folders = self.search.active_folders.clone();
        let search_res = self.db.search_videos(&self.search.active_query, &folders);
        let (results_opt, notice_folder) = match search_res {
            Ok(res) if !res.is_empty() || folders.is_empty() => {
                let notice_folder = if self.search.active_folders.is_empty() {
                    self.t("common.all")
                } else if self.search.active_folders.len() == 1 {
                    self.search.active_folders[0].clone()
                } else {
                    let names: Vec<String> = self
                        .search
                        .active_folders
                        .iter()
                        .map(|f| format::folder_basename(f).to_string())
                        .collect();
                    names.join(", ")
                };
                (Some(res), notice_folder)
            }
            _ => {
                if let Ok(all_res) = self.db.search_videos(&self.search.active_query, &[]) {
                    if !all_res.is_empty() {
                        self.search.active_folders.clear();
                        self.search.selected_folders.clear();
                        let all_label = self.t("common.all");
                        self.search.active_folder = all_label.clone();
                        self.search.selected_folder = all_label.clone();
                        self.settings.last_folders.clear();
                        let _ = self.config_mgr.save_settings(&self.settings);
                        (Some(all_res), all_label)
                    } else {
                        (search_res.ok(), self.t("common.all"))
                    }
                } else {
                    (search_res.ok(), self.t("common.all"))
                }
            }
        };

        if let Some(results) = results_opt {
            let total = results.len();
            self.available_videos = results;
            self.drawers.file_picker_entries.clear();
            self.apply_file_picker_search();
            self.show_video_totals_notice(total, &notice_folder);
            self.reconcile_players_with_available_videos(skip_id);
            self.save_session_state();
            if self.drawers.show_transcript {
                return self.load_transcript_for_focused_player();
            }
        }
        self.modals.search = false;
        Task::none()
    }
}
