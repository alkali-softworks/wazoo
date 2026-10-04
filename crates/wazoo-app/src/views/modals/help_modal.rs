/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Keyboard Shortcut Help Modal Component
 *
 * Categorized keyboard shortcuts display with visual keycaps.
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, Stack, button, column, container, row, scrollable, text, text_input},
};
use wazoo_core::{HelpCategory, HelpShortcut, KeyDisplay};

impl WazooApp {
    pub fn view_help_modal(&self) -> Element<'_, Message> {
        let or_text = self.t("common.or");
        let categories = self.settings.keybinds.help_categories(|k| self.t(k));
        let query = self.modals.help_search.trim().to_lowercase();
        let search_is_empty = query.is_empty();

        // Filter categories according to the search query
        let mut matching_categories: Vec<HelpCategory> = Vec::new();
        for cat in &categories {
            let filtered_shortcuts: Vec<HelpShortcut> = cat
                .shortcuts
                .iter()
                .filter(|item| shortcut_matches(item, &cat.title, &query, &or_text))
                .cloned()
                .collect();

            if !filtered_shortcuts.is_empty() {
                matching_categories.push(HelpCategory {
                    title: cat.title.clone(),
                    icon: cat.icon,
                    shortcuts: filtered_shortcuts,
                });
            }
        }

        let content_el: Element<'_, Message> = if matching_categories.is_empty() {
            container(
                column![
                    text("🔍").size(26),
                    text(format!(
                        "No shortcuts matching \"{}\"",
                        self.modals.help_search.trim()
                    ))
                    .size(13)
                    .color(theme::COLOR_TEXT_MUTED),
                    button(text(self.t("search.clear_search")).size(12))
                        .style(theme::folder_chip_style(false))
                        .on_press(Message::ClearHelpSearch)
                        .padding([4, 12]),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fixed(300.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if matching_categories.len() == 1 {
            // When only a single category matches, display it full width
            column![render_category_card(&matching_categories[0], &or_text)]
                .width(Length::Fill)
                .into()
        } else {
            let mut left_col = column![].spacing(12).width(Length::FillPortion(1));
            let mut right_col = column![].spacing(12).width(Length::FillPortion(1));

            if search_is_empty {
                // Default systemic 2-column layout (Playback + Layout on Left; Drawers + Audio + System on Right)
                for (idx, cat) in categories.iter().enumerate() {
                    if idx == 0 || idx == 2 {
                        left_col = left_col.push(render_category_card(cat, &or_text));
                    } else {
                        right_col = right_col.push(render_category_card(cat, &or_text));
                    }
                }
            } else {
                // Filtered results: distribute evenly across columns
                for (idx, cat) in matching_categories.iter().enumerate() {
                    if idx % 2 == 0 {
                        left_col = left_col.push(render_category_card(cat, &or_text));
                    } else {
                        right_col = right_col.push(render_category_card(cat, &or_text));
                    }
                }
            }

            row![left_col, right_col].spacing(12).into()
        };

        let scrollable_content = container(content_el)
            .padding(iced::Padding {
                top: 0.0,
                right: 16.0,
                bottom: 8.0,
                left: 2.0,
            })
            .width(Length::Fill);

        // Compact search input in header
        let search_has_text = !self.modals.help_search.is_empty();
        let search_input = text_input(&self.t("search.placeholder"), &self.modals.help_search)
            .id("help_search_input")
            .on_input(Message::HelpSearchChanged)
            .style(theme::dark_input_style)
            .size(12)
            .padding(iced::Padding {
                top: 5.0,
                right: if search_has_text { 24.0 } else { 8.0 },
                bottom: 5.0,
                left: 8.0,
            })
            .width(Length::Fixed(180.0));

        let mut search_stack_children: Vec<Element<'_, Message>> = vec![search_input.into()];

        if search_has_text {
            let clear_btn = button(text("✕").size(9))
                .style(theme::search_clear_button_style)
                .on_press(Message::ClearHelpSearch)
                .padding([1, 4]);

            let btn_container = container(clear_btn)
                .width(Length::Fixed(180.0))
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(Alignment::Center)
                .padding(iced::Padding {
                    right: 4.0,
                    ..Default::default()
                });

            search_stack_children.push(btn_container.into());
        }

        let search_bar = Stack::with_children(search_stack_children);

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
            row![
                search_bar,
                Space::new().width(Length::Fixed(8.0)),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::CloseHelpModal),
            ]
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center);

        let card = container(
            column![
                header_row,
                scrollable(scrollable_content)
                    .height(Length::Fixed(500.0))
                    .width(Length::Fill),
            ]
            .spacing(14)
            .width(Length::Fixed(680.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseHelpModal)
    }
}

fn shortcut_matches(
    item: &HelpShortcut,
    cat_title: &str,
    query: &str,
    or_text: &str,
) -> bool {
    if query.is_empty() {
        return true;
    }

    // 1. Description contains query (e.g. "mute", "volume", "fullscreen")
    if item.description.to_lowercase().contains(query) {
        return true;
    }

    // 2. Exact or substring key match
    match &item.key {
        KeyDisplay::Single(k) => {
            if k.to_lowercase() == query || k.to_lowercase().contains(query) {
                return true;
            }
        }
        KeyDisplay::Pair(k1, k2) => {
            if k1.to_lowercase() == query
                || k2.to_lowercase() == query
                || format!("{} {}", k1, k2).to_lowercase().contains(query)
            {
                return true;
            }
        }
        KeyDisplay::Alternatives(keys) => {
            if keys
                .iter()
                .any(|k| k.to_lowercase() == query || k.to_lowercase().contains(query))
            {
                return true;
            }
        }
        KeyDisplay::Combo(keys) => {
            if keys
                .iter()
                .any(|k| k.to_lowercase() == query || k.to_lowercase().contains(query))
                || item
                    .key
                    .to_display_string(or_text)
                    .to_lowercase()
                    .contains(query)
            {
                return true;
            }
        }
    }

    // 3. Category title match (for queries >= 3 chars to avoid single-letter false positives)
    if query.len() >= 3 && cat_title.to_lowercase().contains(query) {
        return true;
    }

    false
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
