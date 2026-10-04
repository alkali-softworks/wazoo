/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Bookmarks Modal Component
 *
 * Bookmark manager for saved video timestamps and searches.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, scrollable, text},
};

impl WazooApp {
    pub fn view_bookmarks_modal(&self) -> Element<'_, Message> {
        let count = self.settings.bookmarks.len();
        let header_row = row![
            text(self.t("bookmarks.title"))
                .size(20)
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
            button(
                row![
                    text("+")
                        .size(14)
                        .color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
                    text(self.t("bookmarks.bookmark_current"))
                        .size(12)
                        .color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .style(theme::primary_button_style)
            .padding([6, 12])
            .on_press(Message::AddBookmarkFocused),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseBookmarksModal),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let content_element: Element<'_, Message> = if self.settings.bookmarks.is_empty() {
            container(
                column![
                    text(self.t("bookmarks.no_bookmarks"))
                        .size(15)
                        .color(theme::COLOR_TEXT_DIM),
                    text(self.t("bookmarks.hint"))
                        .size(13)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .padding([40, 20])
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list = column![].spacing(8);
            for (idx, b) in self.settings.bookmarks.iter().enumerate() {
                let time_str = format::format_time_str(b.position_secs);
                let bookmark_clone = b.clone();

                let mut meta_row = row![
                    container(text(time_str).size(11).color(theme::COLOR_PRIMARY))
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.0, 0.9, 0.7, 0.15
                            ))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(6)
                .align_y(Alignment::Center);

                if !b.query.is_empty() {
                    meta_row = meta_row.push(
                        container(
                            text(format!("#{}", b.query))
                                .size(11)
                                .color(theme::COLOR_TEXT_MUTED),
                        )
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                1.0, 1.0, 1.0, 0.06,
                            ))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                }

                let info_col = column![text(&b.name).size(14).color(iced::Color::WHITE), meta_row,]
                    .spacing(4)
                    .width(Length::Fill);

                let play_btn = button(info_col)
                    .style(theme::bookmark_item_button_style)
                    .width(Length::Fill)
                    .padding([8, 12])
                    .on_press(Message::JumpToBookmark(bookmark_clone));

                let delete_btn = button(text("✕").size(12).color(theme::COLOR_TEXT_MUTED))
                    .style(theme::window_control_button_style)
                    .padding([8, 10])
                    .on_press(Message::RemoveBookmark(idx));

                let row_item = row![play_btn, delete_btn,]
                    .spacing(6)
                    .align_y(Alignment::Center);

                list = list.push(row_item);
            }

            let scrollable_content = container(list)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 0.0,
                    left: 2.0,
                })
                .width(Length::Fill);

            scrollable(scrollable_content)
                .height(Length::Fixed(480.0))
                .width(Length::Fill)
                .into()
        };

        let card = container(
            column![header_row, content_element,]
                .spacing(16)
                .width(Length::Fixed(640.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseBookmarksModal)
    }
}
