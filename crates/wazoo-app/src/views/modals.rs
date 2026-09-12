use iced::{
    widget::{button, column, container, mouse_area, row, scrollable, slider, text, text_input, Space},
    Alignment, Element, Length, Theme,
};
use wazoo_core::PlaybackMode;
use wazoo_scanner::ScanStage;
use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_search_modal(&self) -> Element<'_, Message> {
        let mut folder_chips = row![
            button(text("All"))
                .style(theme::folder_chip_style(self.selected_search_folder == "All"))
                .on_press(Message::SelectSearchFolder("All".to_string()))
                .padding([4, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        for folder in &self.settings.media_folders {
            let label = folder.split(['/', '\\']).filter(|s| !s.is_empty()).last().unwrap_or(folder);
            folder_chips = folder_chips.push(
                button(text(label))
                    .style(theme::folder_chip_style(self.selected_search_folder == *folder))
                    .on_press(Message::SelectSearchFolder(folder.clone()))
                    .padding([4, 12]),
            );
        }

        let card = container(
            column![
                row![
                    text("Folder").size(14).color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseSearchModal),
                ]
                .align_y(Alignment::Center),
                scrollable(folder_chips).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
                row![
                    text_input("Search videos...", &self.search_input)
                        .id("search_input")
                        .on_input(Message::SearchInputChanged)
                        .on_submit(Message::PerformSearch)
                        .style(theme::dark_input_style)
                        .padding(10)
                        .width(Length::Fill),
                    button(text("🔍 Search"))
                        .style(theme::action_button_style)
                        .on_press(Message::PerformSearch)
                        .padding([10, 16]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(format!("Total videos: {}", self.available_videos.len()))
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

    pub(crate) fn view_settings_modal(&self) -> Element<'_, Message> {
        let mut folders_col = column![text("Media Folders").size(14).color(theme::COLOR_TEXT_MUTED)].spacing(6);
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

        let opacity_val = (self.settings.window_opacity * 100.0).round() as u32;

        let scan_btn_text = if self.is_scanning {
            if let Some(ref progress) = self.scan_progress {
                match progress.stage {
                    ScanStage::Listing => {
                        if progress.percent > 0 {
                            format!("Listing... {}% ({} found)", progress.percent, progress.files_found)
                        } else if progress.files_found > 0 {
                            format!("Listing... ({} found)", progress.files_found)
                        } else {
                            "Listing files...".to_string()
                        }
                    }
                    ScanStage::Indexing => {
                        format!("Loading... {}% ({} files)", progress.percent, progress.total)
                    }
                }
            } else {
                "Listing files...".to_string()
            }
        } else {
            "Scan Folders".to_string()
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
                button(text("Add Folder"))
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
                let info_str = match progress.stage {
                    ScanStage::Listing => {
                        if progress.current_name.is_empty() {
                            format!("Discovering files: {}% ({} found)", progress.percent, progress.files_found)
                        } else {
                            format!("Scanning {}: {}% ({} found)", progress.current_name, progress.percent, progress.files_found)
                        }
                    }
                    ScanStage::Indexing => {
                        if progress.current_name.is_empty() {
                            format!("Indexing database: {}% ({} files)", progress.percent, progress.total)
                        } else {
                            format!("Adding {}: {}%", progress.current_name, progress.percent)
                        }
                    }
                };
                scan_controls = scan_controls.push(
                    text(info_str).size(12).color(theme::COLOR_PRIMARY)
                );
            }
        }

        let content = column![
            row![
                text("Settings").size(20).color(iced::Color::WHITE),
                Space::new().width(Length::Fill),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::CloseSettingsModal),
            ]
            .align_y(Alignment::Center),
            column![
                text("Window Opacity").size(14).color(theme::COLOR_TEXT_MUTED),
                row![
                    slider(0.1..=1.0, self.settings.window_opacity, Message::SetWindowOpacity)
                        .step(0.01)
                        .style(theme::volume_slider_style)
                        .width(Length::Fill),
                    text(format!("{opacity_val}%")).size(13).color(theme::COLOR_TEXT_DIM),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(6),
            folders_col,
            scan_controls,
            text(format!("Total Videos: {}", self.available_videos.len()))
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
        let shortcuts = [
            ("Esc", "Toggle Menu / Close Modal"),
            ("?", "Toggle Help"),
            ("b", "Toggle Bookmarks"),
            ("+ / =", "Add Bookmark (or Scroll Speed +)"),
            ("-", "Remove Bookmark (or Scroll Speed -)"),
            ("r", "Random Seek on Video"),
            ("t", "Show Video Title Pill"),
            ("5", "Toggle Scroll Mode"),
            ("6", "Toggle Flip Mode"),
            ("h", "Toggle File Picker"),
            ("n", "Add Player"),
            ("x", "Remove Player"),
            ("Tab", "Focus Next Player"),
            ("l", "Toggle Layout"),
            ("c", "Toggle Subtitles"),
            ("< OR >", "Prev / Next Frame"),
            ("[space]", "Play / Pause"),
            ("↓ ↑", "Prev / Next Video"),
            ("← →", "Seek Back / Forward"),
            ("s", "Toggle Play Mode (Shuffle / Sequential)"),
            ("m", "Toggle Mute"),
            ("[ OR ]", "Adjust Volume"),
            ("j OR /", "Search Videos"),
            ("Alt + X", "Close App"),
            ("Alt + Drag", "Move Window"),
        ];

        let mut shortcuts_list = column![].spacing(8);
        for (key, desc) in shortcuts {
            let key_badge = container(text(key).size(12).color(iced::Color::WHITE))
                .padding([4, 8])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            shortcuts_list = shortcuts_list.push(
                row![
                    key_badge,
                    Space::new().width(Length::Fill),
                    text(desc).size(13).color(theme::COLOR_TEXT_DIM),
                ]
                .align_y(Alignment::Center),
            );
        }

        let scrollable_content = container(shortcuts_list)
            .padding(iced::Padding {
                top: 0.0,
                right: 28.0, // Clear gutter so scrollbar never touches or overlaps text
                bottom: 0.0,
                left: 4.0,
            })
            .width(Length::Fill);

        let card = container(
            column![
                row![
                    text("Keyboard Shortcuts").size(20).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseHelpModal),
                ]
                .align_y(Alignment::Center),
                scrollable(scrollable_content)
                    .height(Length::Fixed(420.0))
                    .width(Length::Fill),
            ]
            .spacing(14)
            .width(Length::Fixed(540.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseHelpModal)
    }

    pub(crate) fn view_bookmarks_modal(&self) -> Element<'_, Message> {
        let count = self.settings.bookmarks.len();
        let header_row = row![
            text("Bookmarks").size(20).color(iced::Color::WHITE),
            container(text(format!("{count}")).size(12).color(theme::COLOR_TEXT_DIM))
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
                    text("+").size(14).color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
                    text("Bookmark Current").size(12).color(iced::Color::from_rgb(0.06, 0.06, 0.06)),
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
                    text("No bookmarks saved yet.").size(15).color(theme::COLOR_TEXT_DIM),
                    text("Press + or = while playing a video (or click '+ Bookmark Current' above) to save the current video and timestamp.")
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
                            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.9, 0.7, 0.15))),
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
                        container(text(format!("#{}", b.query)).size(11).color(theme::COLOR_TEXT_MUTED))
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

                let mode_label = if b.is_shuffle { "Shuffle" } else { "Linear" };
                let mode_color = if b.is_shuffle { theme::COLOR_BLUE_ACTIVE } else { theme::COLOR_TEXT_MUTED };
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

                let info_col = column![
                    text(&b.name).size(14).color(iced::Color::WHITE),
                    meta_row,
                ]
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

                let row_item = row![
                    play_btn,
                    delete_btn,
                ]
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
            column![
                header_row,
                content_element,
            ]
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
                    text("Menu").size(18).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseMenuModal),
                ]
                .align_y(Alignment::Center),
                column![
                    button(text("Add Player")).style(theme::menu_item_style).on_press(Message::AddNewPlayer).padding([8, 12]).width(Length::Fill),
                    button(text("Toggle Layout")).style(theme::menu_item_style).on_press(Message::CycleLayout).padding([8, 12]).width(Length::Fill),
                    button(text(if self.settings.playback_mode == PlaybackMode::Scroll { "Disable Infinity Stream (5)" } else { "Infinity Stream (5)" })).style(theme::menu_item_style).on_press(Message::ToggleScrollMode).padding([8, 12]).width(Length::Fill),
                    button(text("Toggle Files")).style(theme::menu_item_style).on_press(Message::ToggleFilePicker).padding([8, 12]).width(Length::Fill),
                    button(text("Bookmarks (B)")).style(theme::menu_item_style).on_press(Message::ToggleBookmarksModal).padding([8, 12]).width(Length::Fill),
                    button(text("Search")).style(theme::menu_item_style).on_press(Message::OpenSearchModal).padding([8, 12]).width(Length::Fill),
                    button(text("Settings")).style(theme::menu_item_style).on_press(Message::OpenSettingsModal).padding([8, 12]).width(Length::Fill),
                    button(text("Help")).style(theme::menu_item_style).on_press(Message::OpenHelpModal).padding([8, 12]).width(Length::Fill),
                    button(text("Quit")).style(theme::menu_item_style).on_press(Message::CloseApp).padding([8, 12]).width(Length::Fill),
                ]
                .spacing(4),
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

        let center_row = row![
            backdrop_left,
            card,
            backdrop_right,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Shrink);

        let modal_layout = column![
            backdrop_top,
            center_row,
            backdrop_bottom,
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(modal_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::modal_backdrop_style)
            .into()
    }
}
