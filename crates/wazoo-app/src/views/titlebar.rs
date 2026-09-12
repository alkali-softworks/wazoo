/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Window Titlebar & Quick Menu
 * 
 * Renders the sliding window titlebar with application branding, interactive drag handle,
 * window control buttons (minimize, maximize, close), and top-left dropdown menu.
 */

use iced::{
    widget::{button, column, container, mouse_area, row, svg, text, Space},
    Alignment, Element, Length, Theme,
};
use wazoo_core::PlaybackMode;
use crate::app::WazooApp;
use crate::assets::{SVG_WINDOW_CLOSE, SVG_WINDOW_MAXIMIZE, SVG_WINDOW_MINIMIZE};
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_titlebar(&self) -> Element<'_, Message> {
        let badge_btn = button(
            row![
                iced::widget::image(self.app_icon_handle.clone())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
                text("Wazoo").size(16).color(iced::Color::WHITE),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(theme::titlebar_badge_style)
        .on_press(Message::ToggleDropdownMenu)
        .padding(iced::Padding {
            top: 2.0,
            right: 14.0,
            bottom: 2.0,
            left: 8.0,
        });

        let drag_strip = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .on_press(Message::TitleBarPressed)
        .on_double_click(Message::MaximizeWindow);

        let window_buttons = row![
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_MINIMIZE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style)
            .on_press(Message::MinimizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_MAXIMIZE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style)
            .on_press(Message::MaximizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_CLOSE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::close_window_button_style)
            .on_press(Message::CloseApp)
            .padding(0),
        ]
        .width(Length::Shrink);

        let titlebar_row = container(
            row![
                badge_btn,
                drag_strip,
                window_buttons,
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(30.0)),
        )
        .width(Length::Fill)
        .height(Length::Fixed(30.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme::COLOR_TITLEBAR_BG)),
            ..Default::default()
        });

        if self.show_dropdown_menu {
            let menu_dropdown = container(
                column![
                    button(text(format!("{} (N)", self.t("common.add_player")))).style(theme::menu_item_style).on_press(Message::AddNewPlayer).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (L)", self.t("common.toggle_layout")))).style(theme::menu_item_style).on_press(Message::CycleLayout).padding([8, 14]).width(Length::Fill),
                    button(text(if self.settings.playback_mode == PlaybackMode::Scroll { "Disable Infinity Stream (5)" } else { "Infinity Stream (5)" })).style(theme::menu_item_style).on_press(Message::ToggleScrollMode).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (H)", self.t("common.toggle_files")))).style(theme::menu_item_style).on_press(Message::ToggleFilePicker).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (V)", self.t("transcript.title")))).style(theme::menu_item_style).on_press(Message::ToggleTranscript).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (B)", self.t("bookmarks.title")))).style(theme::menu_item_style).on_press(Message::ToggleBookmarksModal).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (J)", self.t("common.search")))).style(theme::menu_item_style).on_press(Message::OpenSearchModal).padding([8, 14]).width(Length::Fill),
                    button(text(self.t("common.settings"))).style(theme::menu_item_style).on_press(Message::OpenSettingsModal).padding([8, 14]).width(Length::Fill),
                    button(text(self.t("common.help"))).style(theme::menu_item_style).on_press(Message::OpenHelpModal).padding([8, 14]).width(Length::Fill),
                    button(text(format!("{} (Alt+X)", self.t("common.quit")))).style(theme::menu_item_style).on_press(Message::CloseApp).padding([8, 14]).width(Length::Fill),
                ]
                .width(Length::Fixed(230.0)),
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                border: iced::Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_right: 6.0,
                        bottom_left: 6.0,
                    },
                    width: 1.0,
                    color: theme::COLOR_BORDER,
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            });

            column![titlebar_row, menu_dropdown].into()
        } else {
            titlebar_row.into()
        }
    }
}
