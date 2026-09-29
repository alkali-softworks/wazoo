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
    pub(crate) fn update_search(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectSearchFolder(folder) => {
                if self.is_all_folder(&folder) {
                    self.reset_search_folder_selection();
                } else {
                    self.selected_search_folders = vec![folder.clone()];
                    self.selected_search_folder = folder;
                }
                Task::none()
            }
            Message::ToggleSearchFolder(folder) => {
                self.toggle_search_folder(&folder);
                Task::none()
            }
            Message::ResetSearchFolder => {
                let all_label = self.t("common.all");
                self.active_search_folders.clear();
                self.selected_search_folders.clear();
                self.active_search_folder = all_label.clone();
                self.selected_search_folder = all_label.clone();
                self.settings.last_folders.clear();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders: Vec<String> = Vec::new();
                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    self.show_video_totals_notice(total, &all_label);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::RemoveActiveSearchFolder(folder) => {
                self.active_search_folders.retain(|f| f != &folder);
                self.selected_search_folders.retain(|f| f != &folder);
                self.settings.last_folders.retain(|f| f != &folder);

                let all_label = self.t("common.all");
                if self.active_search_folders.is_empty() {
                    self.active_search_folder = all_label.clone();
                    self.selected_search_folder = all_label.clone();
                } else if self.active_search_folders.len() == 1 {
                    self.active_search_folder = self.active_search_folders[0].clone();
                    self.selected_search_folder = self.active_search_folders[0].clone();
                } else {
                    self.active_search_folder = self.active_search_folders.join(", ");
                    self.selected_search_folder = self.active_search_folders.join(", ");
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders = self.active_search_folders.clone();
                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    let notice_folder = if self.active_search_folders.is_empty() {
                        all_label
                    } else if self.active_search_folders.len() == 1 {
                        self.active_search_folders[0].clone()
                    } else {
                        let names: Vec<String> = self
                            .active_search_folders
                            .iter()
                            .map(|f| format::folder_basename(f).to_string())
                            .collect();
                        names.join(", ")
                    };
                    self.show_video_totals_notice(total, &notice_folder);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.show_transcript {
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
                            && !self.search_tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                        {
                            self.search_tags.push(tag.to_string());
                        }
                    }
                    self.search_input = remainder.trim_start().to_string();
                    iced::widget::operation::focus("search_input")
                } else {
                    self.search_input = val;
                    Task::none()
                }
            }
            Message::RemoveSearchTag(idx) => {
                if idx < self.search_tags.len() {
                    self.search_tags.remove(idx);
                }
                iced::widget::operation::focus("search_input")
            }
            Message::PerformSearch => {
                let pending = self.search_input.trim();
                if !pending.is_empty()
                    && !self
                        .search_tags
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(pending))
                {
                    self.search_tags.push(pending.to_string());
                }
                self.search_input.clear();
                self.active_search_query = self.search_tags.join(", ");
                self.active_search_folders = self.selected_search_folders.clone();
                if self.active_search_folders.is_empty() {
                    self.active_search_folder = self.t("common.all");
                } else if self.active_search_folders.len() == 1 {
                    self.active_search_folder = self.active_search_folders[0].clone();
                } else {
                    self.active_search_folder = self.active_search_folders.join(", ");
                }
                self.settings.last_query = self.active_search_query.clone();
                self.settings.last_folders = self.active_search_folders.clone();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders = self.active_search_folders.clone();

                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    let notice_folder = if self.active_search_folders.is_empty() {
                        self.t("common.all")
                    } else if self.active_search_folders.len() == 1 {
                        self.active_search_folders[0].clone()
                    } else {
                        let names: Vec<String> = self
                            .active_search_folders
                            .iter()
                            .map(|f| format::folder_basename(f).to_string())
                            .collect();
                        names.join(", ")
                    };
                    self.show_video_totals_notice(total, &notice_folder);

                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                self.show_search_modal = false;
                Task::none()
            }
            Message::ClearSearch => {
                let all_label = self.t("common.all");
                self.search_input.clear();
                self.search_tags.clear();
                self.active_search_query.clear();
                self.active_search_folders.clear();
                self.selected_search_folders.clear();
                self.active_search_folder = all_label.clone();
                self.selected_search_folder = all_label.clone();
                self.settings.last_query.clear();
                self.settings.last_folders.clear();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folders: Vec<String> = Vec::new();
                if let Ok(results) = self.db.search_videos("", &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.file_picker_entries.clear();
                    self.apply_file_picker_search();
                    self.show_video_totals_notice(total, &all_label);
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                Task::none()
            }
            Message::FolderInputChanged(val) => {
                self.folder_input = val;
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
