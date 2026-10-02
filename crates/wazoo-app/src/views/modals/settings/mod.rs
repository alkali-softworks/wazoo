/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal Subsystem
 *
 * Modular settings tabs: General, Playback, and System.
 */

pub mod badge;
pub mod cube_tab;
pub mod general_tab;
pub mod playback_tab;
pub mod system_tab;

use crate::app::{SettingsTab, WazooApp};
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text},
};

impl WazooApp {
    pub fn view_settings_modal(&self) -> Element<'_, Message> {
        let is_general = self.modals.settings_tab == SettingsTab::General;
        let is_playback = self.modals.settings_tab == SettingsTab::Playback;
        let is_system = self.modals.settings_tab == SettingsTab::System;
        let is_cube = self.modals.settings_tab == SettingsTab::Cube;

        let tabs_bar = row![
            button(
                row![
                    text("⚙").size(14),
                    text(self.t("settings.tab_general")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_general))
            .on_press(Message::SetSettingsTab(SettingsTab::General))
            .padding([7, 16]),
            button(
                row![
                    text("▶").size(12),
                    text(self.t("settings.tab_playback")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_playback))
            .on_press(Message::SetSettingsTab(SettingsTab::Playback))
            .padding([7, 16]),
            button(
                row![
                    text("💻").size(14),
                    text(self.t("settings.tab_system")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_system))
            .on_press(Message::SetSettingsTab(SettingsTab::System))
            .padding([7, 16]),
            button(
                row![
                    text("🧊").size(14),
                    text(self.t("settings.tab_cube")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_cube))
            .on_press(Message::SetSettingsTab(SettingsTab::Cube))
            .padding([7, 16]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let header = row![
            text(self.t("settings.title"))
                .size(20)
                .color(iced::Color::WHITE),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseSettingsModal),
        ]
        .align_y(Alignment::Center);

        let active_tab_content: Element<'_, Message> = match self.modals.settings_tab {
            SettingsTab::General => general_tab::view_general_tab(self),
            SettingsTab::Playback => playback_tab::view_playback_tab(self),
            SettingsTab::System => system_tab::view_system_tab(self),
            SettingsTab::Cube => cube_tab::view_cube_tab(self),
        };

        let scrollable_content = container(active_tab_content)
            .padding(iced::Padding {
                top: 2.0,
                right: 14.0,
                bottom: 8.0,
                left: 2.0,
            })
            .width(Length::Fill);

        let card = container(
            column![
                header,
                tabs_bar,
                scrollable(scrollable_content)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::default(),
                    ))
                    .height(Length::Fixed(480.0))
                    .width(Length::Fill),
            ]
            .spacing(18)
            .width(Length::Fixed(640.0)),
        )
        .padding(24)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSettingsModal)
    }
}
