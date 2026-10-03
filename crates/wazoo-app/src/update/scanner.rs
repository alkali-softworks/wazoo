/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Scanner & Library Folder Management Reducer
 */

use crate::app::{DEFAULT_TOAST_SECS, LONG_TOAST_SECS, WazooApp};
use crate::format;
use crate::message::Message;
use iced::Task;
use iced::futures::SinkExt;
use wazoo_scanner::Scanner;

impl WazooApp {
    /// Handles messages for folder picker dialogs, library indexing progress,
    /// scan completion reconciliation, and media directory configuration.
    pub(crate) fn update_scanner(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PickFolders => {
                let starting_dir = self.settings.media_folders.first().cloned();
                Task::perform(
                    async move {
                        let mut dialog =
                            rfd::AsyncFileDialog::new().set_title("Select Media Folder(s)");
                        if let Some(ref dir) = starting_dir {
                            dialog = dialog.set_directory(dir);
                        }
                        if let Some(handles) = dialog.pick_folders().await {
                            handles
                                .into_iter()
                                .map(|h| h.path().to_string_lossy().to_string())
                                .collect()
                        } else {
                            Vec::new()
                        }
                    },
                    Message::FoldersSelected,
                )
            }
            Message::FoldersSelected(folders) => {
                if folders.is_empty() {
                    return Task::none();
                }
                let mut added_any = false;
                for folder in folders {
                    let trimmed = folder.trim().to_string();
                    if !trimmed.is_empty() && !self.settings.media_folders.contains(&trimmed) {
                        self.settings.media_folders.push(trimmed);
                        added_any = true;
                    }
                }
                if added_any {
                    let _ = self.config_mgr.save_settings(&self.settings);
                    self.overlay.toast_message = Some(self.t("settings.folders_added_scanning"));
                    self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                    return self.update(Message::StartScan);
                }
                Task::none()
            }
            Message::AddMediaFolder => {
                let trimmed = self.search.folder_input.trim().to_string();
                if !trimmed.is_empty() {
                    if !self.settings.media_folders.contains(&trimmed) {
                        self.settings.media_folders.push(trimmed.clone());
                        self.search.folder_input.clear();
                        let _ = self.config_mgr.save_settings(&self.settings);
                        self.overlay.toast_message =
                            Some(self.t_with("settings.folder_added", &[("folder", &trimmed)]));
                        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        return self.update(Message::StartScan);
                    } else {
                        self.search.folder_input.clear();
                    }
                    Task::none()
                } else {
                    Task::done(Message::PickFolders)
                }
            }
            Message::RemoveMediaFolder(folder) => {
                self.settings.media_folders.retain(|f| f != &folder);
                let _ = self.config_mgr.save_settings(&self.settings);

                let was_scanning = self.scanner.is_scanning;

                // 1. Immediately delete all files belonging to this folder from the SQLite database
                let removed_count = self.db.remove_videos_in_folder(&folder).unwrap_or(0);
                log::info!(
                    "Removed media folder '{}' ({} videos deleted from database)",
                    folder,
                    removed_count
                );

                // 2. Immediately refresh in-memory available_videos from DB
                if !self.search.active_query.is_empty() {
                    let folders = if self.search.active_folders.is_empty() {
                        self.settings.media_folders.clone()
                    } else {
                        self.search.active_folders.clone()
                    };
                    self.available_videos = self
                        .db
                        .search_videos(&self.search.active_query, &folders)
                        .unwrap_or_default();
                } else {
                    self.available_videos = self.db.get_all_videos().unwrap_or_default();
                }
                self.drawers.file_picker_entries.clear();
                self.apply_file_picker_search();

                // 3. Immediately update active players
                if self.available_videos.is_empty() {
                    self.players.clear();
                    self.focused_idx = 0;
                } else {
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                }

                let all_label = self.t("common.all");
                self.search.active_folders.retain(|f| f != &folder);
                self.search.selected_folders.retain(|f| f != &folder);
                self.settings.last_folders.retain(|f| f != &folder);

                if self.search.active_folders.is_empty() {
                    self.search.active_folder = all_label.clone();
                } else if self.search.active_folders.len() == 1 {
                    self.search.active_folder = self.search.active_folders[0].clone();
                } else {
                    self.search.active_folder = self.search.active_folders.join(", ");
                }

                if self.search.selected_folders.is_empty() {
                    self.search.selected_folder = all_label.clone();
                } else if self.search.selected_folders.len() == 1 {
                    self.search.selected_folder = self.search.selected_folders[0].clone();
                } else {
                    self.search.selected_folder = self.search.selected_folders.join(", ");
                }

                let _ = self.config_mgr.save_settings(&self.settings);

                let folder_name = format::format_video_folder(&folder);
                let display_name = if folder_name.is_empty() {
                    folder
                } else {
                    folder_name
                };
                let count_str = format::format_number(removed_count);
                self.overlay.toast_message = Some(self.t_with(
                    "settings.folder_removed",
                    &[("folder", &display_name), ("count", &count_str)],
                ));
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                self.last_total_videos = self.available_videos.len();

                if was_scanning {
                    return self.update(Message::StartScan);
                }
                Task::none()
            }
            Message::ClearMiscVideos => {
                let removed_count = self.db.clear_misc_videos().unwrap_or(0);
                log::info!("Cleared {} Misc videos from database", removed_count);

                // Refresh available_videos
                if !self.search.active_query.is_empty() || !self.search.active_folders.is_empty() {
                    let folders = self.search.active_folders.clone();
                    self.available_videos = self
                        .db
                        .search_videos(&self.search.active_query, &folders)
                        .unwrap_or_default();
                } else {
                    self.available_videos = self.db.get_all_videos().unwrap_or_default();
                }

                self.drawers.file_picker_entries.clear();
                self.apply_file_picker_search();

                // Reconcile players if any were playing a Misc video
                if self.available_videos.is_empty() {
                    self.players.clear();
                    self.focused_idx = 0;
                } else {
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                }

                self.search.active_folders.retain(|f| f != "Misc");
                self.search.selected_folders.retain(|f| f != "Misc");
                self.settings.last_folders.retain(|f| f != "Misc");
                let _ = self.config_mgr.save_settings(&self.settings);

                let count_str = format::format_number(removed_count);
                self.overlay.toast_message =
                    Some(self.t_with("settings.misc_cleared", &[("count", &count_str)]));
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                self.last_total_videos = self.available_videos.len();
                Task::none()
            }
            Message::StartScan => {
                // Cancel any currently running scan task
                if let Some(cancel) = self.scanner.scan_cancel.take() {
                    cancel.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                self.scanner.current_scan_id += 1;

                if !self.settings.media_folders.is_empty() {
                    let scan_id = self.scanner.current_scan_id;
                    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    self.scanner.scan_cancel = Some(cancel.clone());

                    self.scanner.is_scanning = true;
                    self.scanner.scan_progress = None;
                    let folders = self.settings.media_folders.clone();
                    let db_path = self.config_mgr.database_path();

                    Task::stream(iced::stream::channel(
                        100,
                        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

                            let cancel_inner = cancel.clone();
                            let scan_handle = tokio::spawn(async move {
                                let scanner = Scanner::default();
                                scanner
                                    .scan_and_index_with_cancel(
                                        &folders,
                                        db_path,
                                        Some(tx),
                                        Some(cancel_inner),
                                    )
                                    .await
                            });

                            while let Some(progress) = rx.recv().await {
                                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                    break;
                                }
                                let _ = output
                                    .send(Message::ScanProgressUpdate(scan_id, progress))
                                    .await;
                            }

                            let res = match scan_handle.await {
                                Ok(inner_res) => inner_res,
                                Err(join_err) => Err(join_err.to_string()),
                            };

                            if !cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                let _ = output.send(Message::ScanFinished(scan_id, res)).await;
                            }
                        },
                    ))
                } else {
                    self.scanner.is_scanning = false;
                    self.scanner.scan_progress = None;
                    Task::none()
                }
            }
            Message::ScanProgressUpdate(scan_id, progress) => {
                if scan_id != self.scanner.current_scan_id {
                    return Task::none();
                }
                self.scanner.scan_progress = Some(progress);
                Task::none()
            }
            Message::ScanFinished(scan_id, res) => {
                if scan_id != self.scanner.current_scan_id {
                    return Task::none();
                }
                self.scanner.is_scanning = false;
                self.scanner.scan_progress = None;
                self.scanner.scan_cancel = None;
                match res {
                    Ok(_count) => {
                        let folders = self.search.active_folders.clone();
                        if let Ok(videos) =
                            self.db.search_videos(&self.search.active_query, &folders)
                        {
                            let total = videos.len();
                            self.available_videos = videos;
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
                            if self.players.is_empty() && !self.available_videos.is_empty() {
                                let count = self.settings.player_count.clamp(1, 12);
                                for _ in 0..count {
                                    self.add_player_internal();
                                }
                            }
                            self.reconcile_players_with_available_videos(None);
                            self.save_session_state();
                        }
                    }
                    Err(err) => {
                        if err != "Scan cancelled" {
                            self.overlay.toast_message =
                                Some(self.t_with("settings.scan_error", &[("error", &err)]));
                            self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                        }
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
