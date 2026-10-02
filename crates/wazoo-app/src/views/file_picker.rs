/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * File Browser Drawer
 *
 * Displays an expandable side drawer grouping indexed media by folder, featuring search
 * filtering and direct file selection into the active playback tile.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use crate::wrap::Wrap;
use iced::{
    Alignment, Element, Length,
    widget::{Space, Stack, button, column, container, row, scrollable, text, text_input},
};
use std::collections::BTreeMap;

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

/// Matches a string against an anchor/wildcard pattern.
///
/// Supported patterns:
/// - `^prefix`   : text starts with prefix (e.g. `^k`)
/// - `suffix$`   : text ends with suffix (e.g. `s01$`)
/// - `^exact$`   : text exactly matches exact (e.g. `^bleach$`)
/// - `foo*bar`   : wildcard match (`foo` followed by `bar`)
/// - `^foo*bar$` : wildcard match anchored at start and end
/// - `substring` : contains substring (standard fallback)
pub(crate) fn pattern_matches(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() || pattern == "^" || pattern == "$" || pattern == "^$" || pattern == "*" {
        return true;
    }

    let has_start_anchor = pattern.starts_with('^');
    let has_end_anchor = pattern.len() > 1 && pattern.ends_with('$');

    let core = match (has_start_anchor, has_end_anchor) {
        (true, true) => {
            if pattern.len() <= 2 {
                return true;
            }
            &pattern[1..pattern.len() - 1]
        }
        (true, false) => &pattern[1..],
        (false, true) => &pattern[..pattern.len() - 1],
        (false, false) => pattern,
    };

    if core.is_empty() {
        return true;
    }

    if !core.contains('*') {
        return match (has_start_anchor, has_end_anchor) {
            (true, true) => text == core,
            (true, false) => text.starts_with(core),
            (false, true) => text.ends_with(core),
            (false, false) => text.contains(core),
        };
    }

    // Handles wildcard '*'
    let parts: Vec<&str> = core.split('*').collect();
    let mut remainder = text;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && has_start_anchor {
            if !remainder.starts_with(part) {
                return false;
            }
            remainder = &remainder[part.len()..];
        } else if i == parts.len() - 1 && has_end_anchor {
            if !remainder.ends_with(part) {
                return false;
            }
        } else {
            match remainder.find(part) {
                Some(idx) => {
                    remainder = &remainder[idx + part.len()..];
                }
                None => return false,
            }
        }
    }

    true
}

impl WazooApp {
    pub(crate) fn file_picker_filtered_videos(&mut self) -> Option<Vec<wazoo_core::VideoRecord>> {
        let filter = self.drawers.file_picker_search.trim();
        if filter.is_empty() {
            return None;
        }

        self.apply_file_picker_search();

        let matching_paths: std::collections::HashSet<&str> = self
            .drawers
            .file_picker_groups
            .iter()
            .flat_map(|g| &g.files)
            .map(|f| f.path.as_str())
            .collect();

        if matching_paths.is_empty() {
            return None;
        }

        Some(
            self.available_videos
                .iter()
                .filter(|v| matching_paths.contains(v.path.as_str()))
                .cloned()
                .collect(),
        )
    }
    pub(crate) fn ensure_file_picker_meta(&mut self) {
        let needs_rebuild = self.drawers.file_picker_entries.len() != self.available_videos.len()
            || (!self.available_videos.is_empty()
                && self.drawers.file_picker_entries.first().map(|e| &e.path)
                    != self.available_videos.first().map(|v| &v.path));
        if needs_rebuild {
            self.drawers.file_picker_entries = self
                .available_videos
                .iter()
                .map(|v| {
                    let folder_key = if v.folder.as_deref() == Some("Misc") {
                        "Misc".to_string()
                    } else {
                        let folder = format::format_video_folder(&v.path);
                        if folder.is_empty() {
                            "Other".to_string()
                        } else {
                            folder
                        }
                    };
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

        let search_filter = self.drawers.file_picker_search.trim().to_lowercase();
        let search_clean = search_filter.replace(['_', '-'], " ");
        let search_words: Vec<&str> = search_clean.split_whitespace().collect();

        let mut grouped_map: BTreeMap<String, Vec<FilePickerItem>> = BTreeMap::new();

        if search_words.is_empty() {
            self.drawers.expanded_folders.clear();
            for entry in &self.drawers.file_picker_entries {
                grouped_map
                    .entry(entry.folder_key.clone())
                    .or_default()
                    .push(FilePickerItem {
                        path: entry.path.clone(),
                        title: entry.title.clone(),
                    });
            }
        } else {
            let is_whole_anchored = (search_filter.starts_with('^')
                && !search_filter.starts_with("^^"))
                || (search_filter.ends_with('$') && !search_filter.ends_with("$$"));

            for entry in &self.drawers.file_picker_entries {
                let matches = if is_whole_anchored {
                    pattern_matches(&search_filter, &entry.folder_lower)
                        || pattern_matches(&search_filter, &entry.title_lower)
                        || search_words.iter().all(|w| {
                            pattern_matches(w, &entry.folder_lower)
                                || pattern_matches(w, &entry.title_lower)
                        })
                } else {
                    search_words.iter().all(|w| {
                        pattern_matches(w, &entry.folder_lower)
                            || pattern_matches(w, &entry.title_lower)
                    })
                };

                if !matches {
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
                self.drawers.expanded_folders.insert(folder.clone());
            }
        }

        self.drawers.file_picker_groups = grouped_map
            .into_iter()
            .map(|(folder, files)| FilePickerGroup { folder, files })
            .collect();
    }

    pub fn view_file_picker(&self) -> Element<'_, Message> {
        let groups = &self.drawers.file_picker_groups;

        let playing_paths: std::collections::HashSet<&str> = self
            .players
            .iter()
            .map(|p| p.state.path.as_str())
            .filter(|s| !s.is_empty())
            .collect();

        let mut folders_col = column![].spacing(6);
        for group in groups {
            let count = group.files.len();
            let is_expanded = self.drawers.expanded_folders.contains(&group.folder);
            let chevron = if is_expanded { "▼" } else { "▶" };

            let display_name = if group.folder == "Misc" {
                self.t("common.miscellaneous")
            } else {
                group.folder.clone()
            };

            let header_btn = button(
                row![
                    text(chevron).size(10).color(theme::COLOR_PRIMARY),
                    text(display_name).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    text(format!("({})", format::format_number(count)))
                        .size(12)
                        .color(theme::COLOR_TEXT_MUTED),
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
                    let is_playing = !f.path.is_empty()
                        && (playing_paths.contains(f.path.as_str())
                            || self.players.iter().any(|p| {
                                !p.state.path.is_empty()
                                    && (p.state.path == f.path
                                        || std::path::Path::new(&p.state.path)
                                            == std::path::Path::new(&f.path))
                            }));
                    let p_clone = f.path.clone();

                    let row_content = if is_playing {
                        row![
                            text("▶")
                                .size(9)
                                .color(theme::COLOR_PRIMARY)
                                .width(Length::Fixed(12.0)),
                            text(&f.title)
                                .size(12)
                                .color(iced::Color::WHITE)
                                .font(iced::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                }),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    } else {
                        row![
                            Space::new().width(Length::Fixed(12.0)),
                            text(&f.title)
                                .size(12)
                                .color(iced::Color::from_rgb(0.85, 0.85, 0.85)),
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center)
                    };

                    files_list = files_list.push(
                        button(row_content)
                            .style(theme::transcript_cue_button_style(is_playing))
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

        let is_confined = !self.search.active_folders.is_empty()
            || !self.is_all_folder(&self.search.active_folder);
        let mut header_items: Vec<Element<'_, Message>> = Vec::new();
        header_items.push(
            text(self.t_with(
                "settings.total_videos",
                &[("count", &format::format_number(self.available_videos.len()))]
            ))
            .size(17)
            .color(iced::Color::WHITE)
            .into(),
        );

        if is_confined {
            let folders_to_display: Vec<&String> = if !self.search.active_folders.is_empty() {
                self.search
                    .active_folders
                    .iter()
                    .filter(|f| !self.is_all_folder(f))
                    .collect()
            } else if !self.is_all_folder(&self.search.active_folder) {
                vec![&self.search.active_folder]
            } else {
                Vec::new()
            };

            for folder in folders_to_display {
                let folder_label = if folder == "Misc" {
                    self.t("common.miscellaneous")
                } else {
                    format::folder_basename(folder).to_string()
                };

                let display_name = if folder_label.chars().count() > 24 {
                    format!("{}...", folder_label.chars().take(22).collect::<String>())
                } else {
                    folder_label.to_string()
                };

                let badge = button(
                    row![
                        text(display_name).size(13).color(iced::Color::WHITE),
                        text("✕")
                            .size(12)
                            .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
                    ]
                    .spacing(5)
                    .align_y(Alignment::Center),
                )
                .style(theme::folder_chip_style(true))
                .on_press(Message::RemoveActiveSearchFolder(folder.clone()))
                .padding([3, 8]);

                header_items.push(badge.into());
            }
        }

        let header_wrap = Wrap::with_elements(header_items)
            .spacing(8)
            .vertical_spacing(8)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        let close_btn = button(text("✕").size(14))
            .style(theme::window_control_button_style)
            .on_press(Message::ToggleFilePicker);

        let header_row = row![
            header_wrap,
            close_btn,
        ]
        .spacing(8)
        .align_y(Alignment::Start);

        let search_is_empty = self.drawers.file_picker_search.is_empty();
        let search_input = text_input(
            &self.t("file_picker.search_placeholder"),
            &self.drawers.file_picker_search,
        )
        .id("file_picker_search_input")
        .on_input(Message::FilePickerSearchChanged)
        .on_submit(Message::ApplyFilePickerSearch)
        .style(theme::dark_input_style)
        .padding(iced::Padding {
            top: 8.0,
            right: 28.0,
            bottom: 8.0,
            left: 8.0,
        })
        .width(Length::Fill);

        let mut stack_children: Vec<Element<'_, Message>> = vec![search_input.into()];

        if !search_is_empty {
            let clear_btn = button(text("✕").size(11))
                .style(theme::search_clear_button_style)
                .on_press(Message::ClearFilePickerSearch)
                .padding([2, 5]);

            let btn_container = container(clear_btn)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(Alignment::Center)
                .padding(iced::Padding {
                    right: 6.0,
                    ..Default::default()
                });

            stack_children.push(btn_container.into());
        }

        let search_bar = Stack::with_children(stack_children);

        let content = column![
            Space::new().height(Length::Fixed(14.0)),
            header_row,
            search_bar,
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
