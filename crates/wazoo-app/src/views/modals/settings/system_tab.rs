/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - System Tab
 */

use super::badge::badge_with_reset;
use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, row, slider, text},
};

pub fn view_system_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let opacity_val = (app.settings.window_opacity * 100.0).round() as u32;
    let opacity_active = (app.settings.window_opacity - 1.0).abs() > 0.01;
    let opacity_group = column![
        row![
            text(app.t("settings.window_opacity"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                format!("{opacity_val}%"),
                opacity_active,
                Message::SetWindowOpacity(1.0)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.05..=1.0,
                app.settings.window_opacity,
                Message::SetWindowOpacity
            )
            .step(0.01_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let buffer_dur_str = format!("{}s", app.settings.buffer_duration_secs);
    let buffer_dur_active = app.settings.buffer_duration_secs != 10;
    let buffer_dur_group = column![
        row![
            text(app.t("settings.buffer_duration"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                buffer_dur_str,
                buffer_dur_active,
                Message::SetBufferDuration(10)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                2..=60,
                app.settings.buffer_duration_secs,
                Message::SetBufferDuration
            )
            .step(1_u32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let buffer_size_str = format!("{} MB", app.settings.buffer_size_mb);
    let buffer_size_active = app.settings.buffer_size_mb != 32;
    let buffer_size_group = column![
        row![
            text(app.t("settings.buffer_size"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                buffer_size_str,
                buffer_size_active,
                Message::SetBufferSize(32)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(8..=512, app.settings.buffer_size_mb, Message::SetBufferSize)
                .step(8_u32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let is_pinned = app.settings.is_always_on_top;
    let always_on_top_group = system_toggle_button(
        app,
        if is_pinned { "📌" } else { "📍" },
        app.t("settings.always_on_top"),
        app.t("settings.always_on_top_desc"),
        is_pinned,
        Message::ToggleAlwaysOnTop,
    );

    let is_default = app.settings.is_default_player;
    let default_player_desc = if is_default {
        app.t("settings.default_player_desc_active")
    } else {
        app.t("settings.default_player_desc")
    };
    let default_player_group = system_toggle_button(
        app,
        if is_default { "🎬" } else { "🎞️" },
        app.t("settings.default_player"),
        default_player_desc,
        is_default,
        Message::ToggleDefaultPlayer,
    );

    let flip_interval_str = format!("{}s", app.settings.flip_interval_secs);
    let flip_interval_active =
        app.settings.flip_interval_secs != wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS;
    let flip_interval_group = column![
        row![
            text(app.t("settings.flip_interval"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                flip_interval_str,
                flip_interval_active,
                Message::SetFlipInterval(wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                1..=300_u32,
                app.settings.flip_interval_secs.clamp(1, 300) as u32,
                |secs| Message::SetFlipInterval(secs as u64),
            )
            .step(1_u32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let is_standby = app.settings.use_standby_player;
    let standby_player_group = system_toggle_button(
        app,
        if is_standby { "⚡" } else { "💤" },
        app.t("settings.standby_player"),
        app.t("settings.standby_player_desc"),
        is_standby,
        Message::ToggleStandbyPlayer,
    );

    column![
        always_on_top_group,
        opacity_group,
        default_player_group,
        flip_interval_group,
        buffer_dur_group,
        buffer_size_group,
        standby_player_group,
    ]
    .spacing(16)
    .into()
}

fn system_toggle_button<'a>(
    app: &'a WazooApp,
    icon: &'static str,
    title: String,
    desc: String,
    is_active: bool,
    toggle_msg: Message,
) -> Element<'a, Message> {
    let badge_text = if is_active {
        app.t("settings.badge_on")
    } else {
        app.t("settings.badge_off")
    };

    cursor::PointerCursor::new(
        button(
            row![
                text(icon).size(16),
                column![
                    text(title)
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if is_active {
                            theme::COLOR_PRIMARY
                        } else {
                            iced::Color::WHITE
                        }),
                    text(desc).size(11).color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    badge_text,
                    is_active,
                    toggle_msg.clone(),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if is_active {
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
                    color: if is_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(toggle_msg)
        .padding([12, 16])
        .width(Length::Fill),
    )
    .into()
}
