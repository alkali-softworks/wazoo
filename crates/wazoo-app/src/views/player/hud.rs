/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Controls HUD & Overlays
 */

use crate::app::{
    PLAYER_CONTROLS_BOTTOM_PORTION, PLAYER_CONTROLS_MAX_WIDTH, PLAYER_CONTROLS_TOP_PORTION,
    WazooApp,
};
use crate::assets::{
    SVG_PLAYER_FLIP, SVG_PLAYER_MUTE, SVG_PLAYER_NEXT, SVG_PLAYER_PAUSE, SVG_PLAYER_PLAY,
    SVG_PLAYER_PREV, SVG_PLAYER_REPEAT, SVG_PLAYER_SHUFFLE, SVG_PLAYER_VOLUME,
};
use crate::cursor;
use crate::format;
use crate::message::Message;
use crate::state::AppPlayer;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme,
    widget::{Space, Stack, button, column, container, row, slider, svg, text},
};
use wazoo_core::PlaybackMode;

impl WazooApp {
    pub(crate) fn view_title_pill<'a>(&self, path: &str, alpha: f32) -> Element<'a, Message> {
        let formatted_title = format::format_descriptive_title(path);
        container(
            text(formatted_title)
                .size(22)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(theme::with_alpha(iced::Color::WHITE, alpha)),
        )
        .padding([10, 18])
        .style(theme::title_pill_style_with_alpha(alpha))
        .into()
    }

    pub(crate) fn view_loading_spinner<'a>(&self) -> Element<'a, Message> {
        let angle = self.overlay.spinner_angle() as u32;
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
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.08, 0.08, 0.08, 0.88,
            ))),
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

        container(loading_card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub(crate) fn view_player_hud<'a>(
        &self,
        p: &'a AppPlayer,
        is_focused: bool,
        is_loading: bool,
        overlay_alpha: f32,
    ) -> Element<'a, Message> {
        let player_id = p.id;
        let pos = p.position();
        let dur = p.duration();
        let loop_str = self.t("common.loop");
        let start_str = self.t("common.start");
        let end_str = self.t("common.end");
        let loop_indicator = match (p.mark_in(), p.mark_out()) {
            (Some(i), Some(o)) => format!(
                " - {} {} ➔ {}",
                loop_str,
                format::format_time_str(i.as_secs_f64()),
                format::format_time_str(o.as_secs_f64())
            ),
            (Some(i), None) => {
                format!(
                    " - {} {} ➔ {}",
                    loop_str,
                    format::format_time_str(i.as_secs_f64()),
                    end_str
                )
            }
            (None, Some(o)) => format!(
                " - {} {} ➔ {}",
                loop_str,
                start_str,
                format::format_time_str(o.as_secs_f64())
            ),
            (None, None) => String::new(),
        };
        let time_str = format!(
            "{} / {}{}",
            format::format_time_str(pos.as_secs_f64()),
            format::format_time_str(dur.as_secs_f64()),
            loop_indicator
        );

        let progress_ratio = if dur.as_secs_f64() > 0.0 {
            (pos.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0) as f32
        } else {
            0.0f32
        };

        let is_audio_menu_open = self.open_audio_menu_id == Some(player_id);

        let title_pill = self.view_title_pill(&p.state.path, overlay_alpha);
        let top_row = row![title_pill, Space::new().width(Length::Fill)].width(Length::Fill);

        let seek_slider = slider(0.0..=1.0, progress_ratio, move |ratio| {
            Message::SeekRatio(player_id, ratio)
        })
        .step(0.001_f32)
        .height(24.0)
        .style(theme::progress_slider_style_with_alpha(overlay_alpha))
        .width(Length::Fill);

        let time_display = theme::diffuse_shadowed_text(time_str, 14, overlay_alpha);

        let progress_bar_with_timestamp = cursor::PointerCursor::new(
            Stack::new().push(seek_slider).push(
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

        let is_shuffle = p.shuffle;
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
            button(
                container(text("CC").size(13).font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }))
                .center_x(Length::Shrink)
                .center_y(Length::Shrink),
            )
            .style(theme::cc_button_style_with_alpha(
                self.subtitles_enabled,
                overlay_alpha
            ))
            .on_press(Message::ToggleSubtitles)
            .padding([4, 8]),
            button(
                container(text("TX").size(13).font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }))
                .center_x(Length::Shrink)
                .center_y(Length::Shrink),
            )
            .style(theme::transcript_button_style_with_alpha(
                self.drawers.show_transcript && is_focused,
                overlay_alpha
            ))
            .on_press(Message::ToggleTranscriptForPlayer(player_id))
            .padding([4, 8]),
            button(volume_icon)
                .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                .on_press(Message::TogglePlayerMute(player_id))
                .padding([4, 6]),
            cursor::PointerCursor::new(
                slider(0.0..=1.0, p.state.volume as f32, move |v| {
                    Message::SetVolume(player_id, v as f64)
                },)
                .step(0.01_f32)
                .style(theme::volume_slider_style_with_alpha(overlay_alpha))
                .width(Length::Fixed(80.0)),
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        if p.state.audio_tracks.len() > 1 {
            let active_label = p
                .state
                .current_audio_track_id
                .and_then(|cid| p.state.audio_tracks.iter().find(|t| t.id == cid))
                .map(|t| wazoo_media::format_audio_track_label(t, 0))
                .or_else(|| {
                    p.state
                        .audio_tracks
                        .first()
                        .map(|t| wazoo_media::format_audio_track_label(t, 0))
                })
                .unwrap_or_else(|| "Audio".to_string());

            let arrow = if is_audio_menu_open { "▴" } else { "▾" };

            let audio_button = button(
                container(text(format!("{active_label} {arrow}")).size(12))
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
            )
            .style(theme::audio_track_button_style_with_alpha(
                is_audio_menu_open,
                overlay_alpha,
            ))
            .on_press(Message::ToggleAudioMenu(player_id))
            .padding([4, 10]);

            controls_row = controls_row.push(cursor::PointerCursor::new(audio_button));
        }

        let mut controls_row = controls_row.push(Space::new().width(Length::Fill));

        if self.settings.playback_mode == PlaybackMode::Flip {
            let flip_icon = svg(svg::Handle::from_memory(SVG_PLAYER_FLIP))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0))
                .opacity(overlay_alpha);

            let flip_content = row![
                flip_icon,
                text(format!("{}s", self.flip.countdown))
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            controls_row = controls_row.push(cursor::PointerCursor::new(
                button(flip_content)
                    .style(theme::player_control_button_style_with_alpha(overlay_alpha))
                    .on_press(Message::ToggleFlipMode)
                    .padding([4, 8]),
            ));
        }

        let mut controls_row = controls_row;
        if p.has_loop() {
            let loop_badge_text = match (p.mark_in(), p.mark_out()) {
                (Some(i), Some(o)) => format!(
                    "🔁 {} - {}",
                    format::format_time_str(i.as_secs_f64()),
                    format::format_time_str(o.as_secs_f64())
                ),
                (Some(i), None) => {
                    format!("🔁 In: {}", format::format_time_str(i.as_secs_f64()))
                }
                (None, Some(o)) => {
                    format!("🔁 Out: {}", format::format_time_str(o.as_secs_f64()))
                }
                (None, None) => "🔁".to_string(),
            };
            controls_row = controls_row.push(cursor::PointerCursor::new(
                button(
                    container(
                        row![
                            text(loop_badge_text).size(12).font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            }),
                            text(" ✕")
                                .size(11)
                                .color(iced::Color::from_rgb(0.95, 0.4, 0.4)),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                )
                .style(theme::audio_track_button_style_with_alpha(
                    true,
                    overlay_alpha,
                ))
                .on_press(Message::ClearLoop(player_id))
                .padding([4, 8]),
            ));
        }

        let controls_row = controls_row
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
            let track_labels: Vec<(i64, String)> = p
                .state
                .audio_tracks
                .iter()
                .enumerate()
                .map(|(i, t)| (t.id, wazoo_media::format_audio_track_label(t, i)))
                .collect();

            let max_chars = track_labels
                .iter()
                .map(|(_, label)| label.chars().count())
                .max()
                .unwrap_or(0);

            let card_width = ((max_chars as f32) * 7.5 + 60.0).clamp(150.0, 500.0);

            let menu_items: Vec<Element<'a, Message>> = track_labels
                .into_iter()
                .map(|(track_id, label)| {
                    let is_selected = Some(track_id) == p.state.current_audio_track_id;
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
                        .on_press(Message::SelectAudioTrack(player_id, track_id))
                        .padding([6, 10])
                        .width(Length::Fill);

                    cursor::PointerCursor::new(item_btn).into()
                })
                .collect();

            let audio_menu_card = container(
                column(menu_items)
                    .spacing(2)
                    .width(Length::Fixed(card_width)),
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

        let controls_overlay = container(bottom_col)
            .padding(iced::Padding {
                top: 10.0,
                right: 18.0,
                bottom: 12.0,
                left: 18.0,
            })
            .max_width(PLAYER_CONTROLS_MAX_WIDTH)
            .width(Length::Fill)
            .style(theme::controls_overlay_style_with_alpha(overlay_alpha));

        let centered_overlay = container(controls_overlay)
            .width(Length::Fill)
            .center_x(Length::Fill);

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
                Space::new().height(Length::FillPortion(PLAYER_CONTROLS_TOP_PORTION)),
                centered_overlay,
                Space::new().height(Length::FillPortion(PLAYER_CONTROLS_BOTTOM_PORTION)),
            ]
        }
        .width(Length::Fill)
        .height(Length::Fill);

        overlays_column.into()
    }
}
