/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Quick Video Picker Modal Component
 *
 * Fast modal picker displaying other videos from the current playing video's folder.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::state::modals::QuickPickerEntry;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};

/// Maximum number of item rows rendered at once in Quick Picker so large folders
/// remain smooth to open, type in, and render every frame.
const MAX_QUICK_PICKER_DISPLAY: usize = 200;

/// Filters precomputed Quick Picker entries using the same space/wildcard/anchor
/// rules as the main file picker search.
pub(crate) fn filter_quick_picker_entries<'a>(
    entries: &'a [QuickPickerEntry],
    raw_query: &str,
) -> Vec<&'a QuickPickerEntry> {
    let query = raw_query.trim().to_lowercase();
    if query.is_empty() {
        return entries.iter().collect();
    }

    let clean = query.replace(['_', '-'], " ");
    let words: Vec<&str> = clean.split_whitespace().collect();
    let is_whole_anchored = (query.starts_with('^') && !query.starts_with("^^"))
        || (query.ends_with('$') && !query.ends_with("$$"));

    entries
        .iter()
        .filter(|entry| {
            if is_whole_anchored {
                crate::views::file_picker::pattern_matches(&query, &entry.title_lower)
            } else {
                words.iter().all(|w| {
                    crate::views::file_picker::pattern_matches(w, &entry.title_lower)
                })
            }
        })
        .collect()
}

impl WazooApp {
    pub fn view_quick_picker_modal(&self) -> Element<'_, Message> {
        let pal = self.theme_palette;
        let current_path = self
            .focused_player()
            .map(|p| p.state.path.as_str())
            .unwrap_or("");

        let owned_cache;
        let cache = match self
            .modals
            .quick_picker_cache
            .as_ref()
            .filter(|c| c.source_path == current_path)
        {
            Some(c) => c,
            None => {
                owned_cache = self.build_quick_picker_cache();
                &owned_cache
            }
        };

        // 1. Header Row
        let display_folder_name = match &cache.folder {
            Some(folder) if folder == "Misc" => self.t("common.miscellaneous"),
            Some(folder) => folder.clone(),
            None => self.t("quick_picker.title"),
        };

        let filtered_videos =
            filter_quick_picker_entries(&cache.entries, &self.modals.quick_picker_search);
        let count = filtered_videos.len();

        let header_row = row![
            text("📁").size(18),
            text(display_folder_name)
                .size(18)
                .color(pal.text),
            container(
                text(format::format_number(count))
                    .size(12)
                    .color(pal.text_dim)
            )
            .padding([2, 8])
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(pal.btn_bg)),
                border: iced::Border {
                    radius: pal.radius(10.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style(pal))
                .on_press(Message::CloseQuickPickerModal),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // 2. Search Bar
        let search_is_empty = self.modals.quick_picker_search.is_empty();
        let search_input = text_input(
            &self.t("quick_picker.search_placeholder"),
            &self.modals.quick_picker_search,
        )
        .id("quick_picker_search_input")
        .on_input(Message::QuickPickerSearchChanged)
        .on_submit(Message::QuickPickerSubmit)
        .style(theme::dark_input_style(pal))
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
                .style(theme::search_clear_button_style(pal))
                .on_press(Message::ClearQuickPickerSearch)
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

        // 3. Content List
        let content_element: Element<'_, Message> = if filtered_videos.is_empty() {
            container(
                column![
                    text(self.t("wazoo.no_files_found"))
                        .size(15)
                        .color(pal.text_dim),
                ]
                .align_x(Alignment::Center),
            )
            .padding([30, 20])
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list = column![].spacing(6);
            for entry in filtered_videos.iter().take(MAX_QUICK_PICKER_DISPLAY) {
                let mut row_items = row![
                    text("▶")
                        .size(11)
                        .color(pal.primary)
                        .width(Length::Fixed(14.0)),
                    text(entry.title.clone()).size(13).color(pal.text),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                if let Some(ref b) = entry.badge {
                    row_items = row_items.push(Space::new().width(Length::Fill));
                    row_items = row_items.push(
                        container(text(b.clone()).size(10).color(pal.text_muted))
                            .padding([2, 6])
                            .style(move |_| container::Style {
                                background: Some(iced::Background::Color(theme::with_alpha(
                                    pal.text,
                                    0.08,
                                ))),
                                border: iced::Border {
                                    radius: pal.radius(4.0).into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    );
                }

                let item_btn = button(row_items)
                    .style(theme::bookmark_item_button_style(pal))
                    .padding([8, 12])
                    .width(Length::Fill)
                    .on_press(Message::PlayQuickPickerFile(entry.path.clone()));

                list = list.push(item_btn);
            }

            if count > MAX_QUICK_PICKER_DISPLAY {
                list = list.push(
                    container(
                        text(format!(
                            "{} / {}",
                            format::format_number(MAX_QUICK_PICKER_DISPLAY),
                            format::format_number(count),
                        ))
                        .size(11)
                        .color(pal.text_muted),
                    )
                    .padding([6, 0])
                    .center_x(Length::Fill),
                );
            }

            let scrollable_content = container(list)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 16.0,
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
            column![header_row, search_bar, content_element]
                .spacing(14)
                .width(Length::Fixed(580.0)),
        )
        .padding(20)
        .style(theme::modal_card_style(pal));

        Self::wrap_modal_with_backdrop(card, Message::CloseQuickPickerModal)
    }
}
