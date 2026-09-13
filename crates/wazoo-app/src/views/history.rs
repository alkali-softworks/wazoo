/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Play History Drawer
 * 
 * Displays a reverse-chronological list of videos played across all players during the session,
 * capped at 1,000 entries. Features quick search filtering, index numbering, folder tags, and
 * click-to-play directly into the focused playback tile.
 */

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};
use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_history_drawer(&self) -> Element<'_, Message> {
        let filter = self.history_search.trim().to_lowercase();

        // 1. Header row
        let mut header_left = row![
            text(self.t("history.title"))
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(iced::Color::WHITE),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if !self.play_history.is_empty() {
            let badge = container(
                text(self.t_with("history.count", &[("count", &format::format_number(self.play_history.len()))]))
                    .size(11)
            )
            .padding([2, 8])
            .style(theme::transcript_count_badge_style);
            header_left = header_left.push(badge);
        }

        let mut header_row = row![
            header_left,
            Space::new().width(Length::Fill),
        ]
        .align_y(Alignment::Center)
        .spacing(8);

        if !self.play_history.is_empty() {
            header_row = header_row.push(
                button(text(self.t("history.clear")).size(11))
                    .style(theme::tag_delete_button_style)
                    .on_press(Message::ClearPlayHistory)
                    .padding([3, 8]),
            );
        }

        header_row = header_row.push(
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseHistoryDrawer),
        );

        // 2. Search filter input
        let search_input = text_input(&self.t("history.search_placeholder"), &self.history_search)
            .on_input(Message::HistorySearchChanged)
            .style(theme::dark_input_style)
            .padding(8);

        // 3. History list (reverse-chronological: most recent at top)
        let mut list_col = column![].spacing(3);

        let filtered_items: Vec<(usize, &crate::app::PlayHistoryItem)> = self
            .play_history
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if filter.is_empty() {
                    true
                } else {
                    item.title.to_lowercase().contains(&filter)
                        || item.folder.to_lowercase().contains(&filter)
                        || item.path.to_lowercase().contains(&filter)
                }
            })
            .collect();

        if self.play_history.is_empty() {
            list_col = list_col.push(
                container(
                    text(self.t("history.empty"))
                        .size(13)
                        .color(theme::COLOR_TEXT_MUTED),
                )
                .padding([20, 0])
                .center_x(Length::Fill),
            );
        } else if filtered_items.is_empty() {
            list_col = list_col.push(
                container(
                    text(self.t_with("transcript.no_match", &[("query", &self.history_search)]))
                        .size(13)
                        .color(theme::COLOR_TEXT_MUTED),
                )
                .padding([20, 0])
                .center_x(Length::Fill),
            );
        } else {
            // Render in reverse chronological order
            for (idx, item) in filtered_items.into_iter().rev() {
                let item_number = idx + 1; // 1-based original index
                let p_clone = item.path.clone();

                let mut item_info = column![
                    text(&item.title)
                        .size(13)
                        .color(iced::Color::WHITE),
                ]
                .spacing(2)
                .width(Length::Fill);

                if !item.folder.is_empty() {
                    item_info = item_info.push(
                        text(&item.folder)
                            .size(11)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                }

                let row_content = row![
                    container(
                        text(format!("#{}", item_number))
                            .size(11)
                            .color(theme::COLOR_PRIMARY)
                    )
                    .width(Length::Fixed(36.0)),
                    item_info,
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                let item_btn = button(row_content)
                    .style(theme::menu_item_style)
                    .on_press(Message::PlayFileInFocused(p_clone))
                    .padding([6, 10])
                    .width(Length::Fill);

                list_col = list_col.push(item_btn);
            }
        }

        let scrollable_list = container(list_col)
            .padding(iced::Padding {
                top: 0.0,
                right: 16.0, // Dedicated gutter so vertical scrollbar never overlaps text
                bottom: 0.0,
                left: 0.0,
            })
            .width(Length::Fill);

        let content = column![
            Space::new().height(Length::Fixed(24.0)),
            header_row,
            search_input,
            scrollable(scrollable_list)
                .height(Length::Fill)
                .width(Length::Fill),
        ]
        .spacing(12)
        .padding(16);

        container(content)
            .width(Length::Fixed(420.0))
            .height(Length::Fill)
            .style(theme::file_picker_drawer_style)
            .into()
    }
}
