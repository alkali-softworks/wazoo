/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal Subsystem
 *
 * Modular settings tabs: General, Playback, Filters, CRT, 3D Cube, Plex, Theme, and System.
 */

pub mod badge;
pub mod crt_tab;
pub mod cube_tab;
pub mod filters_tab;
pub mod general_tab;
pub mod playback_tab;
pub mod plex_tab;
pub mod system_tab;
pub mod theme_tab;

use crate::app::{SettingsTab, WazooApp};
use crate::message::Message;
use crate::theme;
use crate::wrap::Wrap;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text},
};

impl WazooApp {
    pub fn view_settings_modal(&self) -> Element<'_, Message> {
        let pal = self.theme_palette;
        let tabs = [
            (SettingsTab::General, self.t("settings.tab_general")),
            (SettingsTab::Playback, self.t("settings.tab_playback")),
            (SettingsTab::Filters, self.t("settings.tab_filters")),
            (SettingsTab::Crt, self.t("settings.tab_crt")),
            (SettingsTab::Cube, self.t("settings.tab_cube")),
            (SettingsTab::Plex, self.t("settings.tab_plex")),
            (SettingsTab::Theme, self.t("settings.tab_theme")),
            (SettingsTab::System, self.t("settings.tab_system")),
        ];

        let tab_buttons: Vec<Element<'_, Message>> = tabs
            .into_iter()
            .map(|(tab, label)| {
                let is_active = self.modals.settings_tab == tab;
                button(text(label).size(13))
                    .style(theme::settings_tab_button_style(pal, is_active))
                    .on_press(Message::SetSettingsTab(tab))
                    .padding([7, 11])
                    .into()
            })
            .collect();

        let tabs_bar = Wrap::with_elements(tab_buttons)
            .spacing(5)
            .vertical_spacing(6)
            .align_items(Alignment::Center);

        let header = row![
            text(self.t("settings.title")).size(20).color(pal.text),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style(pal))
                .on_press(Message::CloseSettingsModal),
        ]
        .align_y(Alignment::Center);

        let active_tab_content: Element<'_, Message> = match self.modals.settings_tab {
            SettingsTab::General => general_tab::view_general_tab(self),
            SettingsTab::Playback => playback_tab::view_playback_tab(self),
            SettingsTab::Filters => filters_tab::view_filters_tab(self),
            SettingsTab::Crt => crt_tab::view_crt_tab(self),
            SettingsTab::Cube => cube_tab::view_cube_tab(self),
            SettingsTab::Plex => plex_tab::view_plex_tab(self),
            SettingsTab::Theme => theme_tab::view_theme_tab(self),
            SettingsTab::System => system_tab::view_system_tab(self),
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
        .style(theme::modal_card_style(pal));

        Self::wrap_modal_with_backdrop(card, Message::CloseSettingsModal)
    }
}
