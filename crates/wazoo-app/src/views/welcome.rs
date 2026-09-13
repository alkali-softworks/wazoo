/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Welcome & Empty State View
 * 
 * Presents an introductory splash card when no media is loaded, guiding users to configure
 * their media folders and initiate library indexing.
 */

use iced::{
    widget::{button, column, container, text},
    Alignment, Element, Length,
};
use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_welcome(&self) -> Element<'_, Message> {
        let card = container(
            column![
                text("📁").size(48),
                text(self.t("wazoo.welcome")).size(24).color(iced::Color::WHITE),
                text(self.t("wazoo.no_folders_msg")).size(14).color(theme::COLOR_TEXT_MUTED),
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
}
