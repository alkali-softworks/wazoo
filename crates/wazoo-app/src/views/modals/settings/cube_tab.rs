/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - 3D Cube Tab
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

pub fn view_cube_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let pal = app.theme_palette;
    let cube_active = app.cube.is_present();
    let cube_count = app.cube.cubes.len();

    // 1. 3D Cube Toggle (Mode 8)
    let toggle_button = cursor::PointerCursor::new(
        button(
            row![
                text(if cube_active { "⏹" } else { "▶" }).size(16),
                column![
                    text(if cube_active {
                        app.t("settings.cube_disable")
                    } else {
                        app.t("settings.cube_enable")
                    })
                    .size(13)
                    .font(theme::FONT_BOLD)
                    .color(if cube_active { pal.primary } else { pal.text }),
                    text(app.t_with(
                        "settings.cube_shortcut_hint",
                        &[(
                            "key",
                            &app.settings
                                .keybinds
                                .menu_hint(&app.settings.keybinds.toggle_cube)
                        )],
                    ))
                    .size(11)
                    .color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    pal,
                    if cube_active {
                        app.t_with(
                            "settings.cube_badge_on",
                            &[("count", &cube_count.to_string())],
                        )
                    } else {
                        app.t("settings.cube_badge_off")
                    },
                    cube_active,
                    Message::ToggleCubeScreensaver,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_toggle_card_style(pal, cube_active))
        .on_press(Message::ToggleCubeScreensaver)
        .padding([12, 16])
        .width(Length::Fill),
    );

    // 2. Cube Overlay Mode Toggle (Mode 9)
    let desktop_active = app.cube.desktop_overlay;
    let desktop_button = cursor::PointerCursor::new(
        button(
            row![
                text(if desktop_active { "⏹" } else { "🖥️" }).size(16),
                column![
                    text(if desktop_active {
                        app.t("settings.cube_desktop_disable")
                    } else {
                        app.t("settings.cube_desktop_enable")
                    })
                    .size(13)
                    .font(theme::FONT_BOLD)
                    .color(if desktop_active { pal.primary } else { pal.text }),
                    text(app.t_with(
                        "settings.cube_desktop_shortcut_hint",
                        &[(
                            "key",
                            &app.settings
                                .keybinds
                                .menu_hint(&app.settings.keybinds.toggle_cube_overlay)
                        )],
                    ))
                    .size(11)
                    .color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    pal,
                    if desktop_active {
                        app.t("settings.cube_badge_desktop_on")
                    } else {
                        app.t("settings.cube_badge_off")
                    },
                    desktop_active,
                    Message::ToggleDesktopCubeScreensaver,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_toggle_card_style(pal, desktop_active))
        .on_press(Message::ToggleDesktopCubeScreensaver)
        .padding([12, 16])
        .width(Length::Fill),
    );

    // 3. 3D Light & Sheen Toggle
    let sheen_active = app.settings.cube_sheen;
    let sheen_button = cursor::PointerCursor::new(
        button(
            row![
                text(if sheen_active { "✨" } else { "💡" }).size(16),
                column![
                    text(app.t("settings.cube_sheen"))
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if sheen_active { pal.primary } else { pal.text }),
                    text(app.t("settings.cube_sheen_desc"))
                        .size(11)
                        .color(pal.text_muted),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    pal,
                    if sheen_active {
                        app.t("settings.badge_on")
                    } else {
                        app.t("settings.badge_off")
                    },
                    sheen_active,
                    Message::ToggleCubeSheen,
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_toggle_card_style(pal, sheen_active))
        .on_press(Message::ToggleCubeSheen)
        .padding([12, 16])
        .width(Length::Fill),
    );

    // 4. Movement & Size Controls (Speed & Size)
    let speed_val = app.cube.speed_multiplier;
    let speed_label = format!("{:.1}x", speed_val);
    let speed_slider = column![
        row![
            column![
                text(app.t("settings.cube_speed")).size(13).color(pal.text),
                text(app.t("settings.cube_speed_desc"))
                    .size(11)
                    .color(pal.text_muted),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                speed_label,
                (speed_val - 1.0).abs() > 0.05,
                Message::SetCubeSpeed(1.0),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(0.2..=3.0, app.cube.speed_multiplier, Message::SetCubeSpeed)
                .step(0.1_f32)
                .style(theme::settings_slider_style(pal))
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let size_val = app.cube.size_multiplier;
    let size_label = format!("{:.1}x", size_val);
    let size_slider = column![
        row![
            column![
                text(app.t("settings.cube_size")).size(13).color(pal.text),
                text(app.t("settings.cube_size_desc"))
                    .size(11)
                    .color(pal.text_muted),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                size_label,
                (size_val - 1.0).abs() > 0.05,
                Message::SetCubeSize(1.0),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(0.5..=2.5, app.cube.size_multiplier, Message::SetCubeSize)
                .step(0.1_f32)
                .style(theme::settings_slider_style(pal))
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    // 5. Cube Controls (Add / Remove)
    let spawn_more_btn = cursor::PointerCursor::new(
        button(
            row![text("+").size(14), text(app.t("settings.cube_spawn")).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .style(theme::action_button_style(pal))
        .on_press(Message::SpawnCube)
        .padding([8, 14]),
    );

    let dismiss_all_btn = cursor::PointerCursor::new(
        button(
            row![text("✕").size(12), text(app.t("settings.cube_dismiss_all")).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .style(theme::action_button_style(pal))
        .on_press(Message::ClearCubes)
        .padding([8, 14]),
    );

    let mut actions_row = row![spawn_more_btn].spacing(10);
    if cube_active {
        actions_row = actions_row.push(dismiss_all_btn);
    }

    column![
        toggle_button,
        desktop_button,
        sheen_button,
        speed_slider,
        size_slider,
        actions_row,
    ]
    .spacing(14)
    .into()
}
