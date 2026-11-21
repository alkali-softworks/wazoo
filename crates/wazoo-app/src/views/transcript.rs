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
use crate::assets::SVG_FOLDER_OPEN;
use crate::format;
use crate::message::Message;
use crate::theme;
use crate::wrap::Wrap;
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, svg, text, text_input},
};

impl WazooApp {
    pub(crate) fn view_transcript_drawer(&self) -> Element<'_, Message> {
        let pal = self.theme_palette;
        let (video_title, current_pos) = if let Some(player) = self.focused_player() {
            (
                format::format_video_title(&player.state.path),
                player.position().as_secs_f64(),
            )
        } else {
            (self.t("transcript.no_active_player"), 0.0)
        };

        let filter = self.drawers.transcript_search.trim().to_lowercase();

        // 1. Header row
        let mut header_left = row![
            text(self.t("transcript.title"))
                .size(18)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(pal.text),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if !self.drawers.transcript_cues.is_empty() {
            let badge = container(
                text(self.t_with(
                    "transcript.cues_count",
                    &[(
                        "count",
                        &format::format_number(self.drawers.transcript_cues.len()),
                    )],
                ))
                .size(11),
            )
            .padding([2, 8])
            .style(theme::transcript_count_badge_style(pal));
            header_left = header_left.push(badge);
        }

        let header = row![
            header_left,
            Space::new().width(Length::Fill),
            button(text("✕").size(14))
                .style(theme::window_control_button_style(pal))
                .on_press(Message::CloseTranscript),
        ]
        .align_y(Alignment::Center);

        // 2. Video Title Subheading
        let video_subheading = text(video_title).size(12).color(pal.text_muted);

        // 2b. Subtitle Track Selector & External Subtitle Browser
        let has_active_video = self
            .focused_player()
            .is_some_and(|p| !p.state.path.is_empty());
        let sub_tracks = self
            .focused_player()
            .map(|p| p.subtitle_tracks())
            .unwrap_or(&[]);
        let make_browse_btn = || {
            let browse_icon = svg(svg::Handle::from_memory(SVG_FOLDER_OPEN))
                .width(Length::Fixed(13.0))
                .height(Length::Fixed(13.0))
                .style(move |_, _| svg::Style {
                    color: Some(pal.primary),
                });
            button(
                row![
                    browse_icon,
                    text(self.t("transcript.browse_subtitle"))
                        .size(12)
                        .color(pal.text),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(theme::transcript_track_button_style(pal, false))
            .on_press(Message::PickSubtitleFile)
            .padding([5, 10])
        };

        let subtitle_selector: Option<Element<'_, Message>> = if sub_tracks.len() > 1 {
            let current_track_idx = if self.drawers.transcript_track_index < sub_tracks.len() {
                self.drawers.transcript_track_index
            } else {
                sub_tracks.iter().position(|t| t.is_selected).unwrap_or(0)
            };
            let active_sub_label = sub_tracks
                .get(current_track_idx)
                .map(|t| wazoo_media::format_subtitle_track_label(t, current_track_idx))
                .unwrap_or_else(|| format!("Track {}", current_track_idx + 1));

            let arrow = if self.drawers.show_transcript_menu {
                "▴"
            } else {
                "▾"
            };
            let track_btn_content = row![
                text(self.t("transcript.subtitle_track"))
                    .size(12)
                    .color(pal.text_muted),
                text(format!("{active_sub_label} {arrow}"))
                    .size(12)
                    .color(pal.text),
            ]
            .spacing(6)
            .align_y(Alignment::Center);

            let track_btn = button(track_btn_content)
                .style(theme::transcript_track_button_style(
                    pal,
                    self.drawers.show_transcript_menu,
                ))
                .on_press(Message::ToggleTranscriptSubtitleMenu)
                .padding([5, 10]);

            let controls_wrap = Wrap::with_elements(vec![
                track_btn.into(),
                make_browse_btn().into(),
            ])
            .spacing(8)
            .vertical_spacing(6)
            .align_items(Alignment::Center)
            .width(Length::Fill);

            let mut selector_col = column![controls_wrap].spacing(4);

            if self.drawers.show_transcript_menu {
                let menu_items: Vec<Element<'_, Message>> = sub_tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let is_selected = i == current_track_idx;
                        let label = wazoo_media::format_subtitle_track_label(t, i);
                        let track_id = t.id;
                        let selected_fg = theme::readable_hud_accent_on(
                            pal,
                            theme::mix_color(pal.hud_bg, pal.primary, 0.20),
                        );
                        let item_row = row![
                            text(if is_selected { "✓" } else { "" })
                                .size(13)
                                .color(if is_selected {
                                    selected_fg
                                } else {
                                    iced::Color::TRANSPARENT
                                })
                                .width(Length::Fixed(14.0)),
                            text(label).size(12).color(if is_selected {
                                selected_fg
                            } else {
                                pal.hud_text
                            }),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center);

                        let item_btn = button(item_row)
                            .style(theme::audio_menu_item_style(pal, is_selected))
                            .on_press(Message::SelectTranscriptSubtitleTrack(i, track_id))
                            .padding([6, 10])
                            .width(Length::Fill);

                        item_btn.into()
                    })
                    .collect();

                let sub_menu_card = container(column(menu_items).spacing(2).width(Length::Fill))
                    .padding(4)
                    .style(theme::audio_menu_card_style(pal));

                selector_col = selector_col.push(sub_menu_card);
            }

            Some(selector_col.into())
        } else if has_active_video {
            Some(make_browse_btn().into())
        } else {
            None
        };

        // 3. Dialogue Search Filter Input
        let search_box = text_input(
            &self.t("transcript.search_placeholder"),
            &self.drawers.transcript_search,
        )
        .on_input(Message::TranscriptSearchChanged)
        .style(theme::dark_input_style(pal))
        .padding(8);

        // 4. Content Area: Loading, Empty, or Scrollable Cues List
        let content_body: Element<'_, Message> = if self.drawers.transcript_loading {
            container(
                column![
                    text(self.t("transcript.extracting"))
                        .size(14)
                        .color(theme::readable_accent_on(pal, pal.drawer_bg)),
                    text(self.t("transcript.loading_subtitles"))
                        .size(12)
                        .color(pal.text_muted),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        } else if self.drawers.transcript_cues.is_empty() {
            let mut empty_col = column![
                text(self.t("transcript.no_subtitles_found"))
                    .size(15)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .color(pal.text),
                Space::new().height(Length::Fixed(4.0)),
                text(self.t("transcript.no_subtitles_desc"))
                    .size(12)
                    .color(pal.text_muted),
            ]
            .spacing(4)
            .align_x(Alignment::Center);

            if has_active_video {
                empty_col = empty_col
                    .push(Space::new().height(Length::Fixed(8.0)))
                    .push(make_browse_btn());
            }

            container(empty_col)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            let active_cue_idx = self
                .drawers
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

            for (orig_idx, cue) in self.drawers.transcript_cues.iter().enumerate() {
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
                .style(theme::transcript_time_badge_style(pal, is_active));

                let cue_text = text(&cue.text).size(13).color(if is_active {
                    pal.text
                } else {
                    pal.text_dim
                });

                let row_content = row![time_badge, cue_text,]
                    .spacing(10)
                    .align_y(Alignment::Center);

                let cue_btn = button(row_content)
                    .style(theme::transcript_cue_button_style(pal, is_active))
                    .on_press(Message::SeekToSubtitle(cue.start_secs))
                    .padding([6, 10])
                    .width(Length::Fill);

                cues_column = cues_column.push(cue_btn);
            }

            if matched_count == 0 {
                container(
                    text(self.t_with(
                        "transcript.no_match",
                        &[("query", &self.drawers.transcript_search)],
                    ))
                    .size(13)
                    .color(pal.text_muted),
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
                        right: 20.0,
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
            Space::new().height(Length::Fixed(14.0)),
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
            .style(theme::file_picker_drawer_style(pal))
            .into()
    }
}
