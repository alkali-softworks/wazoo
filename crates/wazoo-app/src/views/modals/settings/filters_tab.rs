/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Filters Tab Settings Component
 *
 * Dedicated tab for visual GPU post-processing shader filters:
 * - Master filter toggle (bound to the '7' key)
 * - CRT Scanline, Glow & Curvature filter
 * - Wavy Fluid UV Displacement filter
 * - Volumetric Fog FBM noise filter
 */

use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, text},
};

pub fn view_filters_tab(app: &WazooApp) -> Element<'_, Message> {
    let master_active = app.settings.filters_enabled;
    let shortcut_hint = app.settings.keybinds.toggle_filters.to_uppercase();

    // 1. Master Filters Switch Card
    let master_card = cursor::PointerCursor::new(
        button(
            row![
                text(if master_active { "✨" } else { "🔌" }).size(20),
                column![
                    row![
                        text(app.t("settings.master_filters"))
                            .size(14)
                            .font(theme::FONT_BOLD)
                            .color(if master_active {
                                theme::COLOR_PRIMARY
                            } else {
                                iced::Color::WHITE
                            }),
                        Space::new().width(Length::Fixed(6.0)),
                        container(
                            text(format!("[{}]", shortcut_hint))
                                .size(10)
                                .font(theme::FONT_BOLD)
                                .color(if master_active {
                                    theme::COLOR_PRIMARY
                                } else {
                                    theme::COLOR_TEXT_DIM
                                })
                        )
                        .padding([1, 5])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                width: 1.0,
                                color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                            },
                            ..Default::default()
                        }),
                    ]
                    .align_y(Alignment::Center),
                    text(app.t("settings.master_filters_desc"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                view_status_pill(master_active, true),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if master_active {
                match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.25),
                    button::Status::Pressed => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.35),
                    _ => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.14),
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
                    color: if master_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::ToggleFilters)
        .padding([12, 14])
        .width(Length::Fill),
    );

    // 2. Individual Filter Cards
    let crt_active = app.settings.filter_crt;
    let crt_card = build_filter_card(
        if crt_active { "📺" } else { "🖥" },
        app.t("settings.crt_filter"),
        app.t("settings.crt_filter_desc"),
        crt_active,
        master_active,
        Message::ToggleCrtFilter,
    );

    let wavy_active = app.settings.filter_wavy;
    let wavy_card = build_filter_card(
        "🌊",
        app.t("settings.wavy_filter"),
        app.t("settings.wavy_filter_desc"),
        wavy_active,
        master_active,
        Message::ToggleWavyFilter,
    );

    let fog_active = app.settings.filter_fog;
    let fog_card = build_filter_card(
        "🌫️",
        app.t("settings.fog_filter"),
        app.t("settings.fog_filter_desc"),
        fog_active,
        master_active,
        Message::ToggleFogFilter,
    );

    let effects_list = column![crt_card, wavy_card, fog_card].spacing(8);

    column![master_card, effects_list].spacing(16).into()
}

/// Renders a toggleable filter card with icon, title, description, and status pill.
fn build_filter_card<'a>(
    icon: &'static str,
    title: String,
    desc: String,
    is_active: bool,
    master_active: bool,
    toggle_msg: Message,
) -> Element<'a, Message> {
    cursor::PointerCursor::new(
        button(
            row![
                text(icon).size(18),
                column![
                    text(title)
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if is_active && master_active {
                            theme::COLOR_PRIMARY
                        } else if is_active {
                            iced::Color::from_rgb(0.7, 0.85, 1.0)
                        } else {
                            iced::Color::WHITE
                        }),
                    text(desc).size(11).color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                view_status_pill(is_active, master_active),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if is_active && master_active {
                match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.22),
                    button::Status::Pressed => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.3),
                    _ => iced::Color::from_rgba(0.26, 0.72, 0.51, 0.10),
                }
            } else if is_active {
                match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.25),
                    button::Status::Pressed => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.35),
                    _ => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.12),
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
                    color: if is_active && master_active {
                        theme::COLOR_PRIMARY
                    } else if is_active {
                        iced::Color::from_rgb(0.3, 0.45, 0.6)
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(toggle_msg)
        .padding([10, 14])
        .width(Length::Fill),
    )
    .into()
}

/// Renders a neat ON/OFF pill badge.
fn view_status_pill<'a>(is_active: bool, master_active: bool) -> Element<'a, Message> {
    let pill_text = if is_active { "ON" } else { "OFF" };
    let text_color = if is_active && master_active {
        theme::COLOR_PRIMARY
    } else if is_active {
        iced::Color::from_rgb(0.7, 0.85, 1.0)
    } else {
        theme::COLOR_TEXT_DIM
    };

    let border_color = if is_active && master_active {
        theme::COLOR_PRIMARY_BORDER
    } else if is_active {
        iced::Color::from_rgba(0.4, 0.6, 0.9, 0.5)
    } else {
        theme::COLOR_BORDER
    };

    container(
        text(pill_text)
            .size(11)
            .font(if is_active {
                theme::FONT_BOLD
            } else {
                Default::default()
            })
            .color(text_color),
    )
    .padding([3, 10])
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
        ..Default::default()
    })
    .into()
}
