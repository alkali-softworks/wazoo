/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Interactive Subtitle Transcript Drawer
 * 
 * Displays timestamped subtitle cues extracted from embedded media streams or external
 * sidecar files (.srt, .vtt, .ass, .ssa). Features real-time cue highlighting synchronized
 * with the focused video player, instant click-to-seek, and dialogue search filtering.
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
    pub(crate) fn view_transcript_drawer(&self) -> Element<'_, Message> {
        let (video_title, current_pos) = if let Some(player) = self.focused_player() {
            (
                format::format_video_title(&player.state.path),
                player.position().as_secs_f64(),
            )
        } else {
            ("No Active Player".to_string(), 0.0)
        };

        let filter = self.transcript_search.trim().to_lowercase();

        // 1. Header row
        let mut header_left = row![
            text("Transcript")
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
                text(format!("{} cues", self.transcript_cues.len()))
                    .size(11)
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
        let video_subheading = text(video_title)
            .size(12)
            .color(theme::COLOR_TEXT_MUTED);

        // 3. Dialogue Search Filter Input
        let search_box = text_input("Search transcript dialogue...", &self.transcript_search)
            .on_input(Message::TranscriptSearchChanged)
            .style(theme::dark_input_style)
            .padding(8);

        // 4. Content Area: Loading, Empty, or Scrollable Cues List
        let content_body: Element<'_, Message> = if self.transcript_loading {
            container(
                column![
                    text("Extracting dialogue tracks...")
                        .size(14)
                        .color(theme::COLOR_PRIMARY),
                    text("Demuxing embedded subtitles & searching sidecars")
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
                    text("No Subtitles Found")
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(iced::Color::WHITE),
                    Space::new().height(Length::Fixed(4.0)),
                    text("This video has no embedded subtitle stream or sidecar (.srt, .vtt, .ass) file.")
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
            let mut cues_column = column![].spacing(4);
            let mut matched_count = 0;

            for cue in &self.transcript_cues {
                if !filter.is_empty() && !cue.text.to_lowercase().contains(&filter) {
                    continue;
                }
                matched_count += 1;

                let is_active = current_pos >= cue.start_secs && current_pos <= cue.end_secs;
                let time_str = format::format_time_str(cue.start_secs);

                let time_badge = container(
                    text(time_str)
                        .size(11)
                        .font(iced::Font {
                            weight: if is_active {
                                iced::font::Weight::Bold
                            } else {
                                iced::font::Weight::Normal
                            },
                            ..Default::default()
                        })
                )
                .padding([2, 6])
                .style(theme::transcript_time_badge_style(is_active));

                let cue_text = text(&cue.text)
                    .size(13)
                    .color(if is_active {
                        iced::Color::WHITE
                    } else {
                        iced::Color::from_rgb(0.85, 0.85, 0.85)
                    });

                let row_content = row![
                    time_badge,
                    cue_text,
                ]
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
                    text(format!("No dialogue matching \"{}\"", self.transcript_search))
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

        let content = column![
            Space::new().height(Length::Fixed(24.0)),
            header,
            video_subheading,
            search_box,
            content_body,
        ]
        .spacing(12)
        .padding(16);

        container(content)
            .width(Length::Fixed(440.0))
            .height(Length::Fill)
            .style(theme::file_picker_drawer_style)
            .into()
    }
}
