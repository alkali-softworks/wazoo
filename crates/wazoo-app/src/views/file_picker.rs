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

impl WazooApp {
    pub(crate) fn view_file_picker(&self) -> Element<'_, Message> {
        let search_filter = self.file_picker_search.to_lowercase();

        // Group videos by folder
        let mut grouped: BTreeMap<String, Vec<&VideoRecord>> = BTreeMap::new();
        for v in &self.available_videos {
            let title = format::format_video_title(&v.path);
            if !search_filter.is_empty() && !title.to_lowercase().contains(&search_filter) {
                continue;
            }
            let folder = format::format_video_folder(&v.path);
            let folder_key = if folder.is_empty() { "Other".to_string() } else { folder };
            grouped.entry(folder_key).or_default().push(v);
        }

        let mut folders_col = column![].spacing(6);
        for (folder, files) in grouped {
            let count = files.len();
            let is_expanded = self.expanded_folders.contains(&folder);
            let chevron = if is_expanded { "▼" } else { "▶" };

            let header_btn = button(
                row![
                    text(chevron).size(10).color(theme::COLOR_PRIMARY),
                    text(folder.clone()).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    text(format!("({count})")).size(12).color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fixed(4.0)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .style(theme::folder_group_header_style)
            .on_press(Message::ToggleFolderCollapse(folder))
            .padding([7, 10])
            .width(Length::Fill);

            if is_expanded {
                let mut files_list = column![].spacing(2);
                for f in files {
                    let file_title = format::format_video_title(&f.path);
                    let p_clone = f.path.clone();
                    files_list = files_list.push(
                        button(
                            row![
                                Space::new().width(Length::Fixed(12.0)),
                                text(file_title).size(12).color(iced::Color::WHITE),
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

        let content = column![
            Space::new().height(Length::Fixed(24.0)),
            row![
                text(self.t("common.files")).size(18).color(iced::Color::WHITE),
                Space::new().width(Length::Fill),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::ToggleFilePicker),
            ]
            .align_y(Alignment::Center),
            text_input(&self.t("file_picker.search_placeholder"), &self.file_picker_search)
                .on_input(Message::FilePickerSearchChanged)
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
