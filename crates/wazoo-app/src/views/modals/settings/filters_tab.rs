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
use crate::theme::{self, ThemePalette};
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, text},
};

pub fn view_filters_tab(app: &WazooApp) -> Element<'_, Message> {
    let pal = app.theme_palette;
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
                            .color(if master_active { pal.primary } else { pal.text }),
                        Space::new().width(Length::Fixed(6.0)),
                        container(
                            text(format!("[{}]", shortcut_hint))
                                .size(10)
                                .font(theme::FONT_BOLD)
                                .color(if master_active {
                                    pal.primary
                                } else {
                                    pal.text_dim
                                })
                        )
                        .padding([1, 5])
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.0, 0.0, 0.0, 0.3
                            ))),
                            border: iced::Border {
                                radius: pal.radius(4.0).into(),
                                width: 1.0,
                                color: theme::with_alpha(pal.text, 0.12),
                            },
                            ..Default::default()
                        }),
                    ]
                    .align_y(Alignment::Center),
                    text(app.t("settings.master_filters_desc"))
                        .size(11)
                        .color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                view_status_pill(pal, master_active, true),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_toggle_card_style(pal, master_active))
        .on_press(Message::ToggleFilters)
        .padding([12, 14])
        .width(Length::Fill),
    );

    // 2. Individual Filter Cards
    let crt_active = app.settings.filter_crt;
    let crt_card = build_filter_card(
        pal,
        if crt_active { "📺" } else { "🖥" },
        app.t("settings.crt_filter"),
        app.t("settings.crt_filter_desc"),
        crt_active,
        master_active,
        Message::ToggleCrtFilter,
    );

    let wavy_active = app.settings.filter_wavy;
    let wavy_card = build_filter_card(
        pal,
        "🌊",
        app.t("settings.wavy_filter"),
        app.t("settings.wavy_filter_desc"),
        wavy_active,
        master_active,
        Message::ToggleWavyFilter,
    );

    let fog_active = app.settings.filter_fog;
    let fog_card = build_filter_card(
        pal,
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
    pal: ThemePalette,
    icon: &'static str,
    title: String,
    desc: String,
    is_active: bool,
    master_active: bool,
    toggle_msg: Message,
) -> Element<'a, Message> {
    let active_card_bg = theme::mix_color(pal.btn_bg, pal.primary, 0.12);
    let active_title_color = theme::readable_accent_on(pal, active_card_bg);
    let standby_title_color = theme::readable_accent_with_fallback(
        iced::Color::from_rgb(0.7, 0.85, 1.0),
        pal.text,
        pal.card_bg,
    );
    cursor::PointerCursor::new(
        button(
            row![
                text(icon).size(18),
                column![
                    text(title)
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if is_active && master_active {
                            active_title_color
                        } else if is_active {
                            standby_title_color
                        } else {
                            pal.text
                        }),
                    text(desc).size(11).color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                view_status_pill(pal, is_active, master_active),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if is_active && master_active {
                match status {
                    button::Status::Hovered => theme::mix_color(pal.btn_bg, pal.primary, 0.22),
                    button::Status::Pressed => theme::mix_color(pal.btn_bg, pal.primary, 0.30),
                    _ => active_card_bg,
                }
            } else if is_active {
                match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.25),
                    button::Status::Pressed => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.35),
                    _ => iced::Color::from_rgba(0.2, 0.35, 0.5, 0.12),
                }
            } else {
                match status {
                    button::Status::Hovered => pal.btn_hover,
                    button::Status::Pressed => pal.btn_pressed,
                    _ => pal.btn_bg,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: pal.text,
                border: iced::Border {
                    radius: pal.radius(8.0).into(),
                    width: 1.0,
                    color: if is_active && master_active {
                        pal.primary
                    } else if is_active {
                        iced::Color::from_rgb(0.3, 0.45, 0.6)
                    } else {
                        pal.border
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
fn view_status_pill<'a>(
    pal: ThemePalette,
    is_active: bool,
    master_active: bool,
) -> Element<'a, Message> {
    let pill_text = if is_active { "ON" } else { "OFF" };
    let text_color = if is_active && master_active {
        theme::readable_accent_on(pal, pal.card_bg)
    } else if is_active {
        theme::readable_accent_with_fallback(
            iced::Color::from_rgb(0.7, 0.85, 1.0),
            pal.text,
            pal.card_bg,
        )
    } else {
        pal.text_dim
    };

    let border_color = if is_active && master_active {
        pal.primary_border
    } else if is_active {
        iced::Color::from_rgba(0.4, 0.6, 0.9, 0.5)
    } else {
        pal.border
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
    .padding([2, 10])
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(pal.card_bg)),
        border: iced::Border {
            radius: pal.radius(12.0).into(),
            width: 1.0,
            color: border_color,
        },
        ..Default::default()
    })
    .into()
}
