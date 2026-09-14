/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Interactive Subtitle Transcript Drawer
 *
 * Displays timestamped subtitle cues extracted from embedded media streams or external
 * sidecar files (.srt, .vtt, .ass, .ssa). Features real-time cue highlighting synchronized
 * with the focused video player, instant click-to-seek, and dialogue search filtering.
 */

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};

impl WazooApp {
    pub(crate) fn view_transcript_drawer(&self) -> Element<'_, Message> {
        let (video_title, current_pos) = if let Some(player) = self.focused_player() {
            (
                format::format_video_title(&player.state.path),
                player.position().as_secs_f64(),
            )
        } else {
            (self.t("transcript.no_active_player"), 0.0)
        };

        let filter = self.transcript_search.trim().to_lowercase();

        // 1. Header row
        let mut header_left = row![
            text(self.t("transcript.title"))
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(iced::Color::WHITE),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if !self.transcript_cues.is_empty() {
            let badge = container(
                text(self.t_with(
                    "transcript.cues_count",
                    &[("count", &format::format_number(self.transcript_cues.len()))],
                ))
                .size(11),
            )
            .padding([2, 8])
            .style(theme::transcript_count_badge_style);
            header_left = header_left.push(badge);
        }

        let header = row![
            header_left,
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style)
                .on_press(Message::CloseTranscript),
        ]
        .align_y(Alignment::Center);

        // 2. Video Title Subheading
        let video_subheading = text(video_title).size(12).color(theme::COLOR_TEXT_MUTED);

        // 2b. Subtitle Track Selector (if more than 1 subtitle stream available)
        let sub_tracks = self
            .focused_player()
            .map(|p| p.subtitle_tracks())
            .unwrap_or(&[]);
        let subtitle_selector: Option<Element<'_, Message>> = if sub_tracks.len() > 1 {
            let current_track_idx = if self.transcript_track_index < sub_tracks.len() {
                self.transcript_track_index
            } else {
                sub_tracks.iter().position(|t| t.is_selected).unwrap_or(0)
            };
            let active_sub_label = sub_tracks
                .get(current_track_idx)
                .map(|t| wazoo_media::format_subtitle_track_label(t, current_track_idx))
                .unwrap_or_else(|| format!("Track {}", current_track_idx + 1));

            let arrow = if self.show_transcript_menu {
                "▴"
            } else {
                "▾"
            };
            let track_btn_content = row![
                text(self.t("transcript.subtitle_track"))
                    .size(12)
                    .color(theme::COLOR_TEXT_MUTED),
                text(format!("{active_sub_label} {arrow}"))
                    .size(12)
                    .color(iced::Color::WHITE),
            ]
            .spacing(6)
            .align_y(Alignment::Center);

            let track_btn = button(track_btn_content)
                .style(theme::transcript_track_button_style(
                    self.show_transcript_menu,
                ))
                .on_press(Message::ToggleTranscriptSubtitleMenu)
                .padding([5, 10]);

            let mut selector_col = column![track_btn].spacing(4);

            if self.show_transcript_menu {
                let menu_items: Vec<Element<'_, Message>> = sub_tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let is_selected = i == current_track_idx;
                        let label = wazoo_media::format_subtitle_track_label(t, i);
                        let track_id = t.id;
                        let item_row = row![
                            text(if is_selected { "✓" } else { "" })
                                .size(13)
                                .color(if is_selected {
                                    theme::COLOR_PRIMARY
                                } else {
                                    iced::Color::TRANSPARENT
                                })
                                .width(Length::Fixed(14.0)),
                            text(label).size(12).color(if is_selected {
                                theme::COLOR_PRIMARY
                            } else {
                                iced::Color::WHITE
                            }),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center);

                        let item_btn = button(item_row)
                            .style(if is_selected {
                                theme::audio_menu_selected_item_style
                            } else {
                                theme::audio_menu_item_style
                            })
                            .on_press(Message::SelectTranscriptSubtitleTrack(i, track_id))
                            .padding([6, 10])
                            .width(Length::Fill);

                        item_btn.into()
                    })
                    .collect();

                let sub_menu_card = container(column(menu_items).spacing(2).width(Length::Fill))
                    .padding(4)
                    .style(theme::audio_menu_card_style);

                selector_col = selector_col.push(sub_menu_card);
            }

            Some(selector_col.into())
        } else {
            None
        };

        // 3. Dialogue Search Filter Input
        let search_box = text_input(
            &self.t("transcript.search_placeholder"),
            &self.transcript_search,
        )
        .on_input(Message::TranscriptSearchChanged)
        .style(theme::dark_input_style)
        .padding(8);

        // 4. Content Area: Loading, Empty, or Scrollable Cues List
        let content_body: Element<'_, Message> = if self.transcript_loading {
            container(
                column![
                    text(self.t("transcript.extracting"))
                        .size(14)
                        .color(theme::COLOR_PRIMARY),
                    text(self.t("transcript.loading_subtitles"))
                        .size(12)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if self.transcript_cues.is_empty() {
            container(
                column![
                    text(self.t("transcript.no_subtitles_found"))
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                    Space::new().height(Length::Fixed(4.0)),
                    text(self.t("transcript.no_subtitles_desc"))
                        .size(12)
                        .color(theme::COLOR_TEXT_MUTED),
                ]
                .spacing(4)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else {
            // Determine the single active cue for the current playback position.
            // If cues overlap or share boundaries, select the one with the latest start_secs <= current_pos.
            let active_cue_idx = self
                .transcript_cues
                .iter()
                .enumerate()
                .filter(|(_, cue)| current_pos >= cue.start_secs && current_pos < cue.end_secs)
                .max_by(|(_, a), (_, b)| {
                    a.start_secs
                        .partial_cmp(&b.start_secs)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx);

            let mut cues_column = column![].spacing(4);
            let mut matched_count = 0;

            for (orig_idx, cue) in self.transcript_cues.iter().enumerate() {
                if !filter.is_empty() && !cue.text.to_lowercase().contains(&filter) {
                    continue;
                }
                matched_count += 1;

                let is_active = active_cue_idx == Some(orig_idx);
                let time_str = format::format_time_str(cue.start_secs);

                let time_badge = container(text(time_str).size(11).font(iced::Font {
                    weight: if is_active {
                        iced::font::Weight::Bold
                    } else {
                        iced::font::Weight::Normal
                    },
                    ..Default::default()
                }))
                .padding([2, 6])
                .style(theme::transcript_time_badge_style(is_active));

                let cue_text = text(&cue.text).size(13).color(if is_active {
                    iced::Color::WHITE
                } else {
                    iced::Color::from_rgb(0.85, 0.85, 0.85)
                });

                let row_content = row![time_badge, cue_text,]
                    .spacing(10)
                    .align_y(Alignment::Center);

                let cue_btn = button(row_content)
                    .style(theme::transcript_cue_button_style(is_active))
                    .on_press(Message::SeekToSubtitle(cue.start_secs))
                    .padding([6, 10])
                    .width(Length::Fill);

                cues_column = cues_column.push(cue_btn);
            }

            if matched_count == 0 {
                container(
                    text(self.t_with("transcript.no_match", &[("query", &self.transcript_search)]))
                        .size(13)
                        .color(theme::COLOR_TEXT_MUTED),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                let scrollable_cues = container(cues_column)
                    .padding(iced::Padding {
                        top: 0.0,
                        right: 20.0, // Dedicated gutter so vertical scrollbar never overlaps dialogue
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .width(Length::Fill);

                scrollable(scrollable_cues)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
        };

        let mut content = column![
            Space::new().height(Length::Fixed(24.0)),
            header,
            video_subheading,
        ]
        .spacing(12);

        if let Some(selector) = subtitle_selector {
            content = content.push(selector);
        }

        let content = content.push(search_box).push(content_body).padding(16);

        container(content)
            .width(Length::Fixed(440.0))
            .height(Length::Fill)
            .style(theme::file_picker_drawer_style)
            .into()
    }
}
