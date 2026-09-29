/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - General Tab
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, pick_list, row, text},
};
use wazoo_core::Language;
use wazoo_scanner::ScanStage;

pub fn view_general_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let mut folders_col = column![
        text(app.t("settings.media_folders"))
            .size(14)
            .color(theme::COLOR_TEXT_MUTED)
    ]
    .spacing(6);

    for folder in &app.settings.media_folders {
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

    let misc_count = app.db.get_misc_video_count().unwrap_or(0);
    if misc_count > 0 {
        folders_col = folders_col.push(
            container(
                row![
                    text(app.t("common.miscellaneous"))
                        .size(13)
                        .color(iced::Color::WHITE),
                    Space::new().width(Length::Fixed(8.0)),
                    text(format!(
                        "({} {})",
                        format::format_number(misc_count),
                        app.t("common.files").to_lowercase()
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

    let scan_btn_text = if app.scanner.is_scanning {
        if let Some(ref progress) = app.scanner.scan_progress {
            match progress.stage {
                ScanStage::Listing => {
                    if progress.percent > 0 {
                        let pct = progress.percent.to_string();
                        let found = format::format_number(progress.files_found);
                        app.t_with(
                            "settings.scanning_progress",
                            &[("name", "..."), ("percent", &pct), ("found", &found)],
                        )
                    } else {
                        app.t("wazoo.listing_files")
                    }
                }
                ScanStage::Indexing => {
                    if progress.percent > 0 {
                        let pct = progress.percent.to_string();
                        let total = format::format_number(progress.total);
                        app.t_with(
                            "wazoo.loading_progress",
                            &[("percent", &pct), ("total", &total)],
                        )
                    } else {
                        app.t("wazoo.loading_files")
                    }
                }
            }
        } else {
            app.t("wazoo.listing_files")
        }
    } else {
        app.t("settings.scan_folders")
    };

    let scan_btn = if app.scanner.is_scanning {
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
            button(text(app.t("settings.add_folder")))
                .style(theme::action_button_style)
                .on_press(Message::PickFolders)
                .padding([8, 14]),
            scan_btn,
        ]
        .spacing(10),
    ]
    .spacing(6);

    if app.scanner.is_scanning {
        if let Some(ref progress) = app.scanner.scan_progress {
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
    let current_lang = Language::from_code(&app.settings.language);
    let language_dropdown = pick_list(Language::ALL, Some(current_lang), |lang| {
        Message::SetLanguage(lang.code.to_string())
    })
    .style(theme::dark_pick_list_style)
    .menu_style(theme::dark_pick_list_menu_style)
    .width(Length::Fill)
    .padding([8, 12]);

    let language_group = column![
        text(app.t("settings.language"))
            .size(14)
            .color(theme::COLOR_TEXT_MUTED),
        language_dropdown,
    ]
    .spacing(8);

    column![
        language_group,
        folders_col,
        scan_controls,
        text(app.t_with(
            "settings.total_videos",
            &[("count", &format::format_number(app.total_video_count()))]
        ))
        .size(13)
        .color(theme::COLOR_TEXT_MUTED),
    ]
    .spacing(16)
    .into()
}
