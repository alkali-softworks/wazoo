/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Quick Video Picker Modal Component
 *
 * Fast modal picker displaying other videos from the current playing video's folder.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};

impl WazooApp {
    pub fn view_quick_picker_modal(&self) -> Element<'_, Message> {
        let (folder_opt, candidate_videos) = self.quick_picker_videos();

        // 1. Header Row
        let display_folder_name = match &folder_opt {
            Some(folder) if folder == "Misc" => self.t("common.miscellaneous"),
            Some(folder) => folder.clone(),
            None => self.t("quick_picker.title"),
        };

        // Filter candidate videos by search query if any
        let query = self.modals.quick_picker_search.trim().to_lowercase();
        let filtered_videos: Vec<(String, String)> = if query.is_empty() {
            candidate_videos.clone()
        } else {
            let clean = query.replace(['_', '-'], " ");
            let words: Vec<&str> = clean.split_whitespace().collect();
            candidate_videos
                .iter()
                .filter(|(_, title)| {
                    let t_lower = title.to_lowercase();
                    let is_whole_anchored = (query.starts_with('^') && !query.starts_with("^^"))
                        || (query.ends_with('$') && !query.ends_with("$$"));
                    if is_whole_anchored {
                        crate::views::file_picker::pattern_matches(&query, &t_lower)
                    } else {
                        words.iter().all(|w| {
                            crate::views::file_picker::pattern_matches(w, &t_lower)
                        })
                    }
                })
                .cloned()
                .collect()
        };

        let count = filtered_videos.len();

        let header_row = row![
            text("📁").size(18),
            text(display_folder_name)
                .size(18)
                .color(iced::Color::WHITE),
            container(
                text(format::format_number(count))
                    .size(12)
                    .color(theme::COLOR_TEXT_DIM)
            )
            .padding([2, 8])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                border: iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseQuickPickerModal),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // 2. Search Bar
        let search_is_empty = self.modals.quick_picker_search.is_empty();
        let search_input = text_input(
            &self.t("quick_picker.search_placeholder"),
            &self.modals.quick_picker_search,
        )
        .id("quick_picker_search_input")
        .on_input(Message::QuickPickerSearchChanged)
        .on_submit(Message::QuickPickerSubmit)
        .style(theme::dark_input_style)
        .padding(iced::Padding {
            top: 8.0,
            right: 28.0,
            bottom: 8.0,
            left: 10.0,
        })
        .width(Length::Fill);

        let mut stack_children: Vec<Element<'_, Message>> = vec![search_input.into()];

        if !search_is_empty {
            let clear_btn = button(text("✕").size(11))
                .style(theme::search_clear_button_style)
                .on_press(Message::ClearQuickPickerSearch)
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

        let search_bar = iced::widget::Stack::with_children(stack_children);

        // 3. Content List
        let content_element: Element<'_, Message> = if filtered_videos.is_empty() {
            container(
                column![
                    text(self.t("wazoo.no_files_found"))
                        .size(15)
                        .color(theme::COLOR_TEXT_DIM),
                ]
                .align_x(Alignment::Center),
            )
            .padding([30, 20])
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list = column![].spacing(6);
            for (path, title) in filtered_videos {
                let item_btn = button(
                    row![
                        text("▶")
                            .size(11)
                            .color(theme::COLOR_PRIMARY)
                            .width(Length::Fixed(14.0)),
                        text(title)
                            .size(13)
                            .color(iced::Color::WHITE),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .style(theme::bookmark_item_button_style)
                .padding([8, 12])
                .width(Length::Fill)
                .on_press(Message::PlayQuickPickerFile(path));

                list = list.push(item_btn);
            }

            let scrollable_content = container(list)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 16.0,
                    bottom: 0.0,
                    left: 2.0,
                })
                .width(Length::Fill);

            scrollable(scrollable_content)
                .height(Length::Fixed(400.0))
                .width(Length::Fill)
                .into()
        };

        let card = container(
            column![header_row, search_bar, content_element]
                .spacing(14)
                .width(Length::Fixed(580.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseQuickPickerModal)
    }
}
