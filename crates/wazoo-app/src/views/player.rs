/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Video Player View Components
 * 
 * Renders individual video player containers, multi-player layouts (grid, row, column),
 * title overlays, loading spinners, and interactive playback control bars.
 */

use iced::{
    widget::{button, column, container, mouse_area, row, slider, svg, text, Space, Stack},
    Alignment, Element, Length, Theme,
};
use wazoo_core::{LayoutMode, PlaybackMode};
use wazoo_media::VideoHandle;
use crate::app::WazooApp;
use crate::assets::{
    SVG_PLAYER_MUTE, SVG_PLAYER_NEXT, SVG_PLAYER_PAUSE, SVG_PLAYER_PLAY, SVG_PLAYER_PREV, SVG_PLAYER_REPEAT,
    SVG_PLAYER_SHUFFLE, SVG_PLAYER_VOLUME,
};
use crate::cursor;
use crate::format;
use crate::message::Message;
use crate::scroll_view;
use crate::theme;

impl WazooApp {
    pub(crate) fn view_players(&self) -> Element<'_, Message> {
        if self.players.is_empty() {
            return container(text(self.t("wazoo.no_players")).size(18).color(theme::COLOR_TEXT_MUTED))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        if self.settings.playback_mode == PlaybackMode::Scroll {
            return self.view_scroll_stream();
        }

        match self.settings.layout {
            LayoutMode::Row => {
                let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    r = r.push(self.view_single_player(p));
                }
                r.into()
            }
            LayoutMode::Column => {
                let mut c = column![].spacing(0).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    c = c.push(self.view_single_player(p));
                }
                c.into()
            }
            LayoutMode::Grid => {
                let count = self.players.len();
                if count == 1 {
                    self.view_single_player(&self.players[0])
                } else if count == 2 {
                    row![
                        self.view_single_player(&self.players[0]),
                        self.view_single_player(&self.players[1]),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else if count == 3 {
                    // Electron 3-player special layout: top player spans full width, bottom row has 2
                    column![
                        container(self.view_single_player(&self.players[0]))
                            .width(Length::Fill)
                            .height(Length::FillPortion(1)),
                        row![
                            self.view_single_player(&self.players[1]),
                            self.view_single_player(&self.players[2]),
                        ]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::FillPortion(1)),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    let cols = if count <= 4 { 2 } else if count <= 9 { 3 } else { 4 };
                    let mut rows = column![].spacing(0).width(Length::Fill).height(Length::Fill);
                    for chunk in self.players.chunks(cols) {
                        let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                        for p in chunk {
                            r = r.push(self.view_single_player(p));
                        }
                        rows = rows.push(r);
                    }
                    rows.into()
                }
            }
        }
    }

    pub(crate) fn view_scroll_stream(&self) -> Element<'_, Message> {
        let mut stream = scroll_view::ScrollStream::new();
        let mut scroll_items: Vec<(&VideoHandle, &wazoo_media::ScrollItem)> = self
            .players
            .iter()
            .filter_map(|p| self.scroll_engine.items.get(&p.id).map(|item| (p, item)))
            .collect();
        scroll_items.sort_by(|a, b| a.1.y_pos.partial_cmp(&b.1.y_pos).unwrap_or(std::cmp::Ordering::Equal));

        for (p, item) in scroll_items {
            stream = stream.push(self.view_scroll_player(p), item.y_pos, item.height);
        }

        stream.into()
    }

    pub(crate) fn view_single_player<'a>(&self, p: &'a VideoHandle) -> Element<'a, Message> {
        self.view_player_internal(p, false)
    }

    pub(crate) fn view_scroll_player<'a>(&self, p: &'a VideoHandle) -> Element<'a, Message> {
        self.view_player_internal(p, true)
    }

    pub(crate) fn view_player_internal<'a>(&self, p: &'a VideoHandle, is_scroll_mode: bool) -> Element<'a, Message> {
        let player_id = p.id;
        let is_focused = self.focused_player_id() == Some(player_id);
        let is_hovered = !self.is_modal_or_menu_open() && self.hovered_player_id == Some(player_id);

        let pos = p.position();
        let dur = p.duration();
        let time_str = format!(
            "{} / {}",
            format::format_time_str(pos.as_secs_f64()),
            format::format_time_str(dur.as_secs_f64())
        );

        let progress_ratio = if dur.as_secs_f64() > 0.0 {
            (pos.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0) as f32
        } else {
            0.0f32
        };

        let opacity = self.current_opacity();
        let video_widget = p.view_with_fit(opacity, is_scroll_mode);
        let mut stack_children: Vec<Element<'a, Message>> = vec![video_widget];

        let is_loading = self.loading_player_ids.contains(&player_id);

        if is_loading {
            let angle = (self.spinner_ticks * 12) % 360;
            let spinner_svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" fill="none">
<circle cx="18" cy="18" r="14" stroke="rgba(255,255,255,0.15)" stroke-width="3"/>
<path d="M18 4 A14 14 0 0 1 32 18" stroke="#42b883" stroke-width="3" stroke-linecap="round" transform="rotate({} 18 18)"/>
</svg>"##,
                angle
            );

            let loading_card = container(
                column![
                    svg(svg::Handle::from_memory(spinner_svg.into_bytes()))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0)),
                    text(self.t("common.loading"))
                        .size(13)
                        .color(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                ]
                .spacing(12)
                .align_x(Alignment::Center),
            )
            .padding([16, 24])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.08, 0.08, 0.08, 0.88))),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(0.26, 0.72, 0.51, 0.4),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            });

            let loading_layer = container(loading_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

            stack_children.push(Element::from(loading_layer));
        }

        let is_audio_menu_open = self.open_audio_menu_player_id == Some(player_id);

        // Overlays show when mouse is actively moving over this specific player (fades after delay), or when audio menu is open, or while player is loading
        let show_overlay = (!self.is_modal_or_menu_open() && (is_hovered || is_audio_menu_open) && self.player_overlay_ticks > 0) || is_loading;
        let overlay_alpha = if is_loading || is_audio_menu_open {
            1.0
        } else {
            self.player_overlay_alpha()
        };

        // 1. Top-Left Title Pill (Matches Electron Player.vue)
        let formatted_title = format::format_descriptive_title(&p.state.path);
        let title_pill = container(
            text(formatted_title.clone())
                .size(22)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(theme::with_alpha(iced::Color::WHITE, overlay_alpha)),
        )
        .padding([10, 18])
        .style(theme::title_pill_style_with_alpha(overlay_alpha));

        let top_row = row![
            title_pill,
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill);

        if show_overlay {
            // 2. Bottom Progress & Control Overlay (Vue emerald green theme #42b883)
            let seek_slider = slider(
                0.0..=1.0,
                progress_ratio,
                move |ratio| Message::SeekRatio(player_id, ratio),
            )
            .step(0.001)
            .height(24.0)
            .style(theme::progress_slider_style_with_alpha(overlay_alpha))
            .width(Length::Fill);

            let time_display = theme::diffuse_shadowed_text(time_str, 14, overlay_alpha);

            let progress_bar_with_timestamp = cursor::PointerCursor::new(
                Stack::new()
                    .push(seek_slider)
                    .push(
                        container(time_display)
                            .width(Length::Fill)
                            .height(Length::Fixed(24.0))
                            .center_x(Length::Fill)
                            .center_y(Length::Fill),
                    ),
            );

            let play_pause_icon = if p.state.is_playing {
                svg(svg::Handle::from_memory(SVG_PLAYER_PAUSE))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            } else {
                svg(svg::Handle::from_memory(SVG_PLAYER_PLAY))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            };

            let volume_icon = if p.state.is_muted {
                svg(svg::Handle::from_memory(SVG_PLAYER_MUTE))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            } else {
                svg(svg::Handle::from_memory(SVG_PLAYER_VOLUME))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            };

            let is_shuffle = self.is_player_shuffle(player_id);
            let play_mode_icon = if is_shuffle {
                svg(svg::Handle::from_memory(SVG_PLAYER_SHUFFLE))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            } else {
                svg(svg::Handle::from_memory(SVG_PLAYER_REPEAT))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
                    .opacity(overlay_alpha)
            };

            let prev_icon = svg(svg::Handle::from_memory(SVG_PLAYER_PREV))
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0))
                .opacity(overlay_alpha);

            let next_icon = svg(svg::Handle::from_memory(SVG_PLAYER_NEXT))
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0))
                .opacity(overlay_alpha);

            let mut controls_row = row![
                // Left: CC + TX + Volume Icon + Volume Slider
                button(
                    container(
                        text("CC")
                            .size(13)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                    )
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                )
                .style(theme::cc_button_style_with_alpha(self.subtitles_enabled, overlay_alpha))
                .on_press(Message::ToggleSubtitles)
                .padding([4, 8]),
                button(
                    container(
                        text("TX")
                            .size(13)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                    )
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                )
                .style(theme::transcript_button_style_with_alpha(self.show_transcript && is_focused, overlay_alpha))
                .on_press(Message::ToggleTranscriptForPlayer(player_id))
                .padding([4, 8]),
                button(volume_icon)
                    .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                    .on_press(Message::TogglePlayerMute(player_id))
                    .padding([4, 6]),
                cursor::PointerCursor::new(
                    slider(
                        0.0..=1.0,
                        p.state.volume as f32,
                        move |v| Message::SetVolume(player_id, v as f64),
                    )
                    .step(0.01)
                    .style(theme::volume_slider_style_with_alpha(overlay_alpha))
                    .width(Length::Fixed(80.0)),
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center);

            if p.state.audio_tracks.len() > 1 {
                let active_label = p.state.current_audio_track_id
                    .and_then(|cid| p.state.audio_tracks.iter().find(|t| t.id == cid))
                    .map(|t| wazoo_media::format_audio_track_label(t, 0))
                    .or_else(|| p.state.audio_tracks.first().map(|t| wazoo_media::format_audio_track_label(t, 0)))
                    .unwrap_or_else(|| "Audio".to_string());

                let arrow = if is_audio_menu_open { "▴" } else { "▾" };

                let audio_button = button(
                    container(
                        text(format!("{active_label} {arrow}")).size(12)
                    )
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                )
                .style(theme::audio_track_button_style_with_alpha(is_audio_menu_open, overlay_alpha))
                .on_press(Message::ToggleAudioMenu(player_id))
                .padding([4, 10]);

                controls_row = controls_row.push(cursor::PointerCursor::new(audio_button));
            }

            let controls_row = controls_row
                .push(Space::new().width(Length::Fill))
                // Right: Play Mode (Shuffle/Sequential) + Prev + Play/Pause + Skip Next
                .push(
                    button(play_mode_icon)
                        .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                        .on_press(Message::TogglePlayerShuffle(player_id))
                        .padding([4, 8]),
                )
                .push(
                    button(prev_icon)
                        .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                        .on_press(Message::PrevVideo(player_id))
                        .padding([4, 8]),
                )
                .push(
                    button(play_pause_icon)
                        .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                        .on_press(Message::TogglePlay(player_id))
                        .padding([4, 8]),
                )
                .push(
                    button(next_icon)
                        .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                        .on_press(Message::NextVideo(player_id))
                        .padding([4, 8]),
                );

            let mut bottom_col = column![];

            if is_audio_menu_open && p.state.audio_tracks.len() > 1 {
                let menu_items: Vec<Element<'a, Message>> = p.state.audio_tracks
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let is_selected = Some(t.id) == p.state.current_audio_track_id;
                        let label = wazoo_media::format_audio_track_label(t, i);
                        let track_id = t.id;
                        let item_row = row![
                            text(if is_selected { "✓" } else { "" })
                                .size(13)
                                .color(if is_selected { theme::COLOR_PRIMARY } else { iced::Color::TRANSPARENT })
                                .width(Length::Fixed(14.0)),
                            text(label)
                                .size(12)
                                .color(if is_selected { theme::COLOR_PRIMARY } else { iced::Color::WHITE }),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center);

                        let item_btn = button(item_row)
                            .style(if is_selected {
                                theme::audio_menu_selected_item_style
                            } else {
                                theme::audio_menu_item_style
                            })
                            .on_press(Message::SelectAudioTrack(player_id, track_id))
                            .padding([6, 10])
                            .width(Length::Fill);

                        cursor::PointerCursor::new(item_btn).into()
                    })
                    .collect();

                let audio_menu_card = container(
                    column(menu_items)
                        .spacing(2)
                        .width(Length::Fixed(200.0))
                )
                .padding(4)
                .style(theme::audio_menu_card_style);

                let menu_row = row![
                    Space::new().width(Length::Fixed(230.0)),
                    audio_menu_card,
                    Space::new().width(Length::Fill),
                ];
                bottom_col = bottom_col.push(menu_row);
            }

            bottom_col = bottom_col
                .push(controls_row)
                .push(progress_bar_with_timestamp)
                .spacing(8);

            let bottom_overlay = container(bottom_col)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 14.0,
                    bottom: 12.0,
                    left: 14.0,
                })
                .width(Length::Fill)
                .style(theme::controls_overlay_style_with_alpha(overlay_alpha));

            let overlays_column = if is_loading {
                column![
                    Space::new().height(Length::Fixed(80.0)),
                    top_row,
                    Space::new().height(Length::Fill),
                ]
            } else {
                column![
                    Space::new().height(Length::Fixed(80.0)),
                    top_row,
                    Space::new().height(Length::Fill),
                    bottom_overlay,
                ]
            }
            .width(Length::Fill)
            .height(Length::Fill);

            stack_children.push(Element::from(overlays_column));
        } else if self.title_pill_ticks > 0 {
            let pill_alpha = (self.title_pill_ticks as f32 / 20.0).min(1.0);
            let title_pill_fallback = container(
                text(formatted_title)
                    .size(22)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .color(theme::with_alpha(iced::Color::WHITE, pill_alpha)),
            )
            .padding([10, 18])
            .style(theme::title_pill_style_with_alpha(pill_alpha));

            let pill_column = column![
                Space::new().height(Length::Fixed(80.0)),
                row![title_pill_fallback, Space::new().width(Length::Fill)].width(Length::Fill),
                Space::new().height(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            stack_children.push(Element::from(pill_column));
        }

        let show_border = !is_scroll_mode && is_focused && self.focus_border_ticks > 0;
        if show_border {
            let focus_ring = container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::focus_ring_style);
            stack_children.push(Element::from(focus_ring));
        }

        let player_stack = Stack::with_children(stack_children)
            .width(Length::Fill)
            .height(Length::Fill);

        let player_box = container(player_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::player_container_style(opacity));

        mouse_area(player_box)
            .on_press(Message::PlayerClicked(player_id))
            .on_enter(Message::PlayerHovered(player_id))
            .on_move(move |_| Message::PlayerHovered(player_id))
            .on_exit(Message::PlayerUnhovered(player_id))
            .into()
    }
}
