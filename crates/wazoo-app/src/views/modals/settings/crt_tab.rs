/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - CRT Shader Tab
 *
 * Dedicated tab to configure real-time CRT post-processing shader parameters:
 * scanline darkness, density, speed, gamma compensation, saturation, phosphor glow,
 * tube curvature, glass corner rounding, vignette falloff, chromatic aberration, RF noise,
 * and tracking distortion glitch intensity and frequency.
 */

use super::badge::badge_with_reset;
use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme::{self, ThemePalette};
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, slider, text},
};

fn crt_slider<'a>(
    pal: ThemePalette,
    label: String,
    val_str: String,
    is_diff: bool,
    range: std::ops::RangeInclusive<f32>,
    val: f32,
    step: f32,
    on_change: impl Fn(f32) -> Message + 'a,
    reset_msg: Message,
) -> Element<'a, Message> {
    column![
        row![
            text(label).size(13).color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(pal, val_str, is_diff, reset_msg),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(range, val, on_change)
                .step(step)
                .style(theme::settings_slider_style(pal))
                .width(Length::Fill),
        ),
    ]
    .spacing(4)
    .into()
}

pub fn view_crt_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let pal = app.theme_palette;

    // 1. Preset Buttons Card
    let preset_btn = |label: String, preset: wazoo_core::CrtPreset| {
        cursor::PointerCursor::new(
            button(text(label).size(11))
                .style(theme::folder_chip_style(pal, false))
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
                .color(pal.text_dim),
            presets_row,
        ]
        .spacing(6),
    )
    .padding([10, 14])
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 2. Scanlines Section
    let darkness_pct = (app.settings.crt.scanline_darkness * 100.0).round() as i32;
    let darkness_group = crt_slider(
        pal,
        app.t("settings.crt_scanline_darkness"),
        format!("{darkness_pct}%"),
        (app.settings.crt.scanline_darkness - wazoo_core::default_crt_scanline_darkness()).abs()
            > 0.01,
        0.0..=1.0,
        app.settings.crt.scanline_darkness,
        0.01,
        Message::SetCrtScanlineDarkness,
        Message::SetCrtScanlineDarkness(wazoo_core::default_crt_scanline_darkness()),
    );

    let density_val = app.settings.crt.scanline_density.round() as i32;
    let density_group = crt_slider(
        pal,
        app.t("settings.crt_scanline_density"),
        format!("{density_val} lines"),
        (app.settings.crt.scanline_density - wazoo_core::default_crt_scanline_density()).abs()
            > 1.0,
        100.0..=1200.0,
        app.settings.crt.scanline_density,
        10.0,
        Message::SetCrtScanlineDensity,
        Message::SetCrtScanlineDensity(wazoo_core::default_crt_scanline_density()),
    );

    let speed_val = app.settings.crt.scanline_speed;
    let speed_str = if speed_val < 0.1 {
        "Paused".to_string()
    } else {
        format!("{:.1}x", speed_val)
    };
    let speed_group = crt_slider(
        pal,
        app.t("settings.crt_scanline_speed"),
        speed_str,
        (speed_val - wazoo_core::default_crt_scanline_speed()).abs() > 0.05,
        0.0..=10.0,
        speed_val,
        0.5,
        Message::SetCrtScanlineSpeed,
        Message::SetCrtScanlineSpeed(wazoo_core::default_crt_scanline_speed()),
    );

    let scanline_box = container(
        column![
            text(format!("📺 {}", app.t("settings.crt_section_scanlines")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(pal.primary),
            darkness_group,
            density_group,
            speed_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 3. Color & Luminescence Section
    let gamma_val = app.settings.crt.gamma;
    let gamma_group = crt_slider(
        pal,
        app.t("settings.crt_gamma"),
        format!("{:.2}x", gamma_val),
        (gamma_val - wazoo_core::default_crt_gamma()).abs() > 0.01,
        0.5..=2.0,
        gamma_val,
        0.05,
        Message::SetCrtGamma,
        Message::SetCrtGamma(wazoo_core::default_crt_gamma()),
    );

    let sat_pct = (app.settings.crt.saturation * 100.0).round() as i32;
    let sat_group = crt_slider(
        pal,
        app.t("settings.crt_saturation"),
        format!("{sat_pct}%"),
        (app.settings.crt.saturation - wazoo_core::default_crt_saturation()).abs() > 0.01,
        0.0..=2.0,
        app.settings.crt.saturation,
        0.05,
        Message::SetCrtSaturation,
        Message::SetCrtSaturation(wazoo_core::default_crt_saturation()),
    );

    let bright_pct = (app.settings.crt.brightness * 100.0).round() as i32;
    let bright_group = crt_slider(
        pal,
        app.t("settings.crt_brightness"),
        format!("{bright_pct}%"),
        (app.settings.crt.brightness - wazoo_core::default_crt_brightness()).abs() > 0.01,
        0.5..=2.0,
        app.settings.crt.brightness,
        0.05,
        Message::SetCrtBrightness,
        Message::SetCrtBrightness(wazoo_core::default_crt_brightness()),
    );

    let color_box = container(
        column![
            text(format!("🌈 {}", app.t("settings.crt_section_color")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(pal.primary),
            gamma_group,
            sat_group,
            bright_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 4. Tube Geometry Section
    let curv_pct = (app.settings.crt.curvature * 100.0).round() as i32;
    let curv_group = crt_slider(
        pal,
        app.t("settings.crt_curvature"),
        if curv_pct == 0 {
            "Flat".to_string()
        } else {
            format!("{curv_pct}%")
        },
        (app.settings.crt.curvature - wazoo_core::default_crt_curvature()).abs() > 0.01,
        0.0..=1.0,
        app.settings.crt.curvature,
        0.02,
        Message::SetCrtCurvature,
        Message::SetCrtCurvature(wazoo_core::default_crt_curvature()),
    );

    let corner_pct = (app.settings.crt.corner_rounding * 100.0).round() as i32;
    let corner_group = crt_slider(
        pal,
        app.t("settings.crt_corner_rounding"),
        if corner_pct == 0 {
            "Sharp".to_string()
        } else {
            format!("{corner_pct}%")
        },
        (app.settings.crt.corner_rounding - wazoo_core::default_crt_corner_rounding()).abs() > 0.01,
        0.0..=1.0,
        app.settings.crt.corner_rounding,
        0.02,
        Message::SetCrtCornerRounding,
        Message::SetCrtCornerRounding(wazoo_core::default_crt_corner_rounding()),
    );

    let vig_pct = (app.settings.crt.vignette * 100.0).round() as i32;
    let vig_group = crt_slider(
        pal,
        app.t("settings.crt_vignette"),
        if vig_pct == 0 {
            app.t("common.none")
        } else {
            format!("{vig_pct}%")
        },
        (app.settings.crt.vignette - wazoo_core::default_crt_vignette()).abs() > 0.01,
        0.0..=1.0,
        app.settings.crt.vignette,
        0.02,
        Message::SetCrtVignette,
        Message::SetCrtVignette(wazoo_core::default_crt_vignette()),
    );

    let geometry_box = container(
        column![
            text(format!("🔲 {}", app.t("settings.crt_section_geometry")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(pal.primary),
            curv_group,
            corner_group,
            vig_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 5. Analog Artifacts Section
    let ca_pct = (app.settings.crt.chromatic_aberration * 100.0).round() as i32;
    let ca_group = crt_slider(
        pal,
        app.t("settings.crt_chromatic_aberration"),
        if ca_pct == 0 {
            app.t("common.none")
        } else {
            format!("{ca_pct}%")
        },
        (app.settings.crt.chromatic_aberration - wazoo_core::default_crt_chromatic_aberration())
            .abs()
            > 0.01,
        0.0..=1.0,
        app.settings.crt.chromatic_aberration,
        0.02,
        Message::SetCrtChromaticAberration,
        Message::SetCrtChromaticAberration(wazoo_core::default_crt_chromatic_aberration()),
    );

    let noise_pct = (app.settings.crt.noise * 100.0).round() as i32;
    let noise_group = crt_slider(
        pal,
        app.t("settings.crt_noise"),
        if noise_pct == 0 {
            "Clean".to_string()
        } else {
            format!("{noise_pct}%")
        },
        (app.settings.crt.noise - wazoo_core::default_crt_noise()).abs() > 0.01,
        0.0..=1.0,
        app.settings.crt.noise,
        0.02,
        Message::SetCrtNoise,
        Message::SetCrtNoise(wazoo_core::default_crt_noise()),
    );

    let glitch_int_pct = (app.settings.crt.glitch_intensity * 100.0).round() as i32;
    let glitch_intensity_group = crt_slider(
        pal,
        app.t("settings.crt_glitch_intensity"),
        if glitch_int_pct == 0 {
            app.t("common.none")
        } else {
            format!("{glitch_int_pct}%")
        },
        (app.settings.crt.glitch_intensity - wazoo_core::default_crt_glitch_intensity()).abs()
            > 0.005,
        0.0..=1.0,
        app.settings.crt.glitch_intensity,
        0.01,
        Message::SetCrtGlitchIntensity,
        Message::SetCrtGlitchIntensity(wazoo_core::default_crt_glitch_intensity()),
    );

    let glitch_freq_pct = (app.settings.crt.glitch_frequency * 100.0).round() as i32;
    let glitch_frequency_group = crt_slider(
        pal,
        app.t("settings.crt_glitch_frequency"),
        if glitch_freq_pct == 0 {
            app.t("common.never")
        } else {
            format!("{glitch_freq_pct}%")
        },
        (app.settings.crt.glitch_frequency - wazoo_core::default_crt_glitch_frequency()).abs()
            > 0.005,
        0.0..=1.0,
        app.settings.crt.glitch_frequency,
        0.01,
        Message::SetCrtGlitchFrequency,
        Message::SetCrtGlitchFrequency(wazoo_core::default_crt_glitch_frequency()),
    );

    let artifacts_box = container(
        column![
            text(format!("📻 {}", app.t("settings.crt_section_artifacts")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(pal.primary),
            ca_group,
            noise_group,
            glitch_intensity_group,
            glitch_frequency_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(theme::settings_card_style(pal))
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
        .style(theme::settings_reset_button_style(pal))
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
