/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal Badge with Reset Button
 */

use crate::cursor;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, container, row, text},
};

pub fn badge_with_reset(
    val_text: String,
    is_active: bool,
    reset_msg: Message,
) -> Element<'static, Message> {
    let badge_box = container(
        text(val_text)
            .size(12)
            .font(if is_active {
                theme::FONT_BOLD
            } else {
                Default::default()
            })
            .color(if is_active {
                theme::COLOR_PRIMARY
            } else {
                theme::COLOR_TEXT_DIM
            }),
    )
    .padding([2, 8])
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: if is_active {
                theme::COLOR_PRIMARY_BORDER
            } else {
                theme::COLOR_BORDER
            },
        },
        ..Default::default()
    });

    if is_active {
        let reset_btn = cursor::PointerCursor::new(
            button(text("↺").size(11))
                .style(theme::settings_reset_button_style)
                .on_press(reset_msg)
                .padding([2, 5]),
        );
        row![badge_box, reset_btn]
            .spacing(5)
            .align_y(Alignment::Center)
            .into()
    } else {
        row![badge_box, Space::new().width(Length::Fixed(20.0))]
            .spacing(5)
            .align_y(Alignment::Center)
            .into()
    }
}
