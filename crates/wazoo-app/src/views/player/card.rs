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
    widget::{Space, Stack, column, container, mouse_area, row},
};

impl WazooApp {
    pub(crate) fn view_single_player<'a>(&self, p: &'a AppPlayer) -> Element<'a, Message> {
        self.view_player_internal(p, false)
    }

    pub(crate) fn view_scroll_player<'a>(&self, p: &'a AppPlayer) -> Element<'a, Message> {
        self.view_player_internal(p, true)
    }

    pub(crate) fn view_player_internal<'a>(
        &self,
        p: &'a AppPlayer,
        is_scroll_mode: bool,
    ) -> Element<'a, Message> {
        let player_id = p.id;
        let is_focused = self.focused_player_id() == Some(player_id);
        let is_hovered = !self.is_modal_or_menu_open() && self.hovered_player_id == Some(player_id);
        let is_loading = p.is_loading;
        let is_audio_menu_open = self.open_audio_menu_id == Some(player_id);

        let opacity = self.current_opacity();
        let video_widget = p.view_full(opacity, is_scroll_mode, self.settings.crt_enabled);
        let mut stack_children: Vec<Element<'a, Message>> = vec![video_widget];

        if is_loading {
            stack_children.push(self.view_loading_spinner());
        }

        // Overlays show when mouse is actively moving over this specific player (fades after delay),
        // or when audio menu is open, or while player is loading
        let show_overlay = (!self.is_modal_or_menu_open()
            && (is_hovered || is_audio_menu_open)
            && self.overlay.ticks > 0)
            || is_loading;
        let overlay_alpha = if is_loading || is_audio_menu_open {
            1.0
        } else {
            self.player_overlay_alpha()
        };

        if show_overlay {
            let hud = self.view_player_hud(p, is_focused, is_loading, overlay_alpha);
            stack_children.push(hud);
        } else if self.overlay.title_pill_ticks > 0 {
            let pill_alpha = (self.overlay.title_pill_ticks as f32 / 20.0).min(1.0);
            let title_pill = self.view_title_pill(&p.state.path, pill_alpha);

            let pill_column = column![
                Space::new().height(Length::Fixed(80.0)),
                row![title_pill, Space::new().width(Length::Fill)].width(Length::Fill),
                Space::new().height(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            stack_children.push(Element::from(pill_column));
        }

        let show_border = !is_scroll_mode && is_focused && self.overlay.focus_border_ticks > 0;
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

        let mut area = mouse_area(player_box)
            .on_press(Message::PlayerClicked(player_id))
            .on_enter(Message::PlayerHovered(player_id))
            .on_move(move |_| Message::PlayerHovered(player_id))
            .on_exit(Message::PlayerUnhovered(player_id));

        if self.should_hide_cursor() {
            area = area.interaction(mouse::Interaction::Hidden);
        }

        area.into()
    }
}
