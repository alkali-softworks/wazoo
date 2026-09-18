/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Subsystem
 *
 * Renders dialogs for video search with folder filters, application settings, keyboard shortcut
 * references, bookmark manager, and context menus with dismissable backdrops.
 */

use crate::app::WazooApp;
use crate::cursor;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{
        Space, button, column, container, mouse_area, pick_list, row, scrollable, slider, text,
        text_input,
    },
};
use wazoo_core::{HelpCategory, KeyDisplay, Language};
use wazoo_scanner::ScanStage;

fn estimate_chip_width(label: &str) -> f32 {
    let text_width: f32 = label
        .chars()
        .map(|c| {
            if c.is_ascii() {
                if c.is_ascii_uppercase() || matches!(c, 'm' | 'w' | 'M' | 'W' | '@' | '%') {
                    9.5
                } else if matches!(c, 'i' | 'l' | 'j' | 't' | 'f' | '!' | '.' | ':' | ';' | '\'' | ' ') {
                    4.5
                } else {
                    7.5
                }
            } else {
                13.0
            }
        })
        .sum();
    // 24px horizontal padding (12 left + 12 right) + 8px row spacing
    text_width + 32.0
}

impl WazooApp {
    pub(crate) fn view_search_modal(&self) -> Element<'_, Message> {
        let mut chip_items: Vec<(String, Element<'_, Message>)> = Vec::new();

        let all_label = self.t("common.all");
        let is_all_selected = self.is_all_search_selected();
        chip_items.push((
            all_label.clone(),
            button(text(all_label.clone()).size(13))
                .style(theme::folder_chip_style(is_all_selected))
                .on_press(Message::SelectSearchFolder(all_label))
                .padding([4, 12])
                .into(),
        ));

        for folder in &self.settings.media_folders {
            let label = format::ucwords(format::folder_basename(folder));
            let is_selected = self.selected_search_folders.contains(folder);
            chip_items.push((
                label.clone(),
                button(text(label).size(13))
                    .style(theme::folder_chip_style(is_selected))
                    .on_press(Message::ToggleSearchFolder(folder.clone()))
                    .padding([4, 12])
                    .into(),
            ));
        }

        if self.db.has_misc_videos().unwrap_or(false) {
            let is_misc_selected = self.selected_search_folders.contains(&"Misc".to_string());
            let misc_label = self.t("common.miscellaneous");
            chip_items.push((
                misc_label.clone(),
                button(text(misc_label).size(13))
                    .style(theme::folder_chip_style(is_misc_selected))
                    .on_press(Message::ToggleSearchFolder("Misc".to_string()))
                    .padding([4, 12])
                    .into(),
            ));
        }

        let max_row_width = 440.0;
        let mut folders_col = column![].spacing(8);
        let mut current_row = row![].spacing(8).align_y(Alignment::Center);
        let mut current_width = 0.0;
        let mut row_count = 0;

        for (label, chip_elem) in chip_items {
            let chip_width = estimate_chip_width(&label);
            if current_width + chip_width > max_row_width && current_width > 0.0 {
                folders_col = folders_col.push(current_row);
                current_row = row![].spacing(8).align_y(Alignment::Center);
                current_width = 0.0;
                row_count += 1;
            }
            current_row = current_row.push(chip_elem);
            current_width += chip_width;
        }

        if current_width > 0.0 {
            folders_col = folders_col.push(current_row);
            row_count += 1;
        }

        let folders_widget: Element<'_, Message> = if row_count > 4 {
            scrollable(folders_col)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default(),
                ))
                .height(Length::Fixed(135.0))
                .into()
        } else {
            folders_col.into()
        };

        let mut tags_row = row![].spacing(6).align_y(Alignment::Center);

        for (idx, tag) in self.search_tags.iter().enumerate() {
            let chip = container(
                row![
                    text(tag).size(13).color(iced::Color::WHITE),
                    button(text("✕").size(10))
                        .style(theme::tag_delete_button_style)
                        .on_press(Message::RemoveSearchTag(idx))
                        .padding([0, 2]),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([2, 6])
            .style(theme::tag_chip_style);

            tags_row = tags_row.push(chip);
        }

        let placeholder = if self.search_tags.is_empty() {
            self.t("search.placeholder")
        } else {
            String::new()
        };

        let input_widget = text_input(&placeholder, &self.search_input)
            .id("search_input")
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::PerformSearch)
            .style(theme::transparent_input_style)
            .padding([4, 6])
            .width(Length::Fill);

        tags_row = tags_row.push(input_widget);

        let tag_box = container(tags_row)
            .padding([3, 8])
            .style(theme::tag_input_box_style)
            .width(Length::Fill);

        let search_bar_row = row![
            tag_box,
            button(text("🔍").size(16))
                .style(theme::search_button_style)
                .on_press(Message::PerformSearch)
                .padding([8, 12]),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let card = container(
            column![
                row![
                    text(self.t("search.folder"))
                        .size(14)
                        .color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseSearchModal),
                ]
                .align_y(Alignment::Center),
                folders_widget,
                search_bar_row,
                text(self.t_with(
                    "settings.total_videos",
                    &[("count", &format::format_number(self.available_videos.len()))]
                ))
                .size(12)
                .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(14)
            .width(Length::Fixed(480.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSearchModal)
    }

    pub fn view_settings_modal(&self) -> Element<'_, Message> {
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
                        text(self.t("common.miscellaneous")).size(13).color(iced::Color::WHITE),
                        Space::new().width(Length::Fixed(8.0)),
                        text(format!("({} {})", format::format_number(misc_count), self.t("common.files").to_lowercase()))
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

        let opacity_val = (self.settings.window_opacity * 100.0).round() as u32;

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
                        let pct = progress.percent.to_string();
                        let total = format::format_number(progress.total);
                        self.t_with(
                            "wazoo.loading_progress",
                            &[("percent", &pct), ("total", &total)],
                        )
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

        let default_player_toggle = {
            let is_default = self.settings.is_default_player;
            let icon = if is_default { "✓ " } else { "" };
            let label = format!("{}{}", icon, self.t("settings.default_player"));
            button(
                text(label)
                    .size(11)
                    .font(if is_default {
                        theme::FONT_BOLD
                    } else {
                        Default::default()
                    }),
            )
            .style(theme::folder_chip_style(is_default))
            .on_press(Message::ToggleDefaultPlayer)
            .padding([4, 10])
        };

        let content = column![
            row![
                text(self.t("settings.title"))
                    .size(20)
                    .color(iced::Color::WHITE),
                Space::new().width(Length::Fill),
                default_player_toggle,
                Space::new().width(Length::Fixed(8.0)),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::CloseSettingsModal),
            ]
            .align_y(Alignment::Center),
            language_group,
            column![
                text(self.t("settings.window_opacity"))
                    .size(14)
                    .color(theme::COLOR_TEXT_MUTED),
                row![
                    cursor::PointerCursor::new(
                        slider(
                            0.05..=1.0,
                            self.settings.window_opacity,
                            Message::SetWindowOpacity
                        )
                        .step(0.01_f32)
                        .style(theme::volume_slider_style)
                        .width(Length::Fill),
                    ),
                    text(format!("{opacity_val}%"))
                        .size(13)
                        .color(theme::COLOR_TEXT_DIM),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(6),
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
        .width(Length::Fixed(480.0));

        let card = container(scrollable(content))
            .padding(20)
            .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSettingsModal)
    }

    pub(crate) fn view_help_modal(&self) -> Element<'_, Message> {
        let or_text = self.t("common.or");
        let categories = self.settings.keybinds.help_categories(|k| self.t(k));

        let mut left_col = column![].spacing(12).width(Length::FillPortion(1));
        let mut right_col = column![].spacing(12).width(Length::FillPortion(1));

        // Distribute categories systematically:
        // Left Column: Playback & Navigation, Grid & Multi-Player
        // Right Column: Panels & Drawers, Audio & Display, System & Window
        for (idx, cat) in categories.iter().enumerate() {
            if idx == 0 || idx == 2 {
                left_col = left_col.push(render_category_card(cat, &or_text));
            } else {
                right_col = right_col.push(render_category_card(cat, &or_text));
            }
        }

        let columns_row = row![left_col, right_col].spacing(12);

        let scrollable_content = container(columns_row)
            .padding(iced::Padding {
                top: 0.0,
                right: 16.0,
                bottom: 8.0,
                left: 2.0,
            })
            .width(Length::Fill);

        let header_row = row![
            row![
                container(text("⌨️").size(18))
                    .padding([4, 6])
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.259, 0.722, 0.514, 0.15,
                        ))),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 1.0,
                            color: iced::Color::from_rgba(0.259, 0.722, 0.514, 0.4),
                        },
                        ..Default::default()
                    }),
                column![
                    text(self.t("help.title"))
                        .size(17)
                        .font(theme::FONT_BOLD)
                        .color(iced::Color::WHITE),
                    text(self.t("help.subtitle"))
                        .size(11)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(2),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseHelpModal),
        ]
        .align_y(Alignment::Center);

        let card = container(
            column![
                header_row,
                scrollable(scrollable_content)
                    .height(Length::Fixed(480.0))
                    .width(Length::Fill),
            ]
            .spacing(14)
            .width(Length::Fixed(680.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseHelpModal)
    }

    pub(crate) fn view_bookmarks_modal(&self) -> Element<'_, Message> {
        let count = self.settings.bookmarks.len();
        let header_row = row![
            text(self.t("bookmarks.title"))
                .size(20)
                .color(iced::Color::WHITE),
            container(
                text(format::format_number(count))
                    .size(12)
                    .color(theme::COLOR_TEXT_DIM)
            )
            .padding([2, 8])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                border: iced::Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(Length::Fill),
            button(
                row![
                    text("+")
                        .size(14)
                        .color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
                    text(self.t("bookmarks.bookmark_current"))
                        .size(12)
                        .color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .style(theme::primary_button_style)
            .padding([6, 12])
            .on_press(Message::AddBookmarkFocused),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseBookmarksModal),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let content_element: Element<'_, Message> = if self.settings.bookmarks.is_empty() {
            container(
                column![
                    text(self.t("bookmarks.no_bookmarks"))
                        .size(15)
                        .color(theme::COLOR_TEXT_DIM),
                    text(self.t("bookmarks.hint"))
                        .size(13)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .padding([40, 20])
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list = column![].spacing(8);
            for (idx, b) in self.settings.bookmarks.iter().enumerate() {
                let time_str = format::format_time_str(b.position_secs);
                let bookmark_clone = b.clone();

                let mut meta_row = row![
                    container(text(time_str).size(11).color(theme::COLOR_PRIMARY))
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.0, 0.9, 0.7, 0.15
                            ))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(6)
                .align_y(Alignment::Center);

                if !b.query.is_empty() {
                    meta_row = meta_row.push(
                        container(
                            text(format!("#{}", b.query))
                                .size(11)
                                .color(theme::COLOR_TEXT_MUTED),
                        )
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                }

                let is_shuffle = b.is_shuffle;
                let mode_label = if is_shuffle {
                    self.t("bookmarks.shuffle")
                } else {
                    self.t("bookmarks.linear")
                };
                let mode_color = if is_shuffle {
                    theme::COLOR_PRIMARY
                } else {
                    theme::COLOR_TEXT_MUTED
                };
                meta_row = meta_row.push(
                    container(text(mode_label).size(11).color(mode_color))
                        .padding([2, 6])
                        .style(|_theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                );

                let info_col = column![text(&b.name).size(14).color(iced::Color::WHITE), meta_row,]
                    .spacing(4)
                    .width(Length::Fill);

                let play_btn = button(info_col)
                    .style(theme::bookmark_item_button_style)
                    .width(Length::Fill)
                    .padding([8, 12])
                    .on_press(Message::JumpToBookmark(bookmark_clone));

                let delete_btn = button(text("✕").size(12).color(theme::COLOR_TEXT_MUTED))
                    .style(theme::window_control_button_style)
                    .padding([8, 10])
                    .on_press(Message::RemoveBookmark(idx));

                let row_item = row![play_btn, delete_btn,]
                    .spacing(6)
                    .align_y(Alignment::Center);

                list = list.push(row_item);
            }

            let scrollable_content = container(list)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 0.0,
                    left: 2.0,
                })
                .width(Length::Fill);

            scrollable(scrollable_content)
                .height(Length::Fixed(400.0))
                .width(Length::Fill)
                .into()
        };

        let card = container(
            column![header_row, content_element,]
                .spacing(16)
                .width(Length::Fixed(560.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseBookmarksModal)
    }

    pub(crate) fn view_menu_modal(&self) -> Element<'_, Message> {
        let card = container(
            column![
                row![
                    text(self.t("common.menu"))
                        .size(18)
                        .color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseMenuModal),
                ]
                .align_y(Alignment::Center),
                self.view_app_menu_list([8, 12]).spacing(4),
            ]
            .spacing(12)
            .width(Length::Fixed(240.0)),
        )
        .padding(16)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseMenuModal)
    }

    pub(crate) fn wrap_modal_with_backdrop<'a>(
        card: container::Container<'a, Message>,
        on_close: Message,
    ) -> Element<'a, Message> {
        let backdrop_top = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_bottom = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_left = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close.clone());

        let backdrop_right = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close);

        let card_area = mouse_area(card).on_press(Message::ModalCardClicked);

        let center_row = row![backdrop_left, card_area, backdrop_right,]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Shrink);

        let modal_layout = column![backdrop_top, center_row, backdrop_bottom,]
            .width(Length::Fill)
            .height(Length::Fill);

        container(modal_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::modal_backdrop_style)
            .into()
    }
}

fn view_keycap(key_str: &str) -> Element<'static, Message> {
    container(
        text(key_str.to_string())
            .size(11)
            .font(theme::FONT_BOLD)
            .color(iced::Color::from_rgb(0.95, 0.95, 0.95)),
    )
    .padding([3, 7])
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.18, 0.18, 0.20,
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: iced::Color::from_rgb(0.32, 0.32, 0.36),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: iced::Vector::new(0.0, 1.5),
            blur_radius: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn render_key_display(display: &KeyDisplay, or_text: &str) -> Element<'static, Message> {
    match display {
        KeyDisplay::Single(k) => view_keycap(k),
        KeyDisplay::Alternatives(keys) => {
            let mut r = row![].spacing(5).align_y(Alignment::Center);
            for (idx, k) in keys.iter().enumerate() {
                if idx > 0 {
                    r = r.push(
                        text(or_text.to_string())
                            .size(10)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                }
                r = r.push(view_keycap(k));
            }
            r.into()
        }
        KeyDisplay::Pair(k1, k2) => row![view_keycap(k1), view_keycap(k2)]
            .spacing(4)
            .align_y(Alignment::Center)
            .into(),
        KeyDisplay::Combo(keys) => {
            let mut r = row![].spacing(4).align_y(Alignment::Center);
            for (idx, k) in keys.iter().enumerate() {
                if idx > 0 {
                    r = r.push(text("+").size(10).color(theme::COLOR_TEXT_MUTED));
                }
                r = r.push(view_keycap(k));
            }
            r.into()
        }
    }
}

fn render_category_card(cat: &HelpCategory, or_text: &str) -> Element<'static, Message> {
    let header = row![
        text(cat.icon).size(13),
        text(cat.title.clone())
            .size(12)
            .font(theme::FONT_BOLD)
            .color(theme::COLOR_PRIMARY),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut items_col = column![].spacing(6);
    for item in &cat.shortcuts {
        let key_el = render_key_display(&item.key, or_text);
        let key_container = container(key_el)
            .width(Length::Fixed(110.0))
            .align_x(iced::alignment::Horizontal::Left);

        let desc_text = text(item.description.clone())
            .size(12)
            .color(theme::COLOR_TEXT_DIM);

        items_col = items_col.push(
            row![
                key_container,
                Space::new().width(Length::Fixed(6.0)),
                desc_text
            ]
            .align_y(Alignment::Center),
        );
    }

    container(column![header, items_col].spacing(8))
        .padding([11, 14])
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.14, 0.14, 0.15, 0.7,
            ))),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: iced::Color::from_rgb(0.24, 0.24, 0.26),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
