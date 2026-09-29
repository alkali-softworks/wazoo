/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Search Modal Component
 *
 * Folder filters, tag chips, and text search input.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};

fn estimate_chip_width(label: &str) -> f32 {
    let text_width: f32 = label
        .chars()
        .map(|c| {
            if c.is_ascii() {
                if c.is_ascii_uppercase() || matches!(c, 'm' | 'w' | 'M' | 'W' | '@' | '%') {
                    9.5
                } else if matches!(
                    c,
                    'i' | 'l' | 'j' | 't' | 'f' | '!' | '.' | ':' | ';' | '\'' | ' '
                ) {
                    4.5
                } else {
                    7.5
                }
            } else {
                13.0
            }
        })
        .sum();
    // 24px horizontal padding (12 left + 12 right) + 8px row spacing
    text_width + 32.0
}

impl WazooApp {
    pub fn view_search_modal(&self) -> Element<'_, Message> {
        let mut chip_items: Vec<(String, Element<'_, Message>)> = Vec::new();

        let all_label = self.t("common.all");
        let is_all_selected = self.is_all_search_selected();
        chip_items.push((
            all_label.clone(),
            button(text(all_label.clone()).size(13))
                .style(theme::folder_chip_style(is_all_selected))
                .on_press(Message::SelectSearchFolder(all_label))
                .padding([4, 12])
                .into(),
        ));

        for folder in &self.settings.media_folders {
            let label = format::ucwords(format::folder_basename(folder));
            let is_selected = self.selected_search_folders.contains(folder);
            chip_items.push((
                label.clone(),
                button(text(label).size(13))
                    .style(theme::folder_chip_style(is_selected))
                    .on_press(Message::ToggleSearchFolder(folder.clone()))
                    .padding([4, 12])
                    .into(),
            ));
        }

        if self.db.has_misc_videos().unwrap_or(false) {
            let is_misc_selected = self.selected_search_folders.contains(&"Misc".to_string());
            let misc_label = self.t("common.miscellaneous");
            chip_items.push((
                misc_label.clone(),
                button(text(misc_label).size(13))
                    .style(theme::folder_chip_style(is_misc_selected))
                    .on_press(Message::ToggleSearchFolder("Misc".to_string()))
                    .padding([4, 12])
                    .into(),
            ));
        }

        let max_row_width = 440.0;
        let mut folders_col = column![].spacing(8);
        let mut current_row = row![].spacing(8).align_y(Alignment::Center);
        let mut current_width = 0.0;
        let mut row_count = 0;

        for (label, chip_elem) in chip_items {
            let chip_width = estimate_chip_width(&label);
            if current_width + chip_width > max_row_width && current_width > 0.0 {
                folders_col = folders_col.push(current_row);
                current_row = row![].spacing(8).align_y(Alignment::Center);
                current_width = 0.0;
                row_count += 1;
            }
            current_row = current_row.push(chip_elem);
            current_width += chip_width;
        }

        if current_width > 0.0 {
            folders_col = folders_col.push(current_row);
            row_count += 1;
        }

        let folders_widget: Element<'_, Message> = if row_count > 4 {
            scrollable(folders_col)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default(),
                ))
                .height(Length::Fixed(135.0))
                .into()
        } else {
            folders_col.into()
        };

        let mut tags_row = row![].spacing(6).align_y(Alignment::Center);

        for (idx, tag) in self.search_tags.iter().enumerate() {
            let chip = container(
                row![
                    text(tag).size(13).color(iced::Color::WHITE),
                    button(text("✕").size(10))
                        .style(theme::tag_delete_button_style)
                        .on_press(Message::RemoveSearchTag(idx))
                        .padding([0, 2]),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([2, 6])
            .style(theme::tag_chip_style);

            tags_row = tags_row.push(chip);
        }

        let placeholder = if self.search_tags.is_empty() {
            self.t("search.placeholder")
        } else {
            String::new()
        };

        let input_widget = text_input(&placeholder, &self.search_input)
            .id("search_input")
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::PerformSearch)
            .style(theme::transparent_input_style)
            .padding([4, 6])
            .width(Length::Fill);

        tags_row = tags_row.push(input_widget);

        let tag_box = container(tags_row)
            .padding([3, 8])
            .style(theme::tag_input_box_style)
            .width(Length::Fill);

        let search_bar_row = row![
            tag_box,
            button(text("🔍").size(16))
                .style(theme::search_button_style)
                .on_press(Message::PerformSearch)
                .padding([8, 12]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let card = container(
            column![
                row![
                    text(self.t("search.folder"))
                        .size(14)
                        .color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseSearchModal),
                ]
                .align_y(Alignment::Center),
                folders_widget,
                search_bar_row,
                text(self.t_with(
                    "settings.total_videos",
                    &[("count", &format::format_number(self.available_videos.len()))]
                ))
                .size(12)
                .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(14)
            .width(Length::Fixed(480.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSearchModal)
    }
}
