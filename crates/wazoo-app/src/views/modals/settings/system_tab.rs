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
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, slider, text},
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

    let default_player_toggle = {
        let is_default = app.settings.is_default_player;
        let icon = if is_default { "✓ " } else { "" };
        let label = format!("{}{}", icon, app.t("settings.default_player"));
        button(text(label).size(12).font(if is_default {
            theme::FONT_BOLD
        } else {
            Default::default()
        }))
        .style(theme::folder_chip_style(is_default))
        .on_press(Message::ToggleDefaultPlayer)
        .padding([6, 14])
    };

    let default_player_group = container(
        row![
            column![
                text(app.t("settings.default_player"))
                    .size(14)
                    .color(iced::Color::WHITE),
                text(if app.settings.is_default_player {
                    "Associated as default system player for media files"
                } else {
                    "Set as default system player for media files"
                })
                .size(12)
                .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            default_player_toggle,
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 14])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 6.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    });

    let always_on_top_toggle = {
        let is_pinned = app.settings.is_always_on_top;
        let icon = if is_pinned { "✓ " } else { "" };
        let label = format!("{}{}", icon, app.t("settings.always_on_top"));
        button(text(label).size(12).font(if is_pinned {
            theme::FONT_BOLD
        } else {
            Default::default()
        }))
        .style(theme::folder_chip_style(is_pinned))
        .on_press(Message::ToggleAlwaysOnTop)
        .padding([6, 14])
    };

    let always_on_top_group = container(
        row![
            column![
                text(app.t("settings.always_on_top"))
                    .size(14)
                    .color(iced::Color::WHITE),
                text(app.t("settings.always_on_top_desc"))
                    .size(12)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            always_on_top_toggle,
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 14])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 6.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    });

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

    let is_tv_static = app.settings.loading_indicator == wazoo_core::LoadingIndicator::TvStatic;
    let is_spinner = app.settings.loading_indicator == wazoo_core::LoadingIndicator::Spinner;

    let tv_static_btn = cursor::PointerCursor::new(
        button(
            text(format!(
                "{}{}",
                if is_tv_static { "✓ " } else { "" },
                app.t("settings.loading_tv_static")
            ))
            .size(12)
            .font(if is_tv_static {
                theme::FONT_BOLD
            } else {
                Default::default()
            }),
        )
        .style(theme::folder_chip_style(is_tv_static))
        .on_press(Message::SetLoadingIndicator(
            wazoo_core::LoadingIndicator::TvStatic,
        ))
        .padding([6, 14]),
    );

    let spinner_btn = cursor::PointerCursor::new(
        button(
            text(format!(
                "{}{}",
                if is_spinner { "✓ " } else { "" },
                app.t("settings.loading_spinner")
            ))
            .size(12)
            .font(if is_spinner {
                theme::FONT_BOLD
            } else {
                Default::default()
            }),
        )
        .style(theme::folder_chip_style(is_spinner))
        .on_press(Message::SetLoadingIndicator(
            wazoo_core::LoadingIndicator::Spinner,
        ))
        .padding([6, 14]),
    );

    let loading_indicator_group = container(
        row![
            column![
                text(app.t("settings.loading_indicator"))
                    .size(14)
                    .color(iced::Color::WHITE),
                text(app.t("settings.loading_indicator_desc"))
                    .size(12)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            row![spinner_btn, tv_static_btn].spacing(8),
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 14])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 6.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    });

    column![
        always_on_top_group,
        opacity_group,
        loading_indicator_group,
        default_player_group,
        flip_interval_group,
        buffer_dur_group,
        buffer_size_group,
    ]
    .spacing(16)
    .into()
}
