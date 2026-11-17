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
        let pal = self.theme_palette;
        let card = container(
            column![
                text("📁").size(48),
                text(self.t("wazoo.welcome"))
                    .size(24)
                    .color(pal.text),
                text(self.t("wazoo.no_folders_msg"))
                    .size(14)
                    .color(pal.text_muted),
                button(text(self.t("wazoo.open_settings_to_add")))
                    .style(theme::action_button_style(pal))
                    .on_press(Message::OpenSettingsModal)
                    .padding([10, 20]),
            ]
            .spacing(16)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .style(theme::welcome_card_style(pal));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub(crate) fn view_no_matches(&self) -> Element<'_, Message> {
        let pal = self.theme_palette;
        let total_in_db = self.db.get_video_count().unwrap_or(0);
        let mut content = column![
            text("🔍").size(48),
            text(self.t("wazoo.no_videos_found"))
                .size(22)
                .color(pal.text),
        ]
        .spacing(14)
        .align_x(Alignment::Center);

        if !self.search.active_query.trim().is_empty() {
            content = content.push(
                container(
                    text(format!("\"{}\"", self.search.active_query.trim()))
                        .size(13)
                        .color(pal.primary),
                )
                .padding([4, 12])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme::with_alpha(
                        pal.card_bg,
                        0.75,
                    ))),
                    border: iced::Border {
                        radius: pal.radius(4.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        let mut buttons_row = row![
            button(text(self.t("common.search")).size(13))
                .style(theme::action_button_style(pal))
                .on_press(Message::OpenSearchModal)
                .padding([8, 16]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        if total_in_db > 0 {
            buttons_row = buttons_row.push(
                button(text(self.t("search.clear_search")).size(13))
                    .style(theme::menu_item_style(pal))
                    .on_press(Message::ClearSearch)
                    .padding([8, 16]),
            );
        } else {
            buttons_row = buttons_row.push(
                button(text(self.t("wazoo.open_settings_to_add")).size(13))
                    .style(theme::menu_item_style(pal))
                    .on_press(Message::OpenSettingsModal)
                    .padding([8, 16]),
            );
        }

        content = content.push(Space::new().height(Length::Fixed(6.0)));
        content = content.push(buttons_row);

        let card = container(content)
            .padding(40)
            .style(theme::welcome_card_style(pal));

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
