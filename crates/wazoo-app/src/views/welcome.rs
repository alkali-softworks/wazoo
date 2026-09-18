/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Welcome & Empty State View
 *
 * Presents an introductory splash card when no media is loaded, guiding users to configure
 * their media folders and initiate library indexing.
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, text},
};

impl WazooApp {
    pub(crate) fn view_welcome(&self) -> Element<'_, Message> {
        let card = container(
            column![
                text("📁").size(48),
                text(self.t("wazoo.welcome"))
                    .size(24)
                    .color(iced::Color::WHITE),
                text(self.t("wazoo.no_folders_msg"))
                    .size(14)
                    .color(theme::COLOR_TEXT_MUTED),
                button(text(self.t("wazoo.open_settings_to_add")))
                    .style(theme::action_button_style)
                    .on_press(Message::OpenSettingsModal)
                    .padding([10, 20]),
            ]
            .spacing(16)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .style(theme::welcome_card_style);

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub(crate) fn view_no_matches(&self) -> Element<'_, Message> {
        let total_in_db = self.db.get_video_count().unwrap_or(0);
        let mut content = column![
            text("🔍").size(48),
            text(self.t("wazoo.no_videos_found"))
                .size(22)
                .color(iced::Color::WHITE),
            text(self.t("wazoo.query_no_match_desc"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
        ]
        .spacing(14)
        .align_x(Alignment::Center);

        if !self.active_search_query.trim().is_empty() {
            content = content.push(
                container(
                    text(format!("\"{}\"", self.active_search_query.trim()))
                        .size(13)
                        .color(theme::COLOR_PRIMARY),
                )
                .padding([4, 12])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.2, 0.2, 0.25, 0.6,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        let mut buttons_row = row![
            button(text(self.t("common.search")).size(13))
                .style(theme::action_button_style)
                .on_press(Message::OpenSearchModal)
                .padding([8, 16]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        if total_in_db > 0 {
            buttons_row = buttons_row.push(
                button(text(self.t("search.clear_search")).size(13))
                    .style(theme::menu_item_style)
                    .on_press(Message::ClearSearch)
                    .padding([8, 16]),
            );
        } else {
            buttons_row = buttons_row.push(
                button(text(self.t("wazoo.open_settings_to_add")).size(13))
                    .style(theme::menu_item_style)
                    .on_press(Message::OpenSettingsModal)
                    .padding([8, 16]),
            );
        }

        content = content.push(Space::new().height(Length::Fixed(6.0)));
        content = content.push(buttons_row);

        let card = container(content)
            .padding(40)
            .style(theme::welcome_card_style);

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
