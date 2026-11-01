/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - CRT Shader Tab
 *
 * Dedicated tab to configure real-time CRT post-processing shader parameters:
 * scanline darkness, density, speed, gamma compensation, saturation, phosphor glow,
 * tube curvature, glass corner rounding, vignette falloff, chromatic aberration, and RF noise.
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

pub fn view_crt_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    // 1. Preset Buttons Card

    let preset_btn = |label: String, preset: wazoo_core::CrtPreset| {
        cursor::PointerCursor::new(
            button(text(label).size(11))
                .style(theme::folder_chip_style(false))
                .on_press(Message::ApplyCrtPreset(preset))
                .padding([5, 10]),
        )
    };

    let presets_row = row![
        preset_btn(
            format!("☕ {}", app.t("settings.crt_preset_default")),
            wazoo_core::CrtPreset::Default
        ),
        preset_btn(
            format!("✨ {}", app.t("settings.crt_preset_subtle")),
            wazoo_core::CrtPreset::Subtle
        ),
        preset_btn(
            format!("👾 {}", app.t("settings.crt_preset_arcade")),
            wazoo_core::CrtPreset::Arcade
        ),
        preset_btn(
            format!("🔥 {}", app.t("settings.crt_preset_warm")),
            wazoo_core::CrtPreset::WarmPhosphor
        ),
        preset_btn(
            format!("📺 {}", app.t("settings.crt_preset_bw")),
            wazoo_core::CrtPreset::Monochrome
        ),
        preset_btn(
            format!("📼 {}", app.t("settings.crt_preset_vhs")),
            wazoo_core::CrtPreset::Vhs
        ),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let presets_card = container(
        column![
            text(app.t("settings.crt_presets"))
                .size(11)
                .font(theme::FONT_BOLD)
                .color(theme::COLOR_TEXT_DIM),
            presets_row,
        ]
        .spacing(6),
    )
    .padding([10, 14])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // 2. Scanlines Section
    let darkness_pct = (app.settings.crt.scanline_darkness * 100.0).round() as i32;
    let darkness_str = format!("{darkness_pct}%");
    let darkness_diff =
        (app.settings.crt.scanline_darkness - wazoo_core::default_crt_scanline_darkness()).abs()
            > 0.01;
    let darkness_group = column![
        row![
            text(app.t("settings.crt_scanline_darkness"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                darkness_str,
                darkness_diff,
                Message::SetCrtScanlineDarkness(wazoo_core::default_crt_scanline_darkness()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.scanline_darkness,
                Message::SetCrtScanlineDarkness,
            )
            .step(0.01_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let density_val = app.settings.crt.scanline_density.round() as i32;
    let density_str = format!("{density_val} lines");
    let density_diff =
        (app.settings.crt.scanline_density - wazoo_core::default_crt_scanline_density()).abs()
            > 1.0;
    let density_group = column![
        row![
            text(app.t("settings.crt_scanline_density"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                density_str,
                density_diff,
                Message::SetCrtScanlineDensity(wazoo_core::default_crt_scanline_density()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                100.0..=1200.0,
                app.settings.crt.scanline_density,
                Message::SetCrtScanlineDensity,
            )
            .step(10.0_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let speed_val = app.settings.crt.scanline_speed;
    let speed_str = if speed_val < 0.1 {
        "Paused".to_string()
    } else {
        format!("{:.1}x", speed_val)
    };
    let speed_diff =
        (app.settings.crt.scanline_speed - wazoo_core::default_crt_scanline_speed()).abs() > 0.05;
    let speed_group = column![
        row![
            text(app.t("settings.crt_scanline_speed"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                speed_str,
                speed_diff,
                Message::SetCrtScanlineSpeed(wazoo_core::default_crt_scanline_speed()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=10.0,
                app.settings.crt.scanline_speed,
                Message::SetCrtScanlineSpeed,
            )
            .step(0.5_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let scanline_box = container(
        column![
            text(format!("📺 {}", app.t("settings.crt_section_scanlines")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(theme::COLOR_PRIMARY),
            darkness_group,
            density_group,
            speed_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // 3. Color & Luminescence Section (Directly solves user's scanline darkness & gamma compensation)
    let gamma_val = app.settings.crt.gamma;
    let gamma_str = format!("{:.2}x", gamma_val);
    let gamma_diff = (gamma_val - wazoo_core::default_crt_gamma()).abs() > 0.01;
    let gamma_group = column![
        row![
            text(app.t("settings.crt_gamma"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                gamma_str,
                gamma_diff,
                Message::SetCrtGamma(wazoo_core::default_crt_gamma()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(0.5..=2.0, app.settings.crt.gamma, Message::SetCrtGamma)
                .step(0.05_f32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let sat_pct = (app.settings.crt.saturation * 100.0).round() as i32;
    let sat_str = format!("{sat_pct}%");
    let sat_diff =
        (app.settings.crt.saturation - wazoo_core::default_crt_saturation()).abs() > 0.01;
    let sat_group = column![
        row![
            text(app.t("settings.crt_saturation"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                sat_str,
                sat_diff,
                Message::SetCrtSaturation(wazoo_core::default_crt_saturation()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=2.0,
                app.settings.crt.saturation,
                Message::SetCrtSaturation,
            )
            .step(0.05_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let bright_pct = (app.settings.crt.brightness * 100.0).round() as i32;
    let bright_str = format!("{bright_pct}%");
    let bright_diff =
        (app.settings.crt.brightness - wazoo_core::default_crt_brightness()).abs() > 0.01;
    let bright_group = column![
        row![
            text(app.t("settings.crt_brightness"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                bright_str,
                bright_diff,
                Message::SetCrtBrightness(wazoo_core::default_crt_brightness()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.5..=2.0,
                app.settings.crt.brightness,
                Message::SetCrtBrightness,
            )
            .step(0.05_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let color_box = container(
        column![
            text(format!("🌈 {}", app.t("settings.crt_section_color")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(theme::COLOR_PRIMARY),
            gamma_group,
            sat_group,
            bright_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // 4. Tube Geometry Section
    let curv_pct = (app.settings.crt.curvature * 100.0).round() as i32;
    let curv_str = if curv_pct == 0 {
        "Flat".to_string()
    } else {
        format!("{curv_pct}%")
    };
    let curv_diff =
        (app.settings.crt.curvature - wazoo_core::default_crt_curvature()).abs() > 0.01;
    let curv_group = column![
        row![
            text(app.t("settings.crt_curvature"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                curv_str,
                curv_diff,
                Message::SetCrtCurvature(wazoo_core::default_crt_curvature()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.curvature,
                Message::SetCrtCurvature,
            )
            .step(0.02_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let corner_pct = (app.settings.crt.corner_rounding * 100.0).round() as i32;
    let corner_str = if corner_pct == 0 {
        "Sharp".to_string()
    } else {
        format!("{corner_pct}%")
    };
    let corner_diff = (app.settings.crt.corner_rounding
        - wazoo_core::default_crt_corner_rounding())
    .abs()
        > 0.01;
    let corner_group = column![
        row![
            text(app.t("settings.crt_corner_rounding"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                corner_str,
                corner_diff,
                Message::SetCrtCornerRounding(wazoo_core::default_crt_corner_rounding()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.corner_rounding,
                Message::SetCrtCornerRounding,
            )
            .step(0.02_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let vig_pct = (app.settings.crt.vignette * 100.0).round() as i32;
    let vig_str = if vig_pct == 0 {
        "None".to_string()
    } else {
        format!("{vig_pct}%")
    };
    let vig_diff = (app.settings.crt.vignette - wazoo_core::default_crt_vignette()).abs() > 0.01;
    let vig_group = column![
        row![
            text(app.t("settings.crt_vignette"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                vig_str,
                vig_diff,
                Message::SetCrtVignette(wazoo_core::default_crt_vignette()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.vignette,
                Message::SetCrtVignette,
            )
            .step(0.02_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let geometry_box = container(
        column![
            text(format!("🔲 {}", app.t("settings.crt_section_geometry")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(theme::COLOR_PRIMARY),
            curv_group,
            corner_group,
            vig_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // 5. Analog Artifacts Section
    let ca_pct = (app.settings.crt.chromatic_aberration * 100.0).round() as i32;
    let ca_str = if ca_pct == 0 {
        "None".to_string()
    } else {
        format!("{ca_pct}%")
    };
    let ca_diff = (app.settings.crt.chromatic_aberration
        - wazoo_core::default_crt_chromatic_aberration())
    .abs()
        > 0.01;
    let ca_group = column![
        row![
            text(app.t("settings.crt_chromatic_aberration"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                ca_str,
                ca_diff,
                Message::SetCrtChromaticAberration(
                    wazoo_core::default_crt_chromatic_aberration(),
                ),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.chromatic_aberration,
                Message::SetCrtChromaticAberration,
            )
            .step(0.02_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let noise_pct = (app.settings.crt.noise * 100.0).round() as i32;
    let noise_str = if noise_pct == 0 {
        "Clean".to_string()
    } else {
        format!("{noise_pct}%")
    };
    let noise_diff = (app.settings.crt.noise - wazoo_core::default_crt_noise()).abs() > 0.01;
    let noise_group = column![
        row![
            text(app.t("settings.crt_noise"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            badge_with_reset(
                noise_str,
                noise_diff,
                Message::SetCrtNoise(wazoo_core::default_crt_noise()),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=1.0,
                app.settings.crt.noise,
                Message::SetCrtNoise,
            )
            .step(0.02_f32)
            .style(theme::settings_slider_style)
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let artifacts_box = container(
        column![
            text(format!("📻 {}", app.t("settings.crt_section_artifacts")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(theme::COLOR_PRIMARY),
            ca_group,
            noise_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: theme::COLOR_BORDER,
        },
        ..Default::default()
    })
    .width(Length::Fill);

    // 6. Reset All Button
    let reset_all_button = cursor::PointerCursor::new(
        button(
            row![
                text("↺").size(13),
                text(app.t("settings.crt_reset_all")).size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_reset_button_style)
        .on_press(Message::ResetCrtSettings)
        .padding([8, 16]),
    );

    let reset_row = row![Space::new().width(Length::Fill), reset_all_button]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    column![
        presets_card,
        scanline_box,
        color_box,
        geometry_box,
        artifacts_box,
        reset_row,
    ]
    .spacing(12)
    .into()
}
