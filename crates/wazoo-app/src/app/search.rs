/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Search Folder Selection & Library Count Notices
 *
 * Helpers for folder filter chip selection, total library video counts,
 * and displaying toast notifications for search/scan result deltas.
 */

use super::{LONG_TOAST_SECS, WazooApp};
use crate::format;

impl WazooApp {
    pub(crate) fn total_video_count(&self) -> usize {
        self.db
            .get_video_count()
            .unwrap_or(self.available_videos.len())
    }

    pub fn is_all_folder(&self, folder: &str) -> bool {
        wazoo_core::i18n::is_all_folder(folder)
    }

    pub fn is_all_search_selected(&self) -> bool {
        self.search.selected_folders.is_empty()
            || (self.search.selected_folders.len() == 1
                && self.is_all_folder(&self.search.selected_folders[0]))
    }

    pub(crate) fn reset_search_folder_selection(&mut self) {
        self.search.selected_folders.clear();
        self.search.selected_folder = self.t("common.all");
    }

    pub(crate) fn toggle_search_folder(&mut self, folder: &str) {
        if self.is_all_folder(folder) {
            self.reset_search_folder_selection();
            return;
        }

        if let Some(pos) = self
            .search
            .selected_folders
            .iter()
            .position(|f| f == folder)
        {
            self.search.selected_folders.remove(pos);
        } else {
            self.search.selected_folders.push(folder.to_string());
        }

        if self.search.selected_folders.is_empty() {
            self.search.selected_folder = self.t("common.all");
        } else if self.search.selected_folders.len() == 1 {
            self.search.selected_folder = self.search.selected_folders[0].clone();
        } else {
            self.search.selected_folder = self.search.selected_folders.join(", ");
        }
    }

    pub(crate) fn show_video_totals_notice(&mut self, total: usize, folder_label: &str) {
        if total == 0 {
            let msg = if self.is_all_folder(folder_label) {
                self.t("wazoo.no_videos_found")
            } else {
                let folder_clean = format::folder_basename(folder_label);
                self.t_with("wazoo.no_files_found_in", &[("folder", folder_clean)])
            };
            self.overlay.toast_message = Some(msg);
            self.overlay.toast_time_remaining = LONG_TOAST_SECS;
            return;
        }

        let total_str = format::format_number(total);
        let base_msg = self.t_with("wazoo.total_files", &[("total", &total_str)]);

        let msg = if self.last_total_videos != 0 && total > self.last_total_videos {
            let diff = total - self.last_total_videos;
            let diff_str = format::format_number(diff);
            let diff_msg = self.t_with("wazoo.more_than_before", &[("diff", &diff_str)]);
            format!("{base_msg}\n{diff_msg}")
        } else if self.last_total_videos != 0 && total < self.last_total_videos {
            let diff = self.last_total_videos - total;
            let diff_str = format::format_number(diff);
            let diff_msg = self.t_with("wazoo.less_than_before", &[("diff", &diff_str)]);
            format!("{base_msg}\n{diff_msg}")
        } else {
            base_msg
        };

        self.last_total_videos = total;
        self.overlay.toast_message = Some(msg);
        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
    }
}
