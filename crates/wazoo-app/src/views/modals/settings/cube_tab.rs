/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - 3D Video Cube Screensaver Tab
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

pub fn view_cube_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let cube_active = app.cube.is_present();
    let cube_count = app.cube.cubes.len();

    // 1. Header / Status banner
    let status_indicator = if cube_active {
        let active_text = if cube_count == 1 {
            app.t("settings.cube_active_single")
        } else {
            app.t_with(
                "settings.cube_active_multiple",
                &[("count", &cube_count.to_string())],
            )
        };
        container(
            row![
                text("●").size(10).color(theme::COLOR_PRIMARY),
                text(active_text)
                    .size(11)
                    .font(theme::FONT_BOLD)
                    .color(theme::COLOR_PRIMARY),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.26, 0.72, 0.51, 0.15,
            ))),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(0.26, 0.72, 0.51, 0.4),
            },
            ..Default::default()
        })
    } else {
        container(
            row![
                text("●").size(10).color(theme::COLOR_TEXT_MUTED),
                text(app.t("settings.cube_inactive"))
                    .size(11)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .padding([4, 10])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.2, 0.2, 0.22, 0.4,
            ))),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: theme::COLOR_BORDER,
            },
            ..Default::default()
        })
    };

    let header_card = container(
        column![
            row![
                text("🧊").size(22),
                column![
                    row![
                        text(app.t("settings.cube_title"))
                            .size(15)
                            .font(theme::FONT_BOLD)
                            .color(iced::Color::WHITE),
                        container(
                            text(app.t("settings.cube_experimental"))
                                .size(9)
                                .font(theme::FONT_BOLD)
                                .color(iced::Color::from_rgb(0.0, 0.9, 1.0))
                        )
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.0, 0.9, 1.0, 0.12,
                            ))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                width: 1.0,
                                color: iced::Color::from_rgba(0.0, 0.9, 1.0, 0.35),
                            },
                            ..Default::default()
                        }),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text(app.t("settings.cube_desc"))
                        .size(12)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(3),
                Space::new().width(Length::Fill),
                status_indicator,
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(8),
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
    });

    // 2. Master Toggle Button
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
                    .color(if cube_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::WHITE
                    }),
                    text(app.t("settings.cube_shortcut_hint"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
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
        .style(move |_theme: &iced::Theme, status| {
            let bg = if cube_active {
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
                    color: if cube_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::ToggleCubeScreensaver)
        .padding([12, 16])
        .width(Length::Fill),
    );

    // 2b. Desktop Screensaver Mode Toggle (Click-Passthrough)
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
                    .color(if desktop_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::WHITE
                    }),
                    text(app.t("settings.cube_desktop_shortcut_hint"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
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
        .style(move |_theme: &iced::Theme, status| {
            let bg = if desktop_active {
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
                    color: if desktop_active {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::ToggleDesktopCubeScreensaver)
        .padding([12, 16])
        .width(Length::Fill),
    );

    // 3. Physics & Dynamics Controls (Speed & Size)
    let speed_val = app.cube.speed_multiplier;
    let speed_label = format!("{:.1}x", speed_val);
    let speed_slider = column![
        row![
            column![
                text(app.t("settings.cube_speed"))
                    .size(13)
                    .color(iced::Color::WHITE),
                text(app.t("settings.cube_speed_desc"))
                    .size(11)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            badge_with_reset(
                speed_label,
                (speed_val - 1.0).abs() > 0.05,
                Message::SetCubeSpeed(1.0),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(0.2..=3.0, app.cube.speed_multiplier, Message::SetCubeSpeed)
                .step(0.1_f32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    let size_val = app.cube.size_multiplier;
    let size_label = format!("{:.1}x", size_val);
    let size_slider = column![
        row![
            column![
                text(app.t("settings.cube_size"))
                    .size(13)
                    .color(iced::Color::WHITE),
                text(app.t("settings.cube_size_desc"))
                    .size(11)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(1),
            Space::new().width(Length::Fill),
            badge_with_reset(
                size_label,
                (size_val - 1.0).abs() > 0.05,
                Message::SetCubeSize(1.0),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(0.5..=2.5, app.cube.size_multiplier, Message::SetCubeSize)
                .step(0.1_f32)
                .style(theme::settings_slider_style)
                .width(Length::Fill),
        ),
    ]
    .spacing(6);

    // 4. Fleet Management (Spawn / Dismiss)
    let spawn_more_btn = cursor::PointerCursor::new(
        button(
            row![text("+").size(14), text(app.t("settings.cube_spawn")).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .style(theme::action_button_style)
        .on_press(Message::SpawnCube)
        .padding([8, 14]),
    );

    let dismiss_all_btn = cursor::PointerCursor::new(
        button(
            row![text("✕").size(12), text(app.t("settings.cube_dismiss_all")).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .style(theme::action_button_style)
        .on_press(Message::ClearCubes)
        .padding([8, 14]),
    );

    let dynamics_card = container(
        column![
            text(app.t("settings.cube_physics_scale"))
                .size(14)
                .font(theme::FONT_BOLD)
                .color(iced::Color::WHITE),
            speed_slider,
            size_slider,
            row![spawn_more_btn, dismiss_all_btn].spacing(10),
        ]
        .spacing(14),
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
    });

    let mut content = column![header_card, toggle_button, desktop_button].spacing(14);

    if cube_active {
        content = content.push(dynamics_card);
    }

    content.spacing(14).into()
}
