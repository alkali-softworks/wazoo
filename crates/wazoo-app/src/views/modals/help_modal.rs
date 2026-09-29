/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Keyboard Shortcut Help Modal Component
 *
 * Categorized keyboard shortcuts display with visual keycaps.
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, scrollable, text},
};
use wazoo_core::{HelpCategory, KeyDisplay};

impl WazooApp {
    pub fn view_help_modal(&self) -> Element<'_, Message> {
        let or_text = self.t("common.or");
        let categories = self.settings.keybinds.help_categories(|k| self.t(k));

        let mut left_col = column![].spacing(12).width(Length::FillPortion(1));
        let mut right_col = column![].spacing(12).width(Length::FillPortion(1));

        // Distribute categories systematically:
        // Left Column: Playback & Navigation, Grid & Multi-Player
        // Right Column: Panels & Drawers, Audio & Display, System & Window
        for (idx, cat) in categories.iter().enumerate() {
            if idx == 0 || idx == 2 {
                left_col = left_col.push(render_category_card(cat, &or_text));
            } else {
                right_col = right_col.push(render_category_card(cat, &or_text));
            }
        }

        let columns_row = row![left_col, right_col].spacing(12);

        let scrollable_content = container(columns_row)
            .padding(iced::Padding {
                top: 0.0,
                right: 16.0,
                bottom: 8.0,
                left: 2.0,
            })
            .width(Length::Fill);

        let header_row = row![
            row![
                container(text("⌨️").size(18))
                    .padding([4, 6])
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.259, 0.722, 0.514, 0.15,
                        ))),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 1.0,
                            color: iced::Color::from_rgba(0.259, 0.722, 0.514, 0.4),
                        },
                        ..Default::default()
                    }),
                column![
                    text(self.t("help.title"))
                        .size(17)
                        .font(theme::FONT_BOLD)
                        .color(iced::Color::WHITE),
                    text(self.t("help.subtitle"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseHelpModal),
        ]
        .align_y(Alignment::Center);

        let card = container(
            column![
                header_row,
                scrollable(scrollable_content)
                    .height(Length::Fixed(500.0))
                    .width(Length::Fill),
            ]
            .spacing(14)
            .width(Length::Fixed(680.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseHelpModal)
    }
}

fn view_keycap(key_str: &str) -> Element<'static, Message> {
    container(
        text(key_str.to_string())
            .size(11)
            .font(theme::FONT_BOLD)
            .color(iced::Color::from_rgb(0.95, 0.95, 0.95)),
    )
    .padding([3, 7])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.18, 0.18, 0.20,
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: iced::Color::from_rgb(0.32, 0.32, 0.36),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: iced::Vector::new(0.0, 1.5),
            blur_radius: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn render_key_display(display: &KeyDisplay, or_text: &str) -> Element<'static, Message> {
    match display {
        KeyDisplay::Single(k) => view_keycap(k),
        KeyDisplay::Alternatives(keys) => {
            let mut r = row![].spacing(5).align_y(Alignment::Center);
            for (idx, k) in keys.iter().enumerate() {
                if idx > 0 {
                    r = r.push(
                        text(or_text.to_string())
                            .size(10)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                }
                r = r.push(view_keycap(k));
            }
            r.into()
        }
        KeyDisplay::Pair(k1, k2) => row![view_keycap(k1), view_keycap(k2)]
            .spacing(4)
            .align_y(Alignment::Center)
            .into(),
        KeyDisplay::Combo(keys) => {
            let mut r = row![].spacing(4).align_y(Alignment::Center);
            for (idx, k) in keys.iter().enumerate() {
                if idx > 0 {
                    r = r.push(text("+").size(10).color(theme::COLOR_TEXT_MUTED));
                }
                r = r.push(view_keycap(k));
            }
            r.into()
        }
    }
}

fn render_category_card(cat: &HelpCategory, or_text: &str) -> Element<'static, Message> {
    let header = row![
        text(cat.icon).size(13),
        text(cat.title.clone())
            .size(12)
            .font(theme::FONT_BOLD)
            .color(theme::COLOR_PRIMARY),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut items_col = column![].spacing(6);
    for item in &cat.shortcuts {
        let key_el = render_key_display(&item.key, or_text);
        let key_container = container(key_el)
            .width(Length::Fixed(110.0))
            .align_x(iced::alignment::Horizontal::Left);

        let desc_text = text(item.description.clone())
            .size(12)
            .color(theme::COLOR_TEXT_DIM);

        items_col = items_col.push(
            row![
                key_container,
                Space::new().width(Length::Fixed(6.0)),
                desc_text
            ]
            .align_y(Alignment::Center),
        );
    }

    container(column![header, items_col].spacing(8))
        .padding([11, 14])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.14, 0.14, 0.15, 0.7,
            ))),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: iced::Color::from_rgb(0.24, 0.24, 0.26),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
