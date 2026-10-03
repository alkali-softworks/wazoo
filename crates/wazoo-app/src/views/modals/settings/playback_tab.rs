/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - Playback Tab
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

pub fn view_playback_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let gamma_val = app.settings.gamma.round() as i32;
    let gamma_str = if gamma_val > 0 {
        format!("+{gamma_val}")
    } else {
        format!("{gamma_val}")
    };
    let gamma_group = column![
        row![
            text(app.t("settings.gamma"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(gamma_str, gamma_val != 0, Message::SetGamma(0.0)),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(-100.0..=100.0, app.settings.gamma, Message::SetGamma)
                .step(1.0_f32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let contrast_val = app.settings.contrast.round() as i32;
    let contrast_str = if contrast_val > 0 {
        format!("+{contrast_val}")
    } else {
        format!("{contrast_val}")
    };
    let contrast_group = column![
        row![
            text(app.t("settings.contrast"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(contrast_str, contrast_val != 0, Message::SetContrast(0.0)),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(-100.0..=100.0, app.settings.contrast, Message::SetContrast)
                .step(1.0_f32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let brightness_val = app.settings.brightness.round() as i32;
    let brightness_str = if brightness_val > 0 {
        format!("+{brightness_val}")
    } else {
        format!("{brightness_val}")
    };
    let brightness_group = column![
        row![
            text(app.t("settings.brightness"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                brightness_str,
                brightness_val != 0,
                Message::SetBrightness(0.0)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                -100.0..=100.0,
                app.settings.brightness,
                Message::SetBrightness
            )
            .step(1.0_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let saturation_val = app.settings.saturation.round() as i32;
    let saturation_str = if saturation_val > 0 {
        format!("+{saturation_val}")
    } else {
        format!("{saturation_val}")
    };
    let saturation_group = column![
        row![
            text(app.t("settings.saturation"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                saturation_str,
                saturation_val != 0,
                Message::SetSaturation(0.0)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                -100.0..=100.0,
                app.settings.saturation,
                Message::SetSaturation
            )
            .step(1.0_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let speed_str = format!("{:.2}x", app.settings.playback_speed);
    let speed_active = (app.settings.playback_speed - 1.0).abs() > 0.01;
    let speed_group = column![
        row![
            text(app.t("settings.playback_speed"))
                .size(14)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(speed_str, speed_active, Message::SetPlaybackSpeed(1.0)),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.25..=3.0,
                app.settings.playback_speed,
                Message::SetPlaybackSpeed
            )
            .step(0.05_f32)
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

    let reset_btn = button(
        row![
            text("↺").size(14),
            text(app.t("settings.reset_playback")).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .style(theme::action_button_style)
    .on_press(Message::ResetPlaybackOptions)
    .padding([8, 16]);
    column![
        loading_indicator_group,
        gamma_group,
        contrast_group,
        brightness_group,
        saturation_group,
        speed_group,
        reset_btn,
    ]
    .spacing(16)
    .into()
}
