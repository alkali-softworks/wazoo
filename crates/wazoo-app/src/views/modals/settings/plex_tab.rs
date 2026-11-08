/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - Plex Tab
 */

use super::badge::badge_with_reset;
use crate::app::WazooApp;
use crate::cursor;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, text, text_input},
};

pub fn view_plex_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let is_enabled = app.settings.plex.enabled;

    // 1. Enable / Disable integration toggle
    let enable_toggle = {
        let is_active = is_enabled;
        let badge_text = if is_active {
            app.t("settings.badge_on")
        } else {
            app.t("settings.badge_off")
        };
        cursor::PointerCursor::new(
            button(
                row![
                    column![
                        text(app.t("settings.plex_enable"))
                            .size(13)
                            .color(if is_active {
                                theme::COLOR_PRIMARY
                            } else {
                                iced::Color::WHITE
                            }),
                        text(app.t("settings.plex_enable_desc"))
                            .size(11)
                            .color(theme::COLOR_TEXT_MUTED),
                    ]
                    .spacing(2),
                    Space::new().width(Length::Fill),
                    badge_with_reset(badge_text, is_active, Message::SetPlexEnabled(!is_active)),
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
            .on_press(Message::SetPlexEnabled(!is_active))
            .padding([12, 16])
            .width(Length::Fill),
        )
    };

    // 2. Server URL input
    let url_input = column![
        text(app.t("settings.plex_server_url"))
            .size(13)
            .color(theme::COLOR_TEXT_MUTED),
        text_input(
            "http://192.168.1.100:32400",
            &app.settings.plex.server_url
        )
        .on_input(Message::PlexServerUrlChanged)
        .padding([8, 12])
        .size(13)
        .style(theme::dark_input_style),
    ]
    .spacing(6);

    // 3. Token input
    let token_input = column![
        row![
            text(app.t("settings.plex_token"))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
            Space::new().width(Length::Fill),
            text(app.t("settings.plex_token_desc"))
                .size(11)
                .color(theme::COLOR_TEXT_MUTED),
        ]
        .align_y(Alignment::Center),
        text_input(
            "X-Plex-Token",
            &app.settings.plex.token
        )
        .secure(true)
        .on_input(Message::PlexTokenChanged)
        .padding([8, 12])
        .size(13)
        .style(theme::dark_input_style),
    ]
    .spacing(6);

    // 4. Test connection button & feedback
    let test_btn = if app.plex.is_testing {
        button(text("Connecting...").size(13))
            .style(theme::action_button_style)
            .padding([8, 14])
    } else {
        button(text(app.t("settings.plex_test_connection")).size(13))
            .style(theme::action_button_style)
            .on_press(Message::TestPlexConnection)
            .padding([8, 14])
    };

    let test_status_view: Element<'a, Message> = match &app.plex.test_result {
        Some(Ok(info)) => {
            let label = if let Some(ref name) = info.name {
                format!("✓ Connected: {} (v{})", name, info.version)
            } else {
                format!("✓ Connected (v{})", info.version)
            };
            text(label).size(12).color(theme::COLOR_PRIMARY).into()
        }
        Some(Err(err)) => text(format!("✕ {}", err))
            .size(12)
            .color(iced::Color::from_rgb(0.95, 0.35, 0.35))
            .into(),
        None => Space::new().width(Length::Shrink).into(),
    };

    let test_row = row![test_btn, test_status_view]
        .spacing(12)
        .align_y(Alignment::Center);

    // 5. Sync library button & progress
    let sync_btn = if app.plex.is_syncing {
        button(text("Syncing...").size(13))
            .style(theme::action_button_style)
            .padding([8, 14])
    } else {
        button(text(app.t("settings.plex_sync_library")).size(13))
            .style(theme::action_button_style)
            .on_press(Message::StartPlexSync)
            .padding([8, 14])
    };

    let metadata_count = app.metadata_cache.len();
    let sync_status_view: Element<'a, Message> = if app.plex.is_syncing {
        if let Some(ref prog) = app.plex.sync_progress {
            let label = if prog.total_items > 0 {
                format!(
                    "{} - {}/{} ({} matched)",
                    prog.stage, prog.processed_items, prog.total_items, prog.matched_items
                )
            } else {
                prog.stage.clone()
            };
            text(label).size(12).color(theme::COLOR_PRIMARY).into()
        } else {
            text("Starting sync...")
                .size(12)
                .color(theme::COLOR_PRIMARY)
                .into()
        }
    } else if let Some(Ok(ref summary)) = app.plex.sync_result {
        text(format!(
            "✓ Synced {} items ({} matched)",
            summary.total_plex_items, summary.matched_and_updated
        ))
        .size(12)
        .color(theme::COLOR_PRIMARY)
        .into()
    } else if let Some(Err(ref err)) = app.plex.sync_result {
        text(format!("✕ {}", err))
            .size(12)
            .color(iced::Color::from_rgb(0.95, 0.35, 0.35))
            .into()
    } else if metadata_count > 0 {
        text(format!("{} videos have enriched metadata", metadata_count))
            .size(12)
            .color(theme::COLOR_TEXT_MUTED)
            .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let sync_row = row![sync_btn, sync_status_view]
        .spacing(12)
        .align_y(Alignment::Center);

    // 6. Auto-sync toggle
    let is_auto_sync = app.settings.plex.auto_sync_on_scan;
    let auto_sync_badge = if is_auto_sync {
        app.t("settings.badge_on")
    } else {
        app.t("settings.badge_off")
    };
    let auto_sync_toggle = cursor::PointerCursor::new(
        button(
            row![
                column![
                    text(app.t("settings.plex_auto_sync"))
                        .size(13)
                        .color(if is_auto_sync {
                            theme::COLOR_PRIMARY
                        } else {
                            iced::Color::WHITE
                        }),
                    text("Sync Plex metadata automatically after local folder scans")
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
                Space::new().width(Length::Fill),
                badge_with_reset(
                    auto_sync_badge,
                    is_auto_sync,
                    Message::SetPlexAutoSync(!is_auto_sync),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .style(move |_theme: &iced::Theme, status| {
            let bg = if is_auto_sync {
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
                    color: if is_auto_sync {
                        theme::COLOR_PRIMARY
                    } else {
                        iced::Color::from_rgb(0.25, 0.25, 0.28)
                    },
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            }
        })
        .on_press(Message::SetPlexAutoSync(!is_auto_sync))
        .padding([12, 16])
        .width(Length::Fill),
    );

    let content = column![
        enable_toggle,
        url_input,
        token_input,
        test_row,
        sync_row,
        auto_sync_toggle,
    ]
    .spacing(14);

    container(content).padding(10).width(Length::Fill).into()
}

