/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Quick Picker Sibling Video Discovery
 *
 * Opens the Quick Picker modal and gathers sibling videos from the same
 * library folder or parent filesystem directory as the focused player.
 */

use super::WazooApp;
use crate::format;
use crate::message::Message;
use iced::Task;
use std::collections::HashSet;

impl WazooApp {
    pub(crate) fn open_quick_picker(&mut self) -> Task<Message> {
        let (folder_opt, candidate_videos) = self.quick_picker_videos();
        if folder_opt.is_none() || candidate_videos.is_empty() {
            self.overlay.show_toast(self.t("wazoo.no_files_found"));
            return Task::none();
        }

        let exit_task = if self.cube.desktop_overlay {
            self.exit_desktop_cube_overlay()
        } else {
            Task::none()
        };
        self.modals.close_all();
        self.drawers.close_all();
        self.titlebar.show_dropdown_menu = false;
        self.titlebar.show = false;
        self.overlay.clear_toast();
        self.hovered_player_id = None;
        self.overlay.ticks = 0;
        self.overlay.fade_in_ticks = 0;
        self.modals.quick_picker = true;
        self.modals.quick_picker_search.clear();
        Task::batch([
            exit_task,
            iced::widget::operation::focus("quick_picker_search_input"),
        ])
    }

    /// Returns the folder name (or formatted title) and the list of other videos (path, title)
    /// from the folder of the video currently playing in the active/focused player.
    pub fn quick_picker_videos(&self) -> (Option<String>, Vec<(String, String)>) {
        let current_path = match self
            .focused_player()
            .map(|p| p.state.path.clone())
            .filter(|s| !s.trim().is_empty())
        {
            Some(p) => p,
            None => return (None, Vec::new()),
        };

        // Determine the folder name
        let folder_name = if let Some(rec) =
            self.available_videos.iter().find(|v| v.path == current_path)
        {
            if rec.folder.as_deref() == Some("Misc") {
                "Misc".to_string()
            } else {
                resolve_folder_key(&rec.path)
            }
        } else {
            resolve_folder_key(&current_path)
        };

        let mut candidate_videos: Vec<(String, String)> = Vec::new();
        let mut seen_paths: HashSet<String> = HashSet::new();

        // Normalize current path for exclusions
        let clean_current = current_path
            .strip_prefix("file://")
            .unwrap_or(&current_path)
            .to_string();
        seen_paths.insert(current_path.clone());
        seen_paths.insert(clean_current.clone());

        // 1. Match other videos in available_videos belonging to the same folder
        for v in &self.available_videos {
            let matches_folder = if folder_name == "Misc" {
                v.folder.as_deref() == Some("Misc")
            } else {
                resolve_folder_key(&v.path) == folder_name
            };

            if matches_folder {
                let v_clean = v.path.strip_prefix("file://").unwrap_or(&v.path);
                if !seen_paths.contains(&v.path) && !seen_paths.contains(v_clean) {
                    seen_paths.insert(v.path.clone());
                    seen_paths.insert(v_clean.to_string());
                    candidate_videos.push((v.path.clone(), format::format_video_title(&v.path)));
                }
            }
        }

        // 2. Also check immediate video files in the parent directory on disk (if available)
        let disk_path = std::path::Path::new(&clean_current);
        if let Some(parent_dir) = disk_path.parent() {
            if parent_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(parent_dir) {
                    let mut dir_entries: Vec<std::path::PathBuf> = entries
                        .filter_map(|e| e.ok().map(|ent| ent.path()))
                        .filter(|p| p.is_file() && wazoo_scanner::is_video_file(p))
                        .collect();
                    dir_entries.sort();
                    for file_path in dir_entries {
                        let path_str = file_path.to_string_lossy().to_string();
                        if !seen_paths.contains(&path_str) {
                            seen_paths.insert(path_str.clone());
                            let title = format::format_video_title(&path_str);
                            candidate_videos.push((path_str, title));
                        }
                    }
                }
            }
        }

        // Natural sort by path/title
        candidate_videos.sort_by(|a, b| a.0.cmp(&b.0));

        (Some(folder_name), candidate_videos)
    }
}

fn resolve_folder_key(path: &str) -> String {
    let folder = format::format_video_folder(path);
    if !folder.is_empty() {
        return folder;
    }
    let basename = format::folder_basename(path);
    if basename.is_empty() {
        "Other".to_string()
    } else {
        basename.to_string()
    }
}
