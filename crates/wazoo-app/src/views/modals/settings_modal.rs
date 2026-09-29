/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal Component
 *
 * Application settings covering General, Playback, and System tabs.
 */

use crate::app::{SettingsTab, WazooApp};
use crate::cursor;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{
        Space, button, column, container, pick_list, row, scrollable, slider, text,
    },
};
use wazoo_core::Language;
use wazoo_scanner::ScanStage;

impl WazooApp {
    pub fn view_settings_modal(&self) -> Element<'_, Message> {
        let is_general = self.settings_tab == SettingsTab::General;
        let is_playback = self.settings_tab == SettingsTab::Playback;
        let is_system = self.settings_tab == SettingsTab::System;

        let badge_with_reset =
            |val_text: String, is_active: bool, reset_msg: Message| -> Element<'static, Message> {
                let badge_box = container(
                    text(val_text)
                        .size(12)
                        .font(if is_active {
                            theme::FONT_BOLD
                        } else {
                            Default::default()
                        })
                        .color(if is_active {
                            theme::COLOR_PRIMARY
                        } else {
                            theme::COLOR_TEXT_DIM
                        }),
                )
                .padding([2, 8])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: if is_active {
                            theme::COLOR_PRIMARY_BORDER
                        } else {
                            theme::COLOR_BORDER
                        },
                    },
                    ..Default::default()
                });

                if is_active {
                    let reset_btn = cursor::PointerCursor::new(
                        button(text("↺").size(11))
                            .style(theme::settings_reset_button_style)
                            .on_press(reset_msg)
                            .padding([2, 5]),
                    );
                    row![badge_box, reset_btn]
                        .spacing(5)
                        .align_y(Alignment::Center)
                        .into()
                } else {
                    row![badge_box, Space::new().width(Length::Fixed(20.0))]
                        .spacing(5)
                        .align_y(Alignment::Center)
                        .into()
                }
            };

        let tabs_bar = row![
            button(
                row![
                    text("⚙").size(14),
                    text(self.t("settings.tab_general")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_general))
            .on_press(Message::SetSettingsTab(SettingsTab::General))
            .padding([7, 16]),
            button(
                row![
                    text("▶").size(12),
                    text(self.t("settings.tab_playback")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_playback))
            .on_press(Message::SetSettingsTab(SettingsTab::Playback))
            .padding([7, 16]),
            button(
                row![
                    text("💻").size(14),
                    text(self.t("settings.tab_system")).size(13),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::settings_tab_button_style(is_system))
            .on_press(Message::SetSettingsTab(SettingsTab::System))
            .padding([7, 16]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let header = row![
            text(self.t("settings.title"))
                .size(20)
                .color(iced::Color::WHITE),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseSettingsModal),
        ]
        .align_y(Alignment::Center);

        let active_tab_content: Element<'_, Message> = match self.settings_tab {
            SettingsTab::General => {
                let mut folders_col = column![
                    text(self.t("settings.media_folders"))
                        .size(14)
                        .color(theme::COLOR_TEXT_MUTED)
                ]
                .spacing(6);
                for folder in &self.settings.media_folders {
                    let f = folder.clone();
                    folders_col = folders_col.push(
                        container(
                            row![
                                text(folder).size(13).color(iced::Color::WHITE),
                                Space::new().width(Length::Fill),
                                button(text("✕").size(12))
                                    .style(theme::close_window_button_style)
                                    .on_press(Message::RemoveMediaFolder(f)),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .padding([6, 10])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                }

                let misc_count = self.db.get_misc_video_count().unwrap_or(0);
                if misc_count > 0 {
                    folders_col = folders_col.push(
                        container(
                            row![
                                text(self.t("common.miscellaneous"))
                                    .size(13)
                                    .color(iced::Color::WHITE),
                                Space::new().width(Length::Fixed(8.0)),
                                text(format!(
                                    "({} {})",
                                    format::format_number(misc_count),
                                    self.t("common.files").to_lowercase()
                                ))
                                .size(12)
                                .color(theme::COLOR_TEXT_MUTED),
                                Space::new().width(Length::Fill),
                                button(text("✕").size(12))
                                    .style(theme::close_window_button_style)
                                    .on_press(Message::ClearMiscVideos),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .padding([6, 10])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                }

                let scan_btn_text = if self.is_scanning {
                    if let Some(ref progress) = self.scan_progress {
                        match progress.stage {
                            ScanStage::Listing => {
                                if progress.percent > 0 {
                                    let pct = progress.percent.to_string();
                                    let found = format::format_number(progress.files_found);
                                    self.t_with(
                                        "settings.scanning_progress",
                                        &[("name", "..."), ("percent", &pct), ("found", &found)],
                                    )
                                } else {
                                    self.t("wazoo.listing_files")
                                }
                            }
                            ScanStage::Indexing => {
                                if progress.percent > 0 {
                                    let pct = progress.percent.to_string();
                                    let total = format::format_number(progress.total);
                                    self.t_with(
                                        "wazoo.loading_progress",
                                        &[("percent", &pct), ("total", &total)],
                                    )
                                } else {
                                    self.t("wazoo.loading_files")
                                }
                            }
                        }
                    } else {
                        self.t("wazoo.listing_files")
                    }
                } else {
                    self.t("settings.scan_folders")
                };

                let scan_btn = if self.is_scanning {
                    button(text(scan_btn_text))
                        .style(theme::action_button_style)
                        .padding([8, 14])
                } else {
                    button(text(scan_btn_text))
                        .style(theme::action_button_style)
                        .on_press(Message::StartScan)
                        .padding([8, 14])
                };

                let mut scan_controls = column![
                    row![
                        button(text(self.t("settings.add_folder")))
                            .style(theme::action_button_style)
                            .on_press(Message::PickFolders)
                            .padding([8, 14]),
                        scan_btn,
                    ]
                    .spacing(10),
                ]
                .spacing(6);

                if self.is_scanning {
                    if let Some(ref progress) = self.scan_progress {
                        if !progress.current_name.is_empty() {
                            scan_controls = scan_controls.push(
                                text(&progress.current_name)
                                    .size(12)
                                    .color(theme::COLOR_PRIMARY),
                            );
                        }
                    }
                }

                // Language dropdown selector (16 supported languages matching wazoo-desktop)
                let current_lang = Language::from_code(&self.settings.language);
                let language_dropdown = pick_list(Language::ALL, Some(current_lang), |lang| {
                    Message::SetLanguage(lang.code.to_string())
                })
                .style(theme::dark_pick_list_style)
                .menu_style(theme::dark_pick_list_menu_style)
                .width(Length::Fill)
                .padding([8, 12]);

                let language_group = column![
                    text(self.t("settings.language"))
                        .size(14)
                        .color(theme::COLOR_TEXT_MUTED),
                    language_dropdown,
                ]
                .spacing(8);

                column![
                    language_group,
                    folders_col,
                    scan_controls,
                    text(self.t_with(
                        "settings.total_videos",
                        &[("count", &format::format_number(self.total_video_count()))]
                    ))
                    .size(13)
                    .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(16)
                .into()
            }
            SettingsTab::Playback => {
                let gamma_val = self.settings.gamma.round() as i32;
                let gamma_str = if gamma_val > 0 {
                    format!("+{gamma_val}")
                } else {
                    format!("{gamma_val}")
                };
                let gamma_group = column![
                    row![
                        text(self.t("settings.gamma"))
                            .size(14)
                            .color(theme::COLOR_TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        badge_with_reset(gamma_str, gamma_val != 0, Message::SetGamma(0.0)),
                    ]
                    .align_y(Alignment::Center),
                    cursor::PointerCursor::new(
                        slider(-100.0..=100.0, self.settings.gamma, Message::SetGamma)
                            .step(1.0_f32)
                            .style(theme::settings_slider_style)
                            .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let contrast_val = self.settings.contrast.round() as i32;
                let contrast_str = if contrast_val > 0 {
                    format!("+{contrast_val}")
                } else {
                    format!("{contrast_val}")
                };
                let contrast_group = column![
                    row![
                        text(self.t("settings.contrast"))
                            .size(14)
                            .color(theme::COLOR_TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        badge_with_reset(
                            contrast_str,
                            contrast_val != 0,
                            Message::SetContrast(0.0)
                        ),
                    ]
                    .align_y(Alignment::Center),
                    cursor::PointerCursor::new(
                        slider(-100.0..=100.0, self.settings.contrast, Message::SetContrast)
                            .step(1.0_f32)
                            .style(theme::settings_slider_style)
                            .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let brightness_val = self.settings.brightness.round() as i32;
                let brightness_str = if brightness_val > 0 {
                    format!("+{brightness_val}")
                } else {
                    format!("{brightness_val}")
                };
                let brightness_group = column![
                    row![
                        text(self.t("settings.brightness"))
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
                            self.settings.brightness,
                            Message::SetBrightness
                        )
                        .step(1.0_f32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let saturation_val = self.settings.saturation.round() as i32;
                let saturation_str = if saturation_val > 0 {
                    format!("+{saturation_val}")
                } else {
                    format!("{saturation_val}")
                };
                let saturation_group = column![
                    row![
                        text(self.t("settings.saturation"))
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
                            self.settings.saturation,
                            Message::SetSaturation
                        )
                        .step(1.0_f32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let speed_str = format!("{:.2}x", self.settings.playback_speed);
                let speed_active = (self.settings.playback_speed - 1.0).abs() > 0.01;
                let speed_group = column![
                    row![
                        text(self.t("settings.playback_speed"))
                            .size(14)
                            .color(theme::COLOR_TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        badge_with_reset(speed_str, speed_active, Message::SetPlaybackSpeed(1.0)),
                    ]
                    .align_y(Alignment::Center),
                    cursor::PointerCursor::new(
                        slider(
                            0.25..=3.0,
                            self.settings.playback_speed,
                            Message::SetPlaybackSpeed
                        )
                        .step(0.05_f32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let reset_btn = button(
                    row![
                        text("↺").size(14),
                        text(self.t("settings.reset_playback")).size(13),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .style(theme::action_button_style)
                .on_press(Message::ResetPlaybackOptions)
                .padding([8, 16]);

                column![
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
            SettingsTab::System => {
                let opacity_val = (self.settings.window_opacity * 100.0).round() as u32;
                let opacity_active = (self.settings.window_opacity - 1.0).abs() > 0.01;
                let opacity_group = column![
                    row![
                        text(self.t("settings.window_opacity"))
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
                            self.settings.window_opacity,
                            Message::SetWindowOpacity
                        )
                        .step(0.01_f32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let buffer_dur_str = format!("{}s", self.settings.buffer_duration_secs);
                let buffer_dur_active = self.settings.buffer_duration_secs != 10;
                let buffer_dur_group = column![
                    row![
                        text(self.t("settings.buffer_duration"))
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
                            self.settings.buffer_duration_secs,
                            Message::SetBufferDuration
                        )
                        .step(1_u32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let buffer_size_str = format!("{} MB", self.settings.buffer_size_mb);
                let buffer_size_active = self.settings.buffer_size_mb != 32;
                let buffer_size_group = column![
                    row![
                        text(self.t("settings.buffer_size"))
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
                        slider(
                            8..=512,
                            self.settings.buffer_size_mb,
                            Message::SetBufferSize
                        )
                        .step(8_u32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                let default_player_toggle = {
                    let is_default = self.settings.is_default_player;
                    let icon = if is_default { "✓ " } else { "" };
                    let label = format!("{}{}", icon, self.t("settings.default_player"));
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
                            text(self.t("settings.default_player"))
                                .size(14)
                                .color(iced::Color::WHITE),
                            text(if self.settings.is_default_player {
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
                    let is_pinned = self.settings.is_always_on_top;
                    let icon = if is_pinned { "✓ " } else { "" };
                    let label = format!("{}{}", icon, self.t("settings.always_on_top"));
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
                            text(self.t("settings.always_on_top"))
                                .size(14)
                                .color(iced::Color::WHITE),
                            text(self.t("settings.always_on_top_desc"))
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

                let flip_interval_str = format!("{}s", self.settings.flip_interval_secs);
                let flip_interval_active = self.settings.flip_interval_secs
                    != wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS;
                let flip_interval_group = column![
                    row![
                        text(self.t("settings.flip_interval"))
                            .size(14)
                            .color(theme::COLOR_TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        badge_with_reset(
                            flip_interval_str,
                            flip_interval_active,
                            Message::SetFlipInterval(
                                wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS
                            ),
                        ),
                    ]
                    .align_y(Alignment::Center),
                    cursor::PointerCursor::new(
                        slider(
                            1..=300_u32,
                            self.settings.flip_interval_secs.clamp(1, 300) as u32,
                            |secs| Message::SetFlipInterval(secs as u64),
                        )
                        .step(1_u32)
                        .style(theme::settings_slider_style)
                        .width(Length::Fill),
                    ),
                ]
                .spacing(6);

                column![
                    always_on_top_group,
                    opacity_group,
                    default_player_group,
                    flip_interval_group,
                    buffer_dur_group,
                    buffer_size_group,
                ]
                .spacing(16)
                .into()
            }
        };

        let scrollable_content = container(active_tab_content)
            .padding(iced::Padding {
                top: 2.0,
                right: 14.0,
                bottom: 8.0,
                left: 2.0,
            })
            .width(Length::Fill);

        let card = container(
            column![
                header,
                tabs_bar,
                scrollable(scrollable_content)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::default(),
                    ))
                    .height(Length::Fixed(480.0))
                    .width(Length::Fill),
            ]
            .spacing(18)
            .width(Length::Fixed(640.0)),
        )
        .padding(24)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSettingsModal)
    }
}
