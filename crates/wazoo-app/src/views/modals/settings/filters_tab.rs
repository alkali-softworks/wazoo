/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Filters Tab Settings Component
 *
 * Dedicated tab for visual post-processing filters (CRT filter, etc.).
 */

use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme;
use crate::views::modals::settings::badge::badge_with_reset;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, row, text},
};

pub fn view_filters_tab(app: &WazooApp) -> Element<'_, Message> {
    let crt_active = app.settings.crt_enabled;
    let crt_toggle_btn = cursor::PointerCursor::new(
        button(
            row![
                text(if crt_active { "📺" } else { "🖥" }).size(18),
                column![
                    text(app.t("settings.crt_filter"))
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if crt_active {
                            theme::COLOR_PRIMARY
                        } else {
                            iced::Color::WHITE
                        }),
                    text(app.t("settings.crt_filter_desc"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    if crt_active {
                        "ON".to_string()
                    } else {
                        "OFF".to_string()
                    },
                    crt_active,
                    Message::ToggleCrtFilter,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if crt_active {
                match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.22),
                    button::Status::Pressed => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.3),
                    _ => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.12),
                }
            } else {
                match status {
                    button::Status::Hovered => theme::COLOR_BTN_HOVER,
                    button::Status::Pressed => iced::Color::from_rgb(0.15, 0.15, 0.15),
                    _ => theme::COLOR_BTN_BG,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if crt_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::ToggleCrtFilter)
        .padding([10, 14])
        .width(Length::Fill),
    );

    column![crt_toggle_btn].spacing(16).into()
}
