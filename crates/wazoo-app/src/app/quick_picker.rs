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
use crate::state::modals::{QuickPickerCache, QuickPickerEntry};
use iced::Task;
use std::collections::{HashMap, HashSet};

impl WazooApp {
    pub(crate) fn open_quick_picker(&mut self) -> Task<Message> {
        let (Some(source_path), Some(folder_name), mut candidates) =
            self.quick_picker_library_videos()
        else {
            self.overlay.show_toast(self.t("wazoo.no_files_found"));
            return Task::none();
        };

        let clean_current = source_path
            .strip_prefix("file://")
            .unwrap_or(&source_path)
            .to_string();
        let is_url = clean_current.contains("://");

        let mut seen_paths = HashSet::with_capacity(candidates.len() * 2 + 2);
        seen_paths.insert(source_path.clone());
        seen_paths.insert(clean_current.clone());
        for (p, _) in &candidates {
            seen_paths.insert(p.clone());
        }

        // Only block to check disk when the library itself returned zero siblings for a local path
        // (e.g. a standalone CLI file with unlisted sibling files in the same directory).
        let should_scan_disk_async = if candidates.is_empty() {
            if !is_url {
                if let Some(parent_dir) = std::path::Path::new(&clean_current).parent() {
                    let disk_candidates = scan_parent_dir_siblings(parent_dir, &seen_paths);
                    candidates.extend(disk_candidates);
                }
            }
            false
        } else {
            !is_url
        };

        if candidates.is_empty() {
            self.overlay.show_toast(self.t("wazoo.no_files_found"));
            return Task::none();
        }

        candidates.sort_by(|a, b| a.0.cmp(&b.0));

        let entries = self.build_quick_picker_entries(candidates);
        let cache = QuickPickerCache {
            source_path: source_path.clone(),
            folder: Some(folder_name.clone()),
            entries,
        };

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
        self.modals.quick_picker_cache = Some(cache);

        let disk_scan_task = if should_scan_disk_async {
            if let Some(parent_dir) = std::path::Path::new(&clean_current)
                .parent()
                .map(|p| p.to_path_buf())
            {
                let source_for_msg = source_path;
                let folder_for_msg = folder_name;
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            scan_parent_dir_siblings(&parent_dir, &seen_paths)
                        })
                        .await
                        .unwrap_or_default()
                    },
                    move |disk_candidates| Message::QuickPickerDiskSiblingsLoaded {
                        source_path: source_for_msg,
                        folder_name: folder_for_msg,
                        candidates: disk_candidates,
                    },
                )
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };

        Task::batch([
            exit_task,
            disk_scan_task,
            iced::widget::operation::focus("quick_picker_search_input"),
        ])
    }

    pub(crate) fn merge_quick_picker_disk_siblings(
        &mut self,
        source_path: &str,
        folder_name: String,
        disk_candidates: Vec<(String, String)>,
    ) {
        if !self.modals.quick_picker || disk_candidates.is_empty() {
            return;
        }
        let Some(existing) = self.modals.quick_picker_cache.as_ref() else {
            return;
        };
        if existing.source_path != source_path
            || existing.folder.as_deref() != Some(folder_name.as_str())
        {
            return;
        }

        let mut seen: HashSet<String> = existing.entries.iter().map(|e| e.path.clone()).collect();
        seen.insert(source_path.to_string());
        if let Some(stripped) = source_path.strip_prefix("file://") {
            seen.insert(stripped.to_string());
        }

        let new_items: Vec<(String, String)> = disk_candidates
            .into_iter()
            .filter(|(p, _)| !seen.contains(p))
            .collect();
        if new_items.is_empty() {
            return;
        }

        let new_entries = self.build_quick_picker_entries(new_items);
        if let Some(cache) = self.modals.quick_picker_cache.as_mut() {
            cache.entries.extend(new_entries);
            cache.entries.sort_by(|a, b| a.path.cmp(&b.path));
        }
    }

    /// Returns the folder name (or formatted title) and the list of other videos (path, title)
    /// from the folder of the video currently playing in the active/focused player.
    pub fn quick_picker_videos(&self) -> (Option<String>, Vec<(String, String)>) {
        let (Some(current_path), Some(folder_name), mut candidate_videos) =
            self.quick_picker_library_videos()
        else {
            return (None, Vec::new());
        };

        let clean_current = current_path.strip_prefix("file://").unwrap_or(&current_path);
        if !clean_current.contains("://") {
            if let Some(parent_dir) = std::path::Path::new(clean_current).parent() {
                let mut seen_paths = HashSet::with_capacity(candidate_videos.len() * 2 + 2);
                seen_paths.insert(current_path.clone());
                seen_paths.insert(clean_current.to_string());
                for (p, _) in &candidate_videos {
                    let p_clean = p.strip_prefix("file://").unwrap_or(p);
                    seen_paths.insert(p.clone());
                    seen_paths.insert(p_clean.to_string());
                }
                candidate_videos.extend(scan_parent_dir_siblings(parent_dir, &seen_paths));
            }
        }

        // Natural sort by path/title
        candidate_videos.sort_by(|a, b| a.0.cmp(&b.0));

        (Some(folder_name), candidate_videos)
    }

    /// Builds a full `QuickPickerCache` from current state (fallback when the modal
    /// is opened or rendered without calling `open_quick_picker`).
    pub(crate) fn build_quick_picker_cache(&self) -> QuickPickerCache {
        let source_path = self
            .focused_player()
            .map(|p| p.state.path.clone())
            .unwrap_or_default();
        let (folder, candidates) = self.quick_picker_videos();
        let entries = self.build_quick_picker_entries(candidates);
        QuickPickerCache {
            source_path,
            folder,
            entries,
        }
    }

    fn quick_picker_library_videos(
        &self,
    ) -> (Option<String>, Option<String>, Vec<(String, String)>) {
        let Some(current_path) = self
            .focused_player()
            .map(|p| p.state.path.clone())
            .filter(|s| !s.trim().is_empty())
        else {
            return (None, None, Vec::new());
        };

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

        let clean_current = current_path.strip_prefix("file://").unwrap_or(&current_path);
        seen_paths.insert(current_path.clone());
        seen_paths.insert(clean_current.to_string());

        let is_misc = folder_name == "Misc";
        // Memoize `format_video_folder` per parent directory so multi-thousand-video libraries
        // run regex cleaning once per unique folder instead of once per video.
        let mut folder_cache: HashMap<&str, Option<String>> = HashMap::new();

        for v in &self.available_videos {
            let matches_folder = if is_misc {
                v.folder.as_deref() == Some("Misc")
            } else {
                let parent_dir = v.path.rsplit_once(['/', '\\']).map(|(d, _)| d).unwrap_or("");
                let cached = folder_cache.entry(parent_dir).or_insert_with(|| {
                    let folder = format::format_video_folder(&v.path);
                    if folder.is_empty() {
                        None
                    } else {
                        Some(folder)
                    }
                });
                match cached {
                    Some(folder) => folder == &folder_name,
                    None => {
                        let basename = format::folder_basename(&v.path);
                        if basename.is_empty() {
                            folder_name == "Other"
                        } else {
                            basename == folder_name
                        }
                    }
                }
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

        (Some(current_path), Some(folder_name), candidate_videos)
    }

    fn build_quick_picker_entries(
        &self,
        items: Vec<(String, String)>,
    ) -> Vec<QuickPickerEntry> {
        let meta_index = self.metadata_by_path();
        items
            .into_iter()
            .map(|(path, title)| {
                let meta = meta_index
                    .get(path.as_str())
                    .copied()
                    .or_else(|| self.get_metadata_for_path(&path));
                let badge = meta.and_then(|m| {
                    if let Some(y) = m.year {
                        Some(y.to_string())
                    } else {
                        m.genres.first().cloned()
                    }
                });
                let title_lower = title.to_lowercase();
                QuickPickerEntry {
                    path,
                    title,
                    title_lower,
                    badge,
                }
            })
            .collect()
    }
}

fn scan_parent_dir_siblings(
    parent_dir: &std::path::Path,
    seen_paths: &HashSet<String>,
) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(parent_dir) else {
        return Vec::new();
    };

    let mut dir_entries: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        // Check extension before inspecting file_type/stat so non-video files cause 0 syscalls.
        if !wazoo_scanner::is_video_file(&path) {
            continue;
        }
        let is_file = match entry.file_type() {
            Ok(ft) if ft.is_file() => true,
            Ok(ft) if ft.is_symlink() => path.is_file(),
            Ok(_) => false,
            Err(_) => path.is_file(),
        };
        if is_file {
            dir_entries.push(path);
        }
    }

    dir_entries.sort();
    let mut out = Vec::with_capacity(dir_entries.len());
    for file_path in dir_entries {
        let path_str = file_path.to_string_lossy().to_string();
        if !seen_paths.contains(&path_str) {
            let title = format::format_video_title(&path_str);
            out.push((path_str, title));
        }
    }
    out
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
