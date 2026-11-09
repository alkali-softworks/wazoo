/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Explorer Modal Component
 *
 * Quick explorer modal for pulling Plex queries: Recently Added, Continue Watching (On Deck),
 * Top Rated, Decades, Genres, and Random picks, with instant live search and resume support.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};
use wazoo_core::PlexExplorerTab;

impl WazooApp {
    pub fn view_plex_explorer_modal(&self) -> Element<'_, Message> {
        let active_tab = &self.modals.plex_explorer_tab;
        let items = self.plex_explorer_items();
        let count = items.len();

        let has_plex_token = !self.settings.plex.token.trim().is_empty();
        let is_plex_enabled = self.settings.plex.enabled;
        let is_auth_setup = has_plex_token && is_plex_enabled;

        // 1. Header Row
        let mut header_row = row![
            text(self.t("settings.tab_plex"))
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(iced::Color::WHITE),
            text("Explorer")
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Light,
                    ..Default::default()
                })
                .color(theme::COLOR_PRIMARY),
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
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        if !is_auth_setup {
            let auth_warning_pill = button(
                row![
                    text("⚠")
                        .size(12)
                        .color(iced::Color::from_rgb(0.95, 0.65, 0.2)),
                    text(if !has_plex_token {
                        self.t("settings.plex_auth_not_configured")
                    } else {
                        self.t("settings.plex_disabled")
                    })
                    .size(11)
                    .color(iced::Color::WHITE),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .style(|_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.95, 0.65, 0.2, 0.25),
                    button::Status::Pressed => iced::Color::from_rgba(0.95, 0.65, 0.2, 0.35),
                    _ => iced::Color::from_rgba(0.95, 0.65, 0.2, 0.12),
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 10.0.into(),
                        width: 1.0,
                        color: iced::Color::from_rgba(0.95, 0.65, 0.2, 0.4),
                    },
                    ..Default::default()
                }
            })
            .padding([2, 8])
            .on_press(Message::OpenPlexSettingsModal);

            header_row = header_row.push(auth_warning_pill);
        }

        header_row = header_row
            .push(Space::new().width(Length::Fill))
            .push(
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::ClosePlexExplorerModal),
            );

        // 2. Primary Preset Tabs Bar
        let tabs = [
            (
                PlexExplorerTab::All,
                "All",
            ),
            (
                PlexExplorerTab::RecentlyAdded,
                "Recently Added",
            ),
            (
                PlexExplorerTab::OnDeck,
                "On Deck",
            ),
            (
                PlexExplorerTab::TopRated,
                "Top Rated",
            ),
            (
                PlexExplorerTab::Random,
                "Surprise Me",
            ),
        ];

        let mut tab_row = row![].spacing(6).align_y(Alignment::Center);
        for (tab, label) in tabs {
            let is_active = *active_tab == tab;
            let tab_btn = button(
                text(label)
                    .size(12)
                    .font(iced::Font {
                        weight: if is_active {
                            iced::font::Weight::Bold
                        } else {
                            iced::font::Weight::Normal
                        },
                        ..Default::default()
                    }),
            )
            .style(theme::settings_tab_button_style(is_active))
            .padding([5, 10])
            .on_press(Message::PlexExplorerSelectTab(tab));

            tab_row = tab_row.push(tab_btn);
        }

        // 3. Genres Filter Row
        let top_genres = [
            "Sci-Fi",
            "Horror",
            "Action",
            "Thriller",
            "Comedy",
            "Drama",
            "Crime",
            "Animation",
        ];

        let mut genre_row = row![
            text("Genres:")
                .size(11)
                .color(theme::COLOR_TEXT_DIM),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let all_genres_active = self.modals.plex_explorer_genre.is_none();
        let all_genres_btn = button(text("All").size(11))
            .style(theme::settings_tab_button_style(all_genres_active))
            .padding([3, 7])
            .on_press(Message::PlexExplorerToggleGenre(None));
        genre_row = genre_row.push(all_genres_btn);

        for g in top_genres {
            let is_active = self.modals.plex_explorer_genre.as_deref() == Some(g);
            let btn = button(text(g).size(11))
                .style(theme::settings_tab_button_style(is_active))
                .padding([3, 7])
                .on_press(Message::PlexExplorerToggleGenre(Some(g.to_string())));
            genre_row = genre_row.push(btn);
        }

        // 4. Decades Filter Row (on the line below genres)
        let decades = [
            (1920, "pre-30s"),
            (1930, "30s"),
            (1940, "40s"),
            (1950, "50s"),
            (1960, "60s"),
            (1970, "70s"),
            (1980, "80s"),
            (1990, "90s"),
            (2000, "00s"),
            (2010, "10s"),
            (2020, "20s"),
        ];

        let mut decade_row = row![
            text("Decades:")
                .size(11)
                .color(theme::COLOR_TEXT_DIM),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let all_decades_active = self.modals.plex_explorer_decade.is_none();
        let all_decades_btn = button(text("All").size(11))
            .style(theme::settings_tab_button_style(all_decades_active))
            .padding([3, 7])
            .on_press(Message::PlexExplorerToggleDecade(None));
        decade_row = decade_row.push(all_decades_btn);

        for (dec, label) in decades {
            let is_active = self.modals.plex_explorer_decade == Some(dec);
            let btn = button(text(label).size(11))
                .style(theme::settings_tab_button_style(is_active))
                .padding([3, 7])
                .on_press(Message::PlexExplorerToggleDecade(Some(dec)));
            decade_row = decade_row.push(btn);
        }

        // 4. Live Search & Filter Bar
        let search_is_empty = self.modals.plex_explorer_search.is_empty();
        let search_input = text_input(
            "Filter movies, shows, directors, actors, or keywords...",
            &self.modals.plex_explorer_search,
        )
        .id("plex_explorer_search_input")
        .on_input(Message::PlexExplorerSearchChanged)
        .on_submit(Message::PlexExplorerSubmit)
        .style(theme::dark_input_style)
        .padding(iced::Padding {
            top: 8.0,
            right: 28.0,
            bottom: 8.0,
            left: 10.0,
        })
        .width(Length::Fill);

        let mut stack_children: Vec<Element<'_, Message>> = vec![search_input.into()];

        if !search_is_empty {
            let clear_btn = button(text("✕").size(11))
                .style(theme::search_clear_button_style)
                .on_press(Message::ClearPlexExplorerSearch)
                .padding([2, 5]);

            let btn_container = container(clear_btn)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(Alignment::Center)
                .padding(iced::Padding {
                    right: 6.0,
                    ..Default::default()
                });

            stack_children.push(btn_container.into());
        }

        let search_bar = iced::widget::Stack::with_children(stack_children);

        // 5. Content List
        let content_element: Element<'_, Message> = if items.is_empty() {
            let empty_col = if !has_plex_token {
                column![
                    text("🔒").size(28),
                    text(self.t("settings.plex_auth_not_configured"))
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                    text(self.t("settings.plex_auth_not_configured_desc"))
                        .size(12)
                        .color(theme::COLOR_TEXT_DIM),
                    Space::new().height(Length::Fixed(12.0)),
                    button(
                        row![
                            text("⚙").size(13),
                            text(self.t("settings.plex_open_settings")).size(13).font(iced::Font {
                                weight: iced::font::Weight::Medium,
                                ..Default::default()
                            }),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .style(theme::action_button_style)
                    .padding([8, 16])
                    .on_press(Message::OpenPlexSettingsModal),
                ]
                .spacing(6)
                .align_x(Alignment::Center)
            } else if !is_plex_enabled {
                column![
                    text("⚙").size(28),
                    text(self.t("settings.plex_disabled"))
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                    text(self.t("settings.plex_disabled_desc"))
                        .size(12)
                        .color(theme::COLOR_TEXT_DIM),
                    Space::new().height(Length::Fixed(12.0)),
                    button(
                        row![
                            text("⚙").size(13),
                            text(self.t("settings.plex_open_settings")).size(13).font(iced::Font {
                                weight: iced::font::Weight::Medium,
                                ..Default::default()
                            }),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .style(theme::action_button_style)
                    .padding([8, 16])
                    .on_press(Message::OpenPlexSettingsModal),
                ]
                .spacing(6)
                .align_x(Alignment::Center)
            } else if self.metadata_cache.is_empty()
                && self.modals.plex_explorer_search.trim().is_empty()
                && self.modals.plex_explorer_genre.is_none()
                && self.modals.plex_explorer_decade.is_none()
            {
                column![
                    text("🎬").size(28),
                    text(self.t("settings.plex_no_synced_items"))
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                    text(self.t("settings.plex_no_synced_items_desc"))
                        .size(12)
                        .color(theme::COLOR_TEXT_DIM),
                    Space::new().height(Length::Fixed(12.0)),
                    button(
                        row![
                            text("⚙").size(13),
                            text(self.t("settings.plex_open_settings")).size(13).font(iced::Font {
                                weight: iced::font::Weight::Medium,
                                ..Default::default()
                            }),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .style(theme::action_button_style)
                    .padding([8, 16])
                    .on_press(Message::OpenPlexSettingsModal),
                ]
                .spacing(6)
                .align_x(Alignment::Center)
            } else {
                column![
                    text("No matching items found for this query.")
                        .size(14)
                        .color(theme::COLOR_TEXT_DIM),
                ]
                .align_x(Alignment::Center)
            };

            container(empty_col)
                .padding([40, 20])
                .center_x(Length::Fill)
                .into()
        } else {
            let mut list = column![].spacing(6);
            for item in items {
                let path = item.path.clone();
                let view_offset = item.view_offset_ms;

                let mut title_left = row![
                    text("▶")
                        .size(11)
                        .color(theme::COLOR_PRIMARY)
                        .width(Length::Fixed(14.0)),
                    text(item.title)
                        .size(14)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                if let Some(sub) = item.subtitle {
                    title_left = title_left.push(
                        text(format!("— {}", sub))
                            .size(12)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                }

                // Metadata Chips: Year, Rating, Content Rating, Genres, Resume Offset
                let mut chips_row = row![].spacing(5).align_y(Alignment::Center);

                if let Some(offset_ms) = view_offset {
                    let resume_secs = offset_ms / 1000;
                    let dur_secs = item.duration_ms.map(|d| d / 1000).unwrap_or(0);
                    let resume_str = if dur_secs > 0 {
                        format!(
                            "▶ {} / {}",
                            format::format_time_str(resume_secs as f64),
                            format::format_time_str(dur_secs as f64)
                        )
                    } else {
                        format!("▶ {}", format::format_time_str(resume_secs as f64))
                    };

                    chips_row = chips_row.push(
                        container(
                            text(resume_str)
                                .size(10)
                                .font(iced::Font {
                                    weight: iced::font::Weight::Bold,
                                    ..Default::default()
                                })
                                .color(theme::COLOR_PRIMARY),
                        )
                        .padding([2, 6])
                        .style(|_| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.259, 0.722, 0.514, 0.15,
                            ))),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    );
                }

                if let Some(y) = item.year {
                    chips_row = chips_row.push(
                        container(text(y.to_string()).size(10).color(iced::Color::WHITE))
                            .padding([2, 5])
                            .style(|_| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba(
                                    1.0, 1.0, 1.0, 0.08,
                                ))),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    );
                }

                if let Some(cr) = item.content_rating {
                    if !cr.is_empty() {
                        chips_row = chips_row.push(
                            container(text(cr).size(10).color(theme::COLOR_TEXT_MUTED))
                                .padding([2, 5])
                                .style(|_| container::Style {
                                    background: Some(iced::Background::Color(
                                        iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                                    )),
                                    border: iced::Border {
                                        radius: 4.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                        );
                    }
                }

                if let Some(r) = item.rating {
                    chips_row = chips_row.push(
                        container(text(format!("★ {:.1}", r)).size(10).color(theme::COLOR_PRIMARY))
                            .padding([2, 5])
                            .style(|_| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba(
                                    0.259, 0.722, 0.514, 0.12,
                                ))),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    );
                }

                for g in item.genres.into_iter().take(2) {
                    let display_g = if g.eq_ignore_ascii_case("science fiction") {
                        "Sci-Fi".to_string()
                    } else {
                        g
                    };
                    chips_row = chips_row.push(
                        container(text(display_g).size(10).color(theme::COLOR_TEXT_DIM))
                            .padding([2, 5])
                            .style(|_| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba(
                                    1.0, 1.0, 1.0, 0.06,
                                ))),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    );
                }

                let title_row = row![
                    title_left.width(Length::Fill),
                    chips_row.align_y(Alignment::Center),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .width(Length::Fill);

                let mut card_col = column![title_row].spacing(4);

                // Brief Summary or Tagline Snippet
                let snippet_text = match (&item.tagline, &item.summary) {
                    (Some(tag), Some(sum)) if !tag.is_empty() && !sum.is_empty() => {
                        let t_trim = tag.trim();
                        let s_trim = sum.trim();
                        if s_trim.starts_with(t_trim) {
                            Some(s_trim.to_string())
                        } else {
                            Some(format!("{} — {}", t_trim, s_trim))
                        }
                    }
                    (Some(tag), _) if !tag.is_empty() => Some(tag.trim().to_string()),
                    (_, Some(sum)) if !sum.is_empty() => Some(sum.trim().to_string()),
                    _ => None,
                };

                if let Some(snip) = snippet_text {
                    let clean = snip.replace('\n', " ");
                    let short_snip: String = if clean.chars().count() > 180 {
                        clean.chars().take(177).collect::<String>() + "..."
                    } else {
                        clean
                    };

                    card_col = card_col.push(
                        text(short_snip)
                            .size(11)
                            .color(theme::COLOR_TEXT_DIM),
                    );
                }

                let item_btn = button(card_col)
                    .style(theme::bookmark_item_button_style)
                    .padding([8, 12])
                    .width(Length::Fill)
                    .on_press(Message::PlayPlexExplorerItem(path, view_offset));

                list = list.push(item_btn);
            }

            let scrollable_content = container(list)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 14.0,
                    bottom: 0.0,
                    left: 2.0,
                })
                .width(Length::Fill);

            scrollable(scrollable_content)
                .height(Length::Fixed(440.0))
                .width(Length::Fill)
                .into()
        };

        let show_filters = matches!(
            active_tab,
            PlexExplorerTab::All | PlexExplorerTab::Decade(_) | PlexExplorerTab::Genre(_)
        );

        let mut card_col = column![header_row, tab_row].spacing(10);
        if show_filters {
            card_col = card_col.push(genre_row).push(decade_row);
        }
        card_col = card_col.push(search_bar);

        if !is_auth_setup && !items.is_empty() {
            let banner = container(
                row![
                    text("⚠")
                        .size(13)
                        .color(iced::Color::from_rgb(0.95, 0.65, 0.2)),
                    text(if !has_plex_token {
                        self.t("settings.plex_auth_not_configured_desc")
                    } else {
                        self.t("settings.plex_disabled_desc")
                    })
                    .size(11)
                    .color(theme::COLOR_TEXT_DIM)
                    .width(Length::Fill),
                    button(text(self.t("settings.plex_open_settings")).size(11))
                        .style(theme::action_button_style)
                        .padding([4, 10])
                        .on_press(Message::OpenPlexSettingsModal),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding([8, 12])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.95, 0.65, 0.2, 0.08,
                ))),
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(0.95, 0.65, 0.2, 0.25),
                },
                ..Default::default()
            })
            .width(Length::Fill);

            card_col = card_col.push(banner);
        }

        card_col = card_col.push(content_element);

        let card = container(card_col.width(Length::Fixed(900.0)))
            .padding(20)
            .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::ClosePlexExplorerModal)
    }
}
