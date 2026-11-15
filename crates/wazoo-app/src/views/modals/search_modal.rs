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
use crate::wrap::Wrap;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text, text_input, toggler},
};

impl WazooApp {
    pub fn view_search_modal(&self) -> Element<'_, Message> {
        let mut folder_chips: Vec<Element<'_, Message>> = Vec::new();

        let all_label = self.t("common.all");
        let is_all_selected = self.is_all_search_selected();
        folder_chips.push(
            button(text(all_label.clone()).size(13))
                .style(theme::folder_chip_style(is_all_selected))
                .on_press(Message::SelectSearchFolder(all_label))
                .padding([4, 12])
                .into(),
        );

        for folder in &self.settings.media_folders {
            let label = format::ucwords(format::folder_basename(folder));
            let is_selected = self.search.selected_folders.contains(folder);
            folder_chips.push(
                button(text(label).size(13))
                    .style(theme::folder_chip_style(is_selected))
                    .on_press(Message::ToggleSearchFolder(folder.clone()))
                    .padding([4, 12])
                    .into(),
            );
        }

        if self.db.has_misc_videos().unwrap_or(false) {
            let is_misc_selected = self.search.selected_folders.contains(&"Misc".to_string());
            let misc_label = self.t("common.miscellaneous");
            folder_chips.push(
                button(text(misc_label).size(13))
                    .style(theme::folder_chip_style(is_misc_selected))
                    .on_press(Message::ToggleSearchFolder("Misc".to_string()))
                    .padding([4, 12])
                    .into(),
            );
        }

        let folders_wrap = Wrap::with_elements(folder_chips)
            .spacing(8)
            .vertical_spacing(8)
            .align_items(Alignment::Center);

        let folders_widget: Element<'_, Message> = if self.settings.media_folders.len() > 12 {
            scrollable(folders_wrap)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default(),
                ))
                .height(Length::Fixed(135.0))
                .into()
        } else {
            folders_wrap.into()
        };

        let mut tag_elements: Vec<Element<'_, Message>> = Vec::new();

        for (idx, tag) in self.search.tags.iter().enumerate() {
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

            tag_elements.push(chip.into());
        }

        let placeholder = if self.search.tags.is_empty() {
            self.t("search.placeholder")
        } else {
            String::new()
        };

        let input_widget = text_input(&placeholder, &self.search.input)
            .id("search_input")
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::PerformSearch)
            .style(theme::transparent_input_style)
            .padding([4, 6]);

        tag_elements.push(input_widget.into());

        let tags_wrap = Wrap::with_elements(tag_elements)
            .spacing(6)
            .vertical_spacing(6)
            .align_items(Alignment::Center)
            .fill_last(70.0);

        let tag_box = container(tags_wrap)
            .padding([6, 8])
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

        let include_metadata = self.settings.search_include_metadata;
        let switch = toggler(include_metadata)
            .on_toggle(Message::SetSearchIncludeMetadata)
            .size(16)
            .style(|_theme, status| {
                let (bg, bg_border, fg) = match status {
                    toggler::Status::Active { is_toggled }
                    | toggler::Status::Hovered { is_toggled } => {
                        let is_hovered = matches!(status, toggler::Status::Hovered { .. });
                        if is_toggled {
                            (
                                if is_hovered {
                                    theme::COLOR_PRIMARY_BORDER
                                } else {
                                    theme::COLOR_PRIMARY
                                },
                                theme::COLOR_PRIMARY,
                                iced::Color::WHITE,
                            )
                        } else {
                            (
                                if is_hovered {
                                    theme::COLOR_BTN_HOVER
                                } else {
                                    theme::COLOR_BTN_BG
                                },
                                theme::COLOR_BORDER,
                                iced::Color::from_rgb(0.85, 0.85, 0.85),
                            )
                        }
                    }
                    toggler::Status::Disabled { .. } => (
                        theme::COLOR_BTN_BG,
                        theme::COLOR_BORDER,
                        theme::COLOR_TEXT_MUTED,
                    ),
                };
                toggler::Style {
                    background: bg.into(),
                    background_border_width: 1.0,
                    background_border_color: bg_border,
                    foreground: fg.into(),
                    foreground_border_width: 0.0,
                    foreground_border_color: iced::Color::TRANSPARENT,
                    text_color: None,
                    border_radius: None,
                    padding_ratio: 0.15,
                }
            });

        let label_button = |label: String, is_active: bool, target: bool| {
            button(text(label).size(12))
                .style(move |_theme: &iced::Theme, status: button::Status| {
                    let text_color = if is_active || matches!(status, button::Status::Hovered) {
                        iced::Color::WHITE
                    } else {
                        theme::COLOR_TEXT_MUTED
                    };
                    button::Style {
                        background: None,
                        text_color,
                        ..Default::default()
                    }
                })
                .on_press(Message::SetSearchIncludeMetadata(target))
                .padding(0)
        };

        let search_by_toggle = row![
            text(self.t("search.search_by"))
                .size(12)
                .color(theme::COLOR_TEXT_MUTED),
            label_button(self.t("search.filename"), !include_metadata, false),
            switch,
            label_button(self.t("search.metadata"), include_metadata, true),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let footer_row = row![
            text(self.t_with(
                "settings.total_videos",
                &[("count", &format::format_number(self.available_videos.len()))]
            ))
            .size(12)
            .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            search_by_toggle,
        ]
        .align_y(Alignment::Center);

        let modal_width = 640.0;
        let card_padding = 24.0;

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
                footer_row,
            ]
            .spacing(16)
            .width(Length::Fixed(modal_width)),
        )
        .padding(card_padding)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSearchModal)
    }
}
