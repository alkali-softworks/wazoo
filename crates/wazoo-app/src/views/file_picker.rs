/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * File Browser Drawer
 * 
 * Displays an expandable side drawer grouping indexed media by folder, featuring search
 * filtering and direct file selection into the active playback tile.
 */

use std::collections::BTreeMap;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};
use wazoo_core::VideoRecord;
use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilePickerItem {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilePickerGroup {
    pub folder: String,
    pub files: Vec<FilePickerItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PrecomputedVideoMeta {
    pub path: String,
    pub title: String,
    pub title_lower: String,
    pub folder_key: String,
    pub folder_lower: String,
}

impl WazooApp {
    pub(crate) fn ensure_file_picker_meta(&mut self) {
        let needs_rebuild = self.file_picker_entries.len() != self.available_videos.len()
            || (!self.available_videos.is_empty()
                && self.file_picker_entries.first().map(|e| &e.path) != self.available_videos.first().map(|v| &v.path));
        if needs_rebuild {
            self.file_picker_entries = self
                .available_videos
                .iter()
                .map(|v| {
                    let folder = format::format_video_folder(&v.path);
                    let folder_key = if folder.is_empty() { "Other".to_string() } else { folder };
                    let title = format::format_video_title(&v.path);
                    let folder_lower = folder_key.to_lowercase();
                    let title_lower = title.to_lowercase();
                    PrecomputedVideoMeta {
                        path: v.path.clone(),
                        title,
                        title_lower,
                        folder_key,
                        folder_lower,
                    }
                })
                .collect();
        }
    }

    pub(crate) fn apply_file_picker_search(&mut self) {
        self.ensure_file_picker_meta();

        let search_filter = self.file_picker_search.trim().to_lowercase();
        let search_clean = search_filter.replace(['_', '-'], " ");
        let search_words: Vec<&str> = search_clean.split_whitespace().collect();

        let mut grouped_map: BTreeMap<String, Vec<FilePickerItem>> = BTreeMap::new();

        if search_words.is_empty() {
            self.expanded_folders.clear();
            for entry in &self.file_picker_entries {
                grouped_map
                    .entry(entry.folder_key.clone())
                    .or_default()
                    .push(FilePickerItem {
                        path: entry.path.clone(),
                        title: entry.title.clone(),
                    });
            }
        } else {
            for entry in &self.file_picker_entries {
                let matches_all_words = search_words
                    .iter()
                    .all(|w| entry.folder_lower.contains(w) || entry.title_lower.contains(w));
                if !matches_all_words {
                    continue;
                }
                grouped_map
                    .entry(entry.folder_key.clone())
                    .or_default()
                    .push(FilePickerItem {
                        path: entry.path.clone(),
                        title: entry.title.clone(),
                    });
            }

            for folder in grouped_map.keys() {
                self.expanded_folders.insert(folder.clone());
            }
        }

        self.file_picker_groups = grouped_map
            .into_iter()
            .map(|(folder, files)| FilePickerGroup { folder, files })
            .collect();
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn filter_and_group_videos_for_picker(&self) -> BTreeMap<String, Vec<&VideoRecord>> {
        let search_filter = self.file_picker_search.trim().to_lowercase();
        let search_clean = search_filter.replace(['_', '-'], " ");
        let search_words: Vec<&str> = search_clean.split_whitespace().collect();

        // Group videos by folder
        let mut grouped: BTreeMap<String, Vec<&VideoRecord>> = BTreeMap::new();
        for v in &self.available_videos {
            let folder = format::format_video_folder(&v.path);
            let folder_key = if folder.is_empty() { "Other".to_string() } else { folder };
            let title = format::format_video_title(&v.path);

            if !search_words.is_empty() {
                let folder_lower = folder_key.to_lowercase();
                let title_lower = title.to_lowercase();
                let matches_all_words = search_words
                    .iter()
                    .all(|w| folder_lower.contains(w) || title_lower.contains(w));
                if !matches_all_words {
                    continue;
                }
            }

            grouped.entry(folder_key).or_default().push(v);
        }

        grouped
    }

    pub(crate) fn view_file_picker(&self) -> Element<'_, Message> {
        let groups = &self.file_picker_groups;

        let mut folders_col = column![].spacing(6);
        for group in groups {
            let count = group.files.len();
            let is_expanded = self.expanded_folders.contains(&group.folder);
            let chevron = if is_expanded { "▼" } else { "▶" };

            let header_btn = button(
                row![
                    text(chevron).size(10).color(theme::COLOR_PRIMARY),
                    text(group.folder.clone()).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    text(format!("({})", format::format_number(count))).size(12).color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fixed(4.0)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .style(theme::folder_group_header_style)
            .on_press(Message::ToggleFolderCollapse(group.folder.clone()))
            .padding([7, 10])
            .width(Length::Fill);

            if is_expanded {
                let mut files_list = column![].spacing(2);
                for f in &group.files {
                    let p_clone = f.path.clone();
                    files_list = files_list.push(
                        button(
                            row![
                                Space::new().width(Length::Fixed(12.0)),
                                text(&f.title).size(12).color(iced::Color::WHITE),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .style(theme::menu_item_style)
                        .on_press(Message::PlayFileInFocused(p_clone))
                        .padding([4, 8])
                        .width(Length::Fill),
                    );
                }
                folders_col = folders_col.push(column![header_btn, files_list].spacing(4));
            } else {
                folders_col = folders_col.push(header_btn);
            }
        }

        let scrollable_folders = container(folders_col)
            .padding(iced::Padding {
                top: 0.0,
                right: 20.0, // Dedicated gutter so vertical scrollbar never overlaps text
                bottom: 0.0,
                left: 0.0,
            })
            .width(Length::Fill);

        let is_confined = !self.is_all_folder(&self.active_search_folder);
        let mut header_row = row![
            text(self.t_with("settings.total_videos", &[("count", &format::format_number(self.available_videos.len()))]))
                .size(17)
                .color(iced::Color::WHITE),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if is_confined {
            let folder_label = self
                .active_search_folder
                .split(['/', '\\'])
                .filter(|s| !s.is_empty())
                .next_back()
                .unwrap_or(&self.active_search_folder);

            let display_name = if folder_label.chars().count() > 18 {
                format!("{}...", folder_label.chars().take(16).collect::<String>())
            } else {
                folder_label.to_string()
            };

            let badge = button(
                row![
                    text(display_name).size(11).color(iced::Color::WHITE),
                    text("✕").size(9).color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .style(theme::folder_chip_style(true))
            .on_press(Message::ResetSearchFolder)
            .padding([3, 8]);

            header_row = header_row.push(badge);
        }

        header_row = header_row.push(Space::new().width(Length::Fill));
        header_row = header_row.push(
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::ToggleFilePicker),
        );

        let content = column![
            Space::new().height(Length::Fixed(24.0)),
            header_row,
            text_input(&self.t("file_picker.search_placeholder"), &self.file_picker_search)
                .on_input(Message::FilePickerSearchChanged)
                .on_submit(Message::ApplyFilePickerSearch)
                .style(theme::dark_input_style)
                .padding(8),
            scrollable(scrollable_folders)
                .height(Length::Fill)
                .width(Length::Fill),
        ]
        .spacing(12)
        .padding(16);

        container(content)
            .width(Length::Fixed(420.0))
            .height(Length::Fill)
            .style(theme::file_picker_drawer_style)
            .into()
    }
}
