/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Card Container
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::state::AppPlayer;
use crate::theme;
use iced::{
    Element, Length, mouse,
    widget::{Space, Stack, column, container, mouse_area, responsive, row},
};

impl WazooApp {
    pub(crate) fn view_single_player<'a>(&'a self, p: &'a AppPlayer) -> Element<'a, Message> {
        self.view_player_internal(p, false)
    }

    pub(crate) fn view_scroll_player<'a>(&'a self, p: &'a AppPlayer) -> Element<'a, Message> {
        self.view_player_internal(p, true)
    }

    pub(crate) fn view_player_internal<'a>(
        &'a self,
        p: &'a AppPlayer,
        is_scroll_mode: bool,
    ) -> Element<'a, Message> {
        let player_id = p.id;
        let is_resizing = self.window.is_resizing();
        let is_focused = self.focused_player_id() == Some(player_id);
        let is_hovered = !is_resizing && !self.is_modal_or_menu_open() && self.hovered_player_id == Some(player_id);
        let is_loading = p.is_loading;
        let is_audio_menu_open = self.open_audio_menu_id == Some(player_id);

        let opacity = self.current_opacity();
        let mut stack_children: Vec<Element<'a, Message>> = Vec::new();

        let crt = self.settings.filters_enabled && self.settings.filter_crt;
        let wavy = self.settings.filters_enabled && self.settings.filter_wavy;
        let fog = self.settings.filters_enabled && self.settings.filter_fog;

        if is_loading {
            let use_tv_static = self.settings.loading_indicator
                == wazoo_core::LoadingIndicator::TvStatic
                && !self.raw_static_frames.is_empty();

            if use_tv_static {
                let static_id = 0x8000_0000_0000_0000u64 | (player_id as u64);
                let program = wazoo_media::pipeline::VideoProgram::new_full(
                    static_id,
                    std::sync::Arc::clone(&self.static_frame),
                    std::sync::Arc::clone(&self.static_frame_alive),
                    opacity,
                    true,
                    crt,
                    wavy,
                    fog,
                );
                let static_view = Element::new(wazoo_media::pipeline::video_shader(program));
                stack_children.push(static_view);
            } else {
                let loading_backdrop =
                    container(Space::new().width(Length::Fill).height(Length::Fill))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(theme::player_container_style(opacity));
                stack_children.push(Element::from(loading_backdrop));
                stack_children.push(self.view_loading_spinner());
            }
        } else {
            let video_widget = p.view_full(opacity, is_scroll_mode, crt, wavy, fog);
            stack_children.push(video_widget);
        }

        // Overlays show when mouse is actively moving over this specific player (fades after delay),
        // or when audio menu is open, or while player is loading.
        // Paused during window resize to prevent layout thrashing and widget tree churn.
        let show_overlay = !is_resizing
            && (((!self.is_modal_or_menu_open()
                && (is_hovered || is_audio_menu_open)
                && self.overlay.ticks > 0))
                || is_loading);
        let overlay_alpha = if is_loading || is_audio_menu_open {
            1.0
        } else {
            self.player_overlay_alpha()
        };

        if show_overlay {
            let hud = responsive(move |size| {
                let (video_x, video_y) =
                    compute_video_offset(size, p.aspect_ratio(), is_scroll_mode);
                self.view_player_hud(
                    p,
                    is_focused,
                    is_loading,
                    overlay_alpha,
                    video_x,
                    video_y,
                )
            });
            stack_children.push(Element::from(hud));
        } else if !is_resizing && self.overlay.title_pill_ticks > 0 {
            let pill_alpha = (self.overlay.title_pill_ticks as f32 / 20.0).min(1.0);
            let pill_view = responsive(move |size| {
                let (video_x, video_y) =
                    compute_video_offset(size, p.aspect_ratio(), is_scroll_mode);
                let title_pill = self.view_title_pill(&p.state.path, pill_alpha);

                let top_row = if video_x > 0.0 {
                    row![
                        Space::new().width(Length::Fixed(video_x)),
                        title_pill,
                        Space::new().width(Length::Fill),
                    ]
                } else {
                    row![title_pill, Space::new().width(Length::Fill)]
                }
                .width(Length::Fill);

                column![
                    Space::new().height(Length::Fixed(80.0 + video_y)),
                    top_row,
                    Space::new().height(Length::Fill),
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            });

            stack_children.push(Element::from(pill_view));
        }

        let show_border = !is_resizing && !is_scroll_mode && is_focused && self.overlay.focus_border_ticks > 0;
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

        let mut area = mouse_area(player_box);
        if !is_resizing {
            area = area
                .on_press(Message::PlayerClicked(player_id))
                .on_enter(Message::PlayerHovered(player_id))
                .on_move(move |_| Message::PlayerHovered(player_id))
                .on_exit(Message::PlayerUnhovered(player_id));

            if self.should_hide_cursor() {
                area = area.interaction(mouse::Interaction::Hidden);
            }
        }

        area.into()
    }
}

/// Calculates the (x, y) offset of the letterboxed/pillarboxed video inside the player tile.
/// When the window/tile is wider than the video, `x > 0` represents the left black bar width.
/// When the window/tile is taller than the video, `y > 0` represents the top black bar height.
pub(crate) fn compute_video_offset(
    size: iced::Size,
    aspect_ratio: Option<f32>,
    is_scroll_mode: bool,
) -> (f32, f32) {
    if is_scroll_mode || size.width <= 0.0 || size.height <= 0.0 {
        return (0.0, 0.0);
    }

    if let Some(video_aspect) = aspect_ratio.filter(|&a| a > 0.0) {
        let bounds_aspect = size.width / size.height;
        if bounds_aspect > video_aspect {
            // Pillarbox: black bars on left & right
            let video_w = size.height * video_aspect;
            let video_x = ((size.width - video_w) / 2.0).max(0.0);
            let video_x = if video_x > 1.0 { video_x } else { 0.0 };
            return (video_x, 0.0);
        } else if bounds_aspect < video_aspect {
            // Letterbox: black bars on top & bottom
            let video_h = size.width / video_aspect;
            let video_y = ((size.height - video_h) / 2.0).max(0.0);
            let video_y = if video_y > 1.0 { video_y } else { 0.0 };
            return (0.0, video_y);
        }
    }

    (0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    #[test]
    fn test_compute_video_offset_matching_aspect() {
        let size = Size::new(1920.0, 1080.0);
        let (x, y) = compute_video_offset(size, Some(16.0 / 9.0), false);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_compute_video_offset_wide_window_pillarbox() {
        // Ultrawide: 2560x1080 window with 16:9 video
        let size = Size::new(2560.0, 1080.0);
        let (x, y) = compute_video_offset(size, Some(16.0 / 9.0), false);
        // video_w = 1080 * 16/9 = 1920
        // x = (2560 - 1920) / 2 = 320
        assert!((x - 320.0).abs() < 0.1);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_compute_video_offset_tall_window_letterbox() {
        // Tall / portrait window: 1080x1920 with 16:9 video
        let size = Size::new(1080.0, 1920.0);
        let (x, y) = compute_video_offset(size, Some(16.0 / 9.0), false);
        // video_h = 1080 / (16/9) = 607.5
        // y = (1920 - 607.5) / 2 = 656.25
        assert_eq!(x, 0.0);
        assert!((y - 656.25).abs() < 0.1);
    }

    #[test]
    fn test_compute_video_offset_4_3_in_16_9_window() {
        // 4:3 video in 1920x1080 window
        let size = Size::new(1920.0, 1080.0);
        let (x, y) = compute_video_offset(size, Some(4.0 / 3.0), false);
        // video_w = 1080 * (4/3) = 1440
        // x = (1920 - 1440) / 2 = 240
        assert!((x - 240.0).abs() < 0.1);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_compute_video_offset_vertical_video_in_landscape() {
        // 9:16 phone video in 1920x1080 window
        let size = Size::new(1920.0, 1080.0);
        let (x, y) = compute_video_offset(size, Some(9.0 / 16.0), false);
        // video_w = 1080 * (9/16) = 607.5
        // x = (1920 - 607.5) / 2 = 656.25
        assert!((x - 656.25).abs() < 0.1);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_compute_video_offset_scroll_mode_disabled() {
        let size = Size::new(2560.0, 1080.0);
        let (x, y) = compute_video_offset(size, Some(16.0 / 9.0), true);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_compute_video_offset_none_aspect() {
        let size = Size::new(2560.0, 1080.0);
        let (x, y) = compute_video_offset(size, None, false);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }
}

