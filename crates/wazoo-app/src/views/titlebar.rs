/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Titlebar & Quick Menu
 *
 * Renders the sliding window titlebar with application branding, interactive drag handle,
 * window control buttons (minimize, maximize, close), and top-left dropdown menu.
 */

use crate::app::WazooApp;
use crate::assets::{SVG_WINDOW_CLOSE, SVG_WINDOW_MAXIMIZE, SVG_WINDOW_MINIMIZE};
use crate::cursor;
use crate::message::Message;
use crate::slide::SlideDown;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme, mouse,
    widget::{Space, button, column, container, mouse_area, row, svg, text},
};

impl WazooApp {
    pub fn view_titlebar(&self) -> Element<'_, Message> {
        let alpha = self.titlebar_alpha();

        let badge_btn = button(
            row![
                iced::widget::image(self.app_icon_handle.clone())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(alpha),
                text("Wazoo")
                    .size(16)
                    .color(theme::with_alpha(iced::Color::WHITE, alpha)),
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

        let alkali_btn = button(text("ALKALI").size(11).font(theme::FONT_BOLD))
            .style(theme::titlebar_brand_link_style_with_alpha(alpha))
            .on_press(Message::OpenAlkaliWebsite)
            .padding(iced::Padding {
                top: 6.0,
                right: 8.0,
                bottom: 2.0,
                left: 8.0,
            });

        let drag_strip = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .interaction(mouse::Interaction::Idle)
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
                Space::new().width(Length::Fixed(6.0)),
                alkali_btn,
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
            background: Some(iced::Background::Color(theme::with_alpha(
                theme::COLOR_TITLEBAR_BG,
                alpha,
            ))),
            ..Default::default()
        });

        let slide_progress = self.titlebar_slide_progress();

        if self.titlebar.show_dropdown_menu {
            let dp = self.dropdown_menu_slide_progress();
            let menu_dropdown = container(
                self.view_app_menu_list([8, 14])
                    .spacing(2)
                    .width(Length::Fixed(240.0)),
            )
            .padding(4)
            .style(theme::menu_dropdown_style);

            let animated_dropdown = SlideDown::new(menu_dropdown, dp, true);
            let titlebar_content = column![titlebar_row, animated_dropdown];
            SlideDown::new(titlebar_content, slide_progress, false).into()
        } else {
            SlideDown::new(titlebar_row, slide_progress, false).into()
        }
    }

    fn menu_item(
        label: String,
        hint: Option<String>,
        msg: Message,
        padding: [u16; 2],
    ) -> Element<'static, Message> {
        let mut row_children: Vec<Element<'static, Message>> = vec![
            text(label).size(13).color(iced::Color::WHITE).into(),
            Space::new().width(Length::Fill).into(),
        ];
        if let Some(h) = hint {
            row_children.push(text(h).size(11).color(theme::COLOR_TEXT_MUTED).into());
        }
        cursor::PointerCursor::new(
            button(
                iced::widget::Row::with_children(row_children)
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
            )
            .style(theme::menu_item_style)
            .on_press(msg)
            .padding(padding)
            .width(Length::Fill),
        )
        .into()
    }

    pub(crate) fn view_app_menu_list(
        &self,
        padding: [u16; 2],
    ) -> iced::widget::Column<'_, Message> {
        let kb = &self.settings.keybinds;
        column![
            Self::menu_item(
                self.t("common.search"),
                Some(kb.menu_hint(&kb.search_videos)),
                Message::OpenSearchModal,
                padding,
            ),
            Self::menu_item(
                self.t("common.add_player"),
                Some(kb.menu_hint(&kb.add_player)),
                Message::AddNewPlayer,
                padding,
            ),
            Self::menu_item(
                self.t("common.toggle_layout"),
                Some(kb.menu_hint(&kb.toggle_layout)),
                Message::CycleLayout,
                padding,
            ),
            Self::menu_item(
                self.t("common.toggle_files"),
                Some(kb.menu_hint(&kb.toggle_file_picker)),
                Message::ToggleFilePicker,
                padding,
            ),
            Self::menu_item(
                self.t("transcript.title"),
                Some(kb.menu_hint(&kb.toggle_transcript)),
                Message::ToggleTranscript,
                padding,
            ),
            Self::menu_item(
                self.t("bookmarks.title"),
                Some(kb.menu_hint(&kb.toggle_bookmarks)),
                Message::ToggleBookmarksModal,
                padding,
            ),
            Self::menu_item(
                self.t("history.title"),
                Some(kb.menu_hint(&kb.toggle_history)),
                Message::ToggleHistoryDrawer,
                padding,
            ),
            Self::menu_item(
                if self.settings.is_always_on_top {
                    self.t("common.unpin_window")
                } else {
                    self.t("common.pin_window")
                },
                Some(kb.menu_hint(&kb.toggle_pin)),
                Message::CyclePinMode,
                padding,
            ),
            Self::menu_item(
                self.t("common.fit_window"),
                Some(kb.menu_hint(&kb.fit_window)),
                Message::FitWindow,
                padding,
            ),
            Self::menu_item(
                self.t("common.settings"),
                Some(kb.menu_hint(&kb.open_settings)),
                Message::OpenSettingsModal,
                padding,
            ),
            Self::menu_item(
                self.t("common.help"),
                Some(kb.menu_hint(&kb.open_help)),
                Message::OpenHelpModal,
                padding,
            ),
            Self::menu_item(
                self.t("common.quit"),
                Some(kb.menu_hint(&kb.close_app)),
                Message::CloseApp,
                padding,
            ),
        ]
    }
}
