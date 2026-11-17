/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - System Tab
 */

use super::badge::badge_with_reset;
use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme::{self, ThemePalette};
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, row, slider, text},
};

pub fn view_system_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let pal = app.theme_palette;
    let opacity_val = (app.settings.window_opacity * 100.0).round() as u32;
    let opacity_active = (app.settings.window_opacity - 1.0).abs() > 0.01;
    let opacity_group = column![
        row![
            text(app.t("settings.window_opacity"))
                .size(14)
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
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
            .style(theme::settings_slider_style(pal))
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
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
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
            .style(theme::settings_slider_style(pal))
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
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                buffer_size_str,
                buffer_size_active,
                Message::SetBufferSize(32)
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(8..=512, app.settings.buffer_size_mb, Message::SetBufferSize)
                .step(8_u32)
                .style(theme::settings_slider_style(pal))
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let is_pinned = app.settings.is_always_on_top;
    let always_on_top_group = system_toggle_button(
        app,
        pal,
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
        pal,
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
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
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
            .style(theme::settings_slider_style(pal))
            .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let is_standby = app.settings.use_standby_player;
    let standby_player_group = system_toggle_button(
        app,
        pal,
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
    pal: ThemePalette,
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
                        .color(if is_active { pal.primary } else { pal.text }),
                    text(desc).size(11).color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(pal, badge_text, is_active, toggle_msg.clone()),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_toggle_card_style(pal, is_active))
        .on_press(toggle_msg)
        .padding([12, 16])
        .width(Length::Fill),
    )
    .into()
}
