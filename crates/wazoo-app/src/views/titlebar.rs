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
use crate::app::WazooApp;
use crate::assets::{SVG_WINDOW_CLOSE, SVG_WINDOW_MAXIMIZE, SVG_WINDOW_MINIMIZE};
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_titlebar(&self) -> Element<'_, Message> {
        let alpha = self.titlebar_alpha();

        let badge_btn = button(
            row![
                iced::widget::image(self.app_icon_handle.clone())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(alpha),
                text("Wazoo").size(16).color(theme::with_alpha(iced::Color::WHITE, alpha)),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(theme::titlebar_badge_style_with_alpha(alpha))
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
                        .opacity(alpha)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style_with_alpha(alpha))
            .on_press(Message::MinimizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_MAXIMIZE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                        .opacity(alpha)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style_with_alpha(alpha))
            .on_press(Message::MaximizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_CLOSE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                        .opacity(alpha)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::close_window_button_style_with_alpha(alpha))
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
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme::with_alpha(theme::COLOR_TITLEBAR_BG, alpha))),
            ..Default::default()
        });

        if self.show_dropdown_menu {
            let menu_dropdown = container(
                self.view_app_menu_list([8, 14])
                    .spacing(2)
                    .width(Length::Fixed(230.0)),
            )
            .padding(4)
            .style(theme::menu_dropdown_style);

            column![titlebar_row, menu_dropdown].into()
        } else {
            titlebar_row.into()
        }
    }

    pub(crate) fn view_app_menu_list(&self, padding: [u16; 2]) -> iced::widget::Column<'_, Message> {
        let kb = &self.settings.keybinds;
        column![
            button(text(format!("{} ({})", self.t("common.search"), kb.menu_hint(&kb.search_videos))))
                .style(theme::menu_item_style)
                .on_press(Message::OpenSearchModal)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("common.add_player"), kb.menu_hint(&kb.add_player))))
                .style(theme::menu_item_style)
                .on_press(Message::AddNewPlayer)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("common.toggle_layout"), kb.menu_hint(&kb.toggle_layout))))
                .style(theme::menu_item_style)
                .on_press(Message::CycleLayout)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("common.toggle_files"), kb.menu_hint(&kb.toggle_file_picker))))
                .style(theme::menu_item_style)
                .on_press(Message::ToggleFilePicker)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("transcript.title"), kb.menu_hint(&kb.toggle_transcript))))
                .style(theme::menu_item_style)
                .on_press(Message::ToggleTranscript)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("bookmarks.title"), kb.menu_hint(&kb.toggle_bookmarks))))
                .style(theme::menu_item_style)
                .on_press(Message::ToggleBookmarksModal)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("history.title"), kb.menu_hint(&kb.toggle_history))))
                .style(theme::menu_item_style)
                .on_press(Message::ToggleHistoryDrawer)
                .padding(padding)
                .width(Length::Fill),
            button(text(self.t("common.settings")))
                .style(theme::menu_item_style)
                .on_press(Message::OpenSettingsModal)
                .padding(padding)
                .width(Length::Fill),
            button(text(self.t("common.help")))
                .style(theme::menu_item_style)
                .on_press(Message::OpenHelpModal)
                .padding(padding)
                .width(Length::Fill),
            button(text(format!("{} ({})", self.t("common.quit"), kb.menu_hint(&kb.close_app))))
                .style(theme::menu_item_style)
                .on_press(Message::CloseApp)
                .padding(padding)
                .width(Length::Fill),
        ]
    }
}
