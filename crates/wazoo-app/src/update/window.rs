/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Lifecycle & Hardware Input Reducer
 */

use crate::app::{
    DEFAULT_TOAST_SECS, DROPDOWN_MENU_SLIDE_TICKS, PLAYER_OVERLAY_FADE_TICKS, TITLEBAR_FADE_TICKS,
    TITLEBAR_HIDE_TICKS, TITLEBAR_SHOW_DELAY_TICKS, TITLEBAR_SLIDE_TICKS, WazooApp,
};
use crate::message::Message;
use crate::state::AppPlayer;
use iced::{Point, Task, keyboard::Key};
use std::time::{Duration, Instant};
use wazoo_core::PlaybackMode;

impl WazooApp {
    pub(crate) fn update_window(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowIdReceived(id) => {
                self.window.id = Some(id);
                let focus_task = iced::window::gain_focus(id);
                let level_task = if self.settings.is_always_on_top {
                    iced::window::set_level(id, iced::window::Level::AlwaysOnTop)
                } else {
                    Task::none()
                };
                Task::batch([focus_task, level_task])
            }
            Message::GainWindowFocus => {
                if let Some(id) = self.window.id {
                    iced::window::gain_focus(id)
                } else {
                    iced::window::oldest().then(|maybe_id| {
                        if let Some(id) = maybe_id {
                            iced::window::gain_focus(id)
                        } else {
                            Task::none()
                        }
                    })
                }
            }
            Message::WindowFocused => {
                self.window.is_focused = true;
                self.window.unfocused_frame_ticks = 0;
                self.window.is_alt_pressed = false;
                if self.settings.is_always_on_top && self.window.ghost_passthrough_active {
                    self.window.ghost_passthrough_active = false;
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    if let Some(id) = self.window.id {
                        return iced::window::disable_mouse_passthrough(id);
                    }
                }
                Task::none()
            }
            Message::WindowUnfocused => {
                self.window.is_focused = false;
                self.window.is_alt_pressed = false;
                if !self.window.is_dragging {
                    self.window.cursor_position = Point::new(-1000.0, -1000.0);
                    self.titlebar.drag_pending = false;
                    self.titlebar.press_origin = None;
                    self.titlebar.hover_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        let current_fade =
                            self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.overlay.ticks = self.overlay.ticks.min(current_fade);
                    }
                }
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                if self.settings.is_always_on_top
                    && !self.window.ghost_passthrough_active
                    && !self.is_modal_or_menu_open()
                {
                    self.window.ghost_passthrough_active = true;
                    if let Some(id) = self.window.id {
                        return iced::window::enable_mouse_passthrough(id);
                    }
                }
                Task::none()
            }
            Message::WindowMoved(id, point) => {
                self.window.id = Some(id);
                let new_x = point.x as i32;
                let new_y = point.y as i32;
                if self.settings.window_bounds.x != new_x || self.settings.window_bounds.y != new_y
                {
                    self.settings.window_bounds.x = new_x;
                    self.settings.window_bounds.y = new_y;
                    self.window.bounds_dirty = true;
                }
                if self.window.is_dragging {
                    self.window.last_drag_move = Some(Instant::now());
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                }
                Task::none()
            }
            Message::WindowResized(id, size) => {
                self.window.id = Some(id);
                let new_w = (size.width as u32).clamp(200, 7680);
                let new_h = (size.height as u32).clamp(150, 4320);
                if self.settings.window_bounds.width != new_w
                    || self.settings.window_bounds.height != new_h
                {
                    self.settings.window_bounds.width = new_w;
                    self.settings.window_bounds.height = new_h;
                    self.window.bounds_dirty = true;
                }
                self.scroll_engine.set_window_size(size.width, size.height);
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    for p in &self.players {
                        let item_h = self.calculate_player_scroll_height(p);
                        self.scroll_engine.update_height(p.id, item_h);
                    }
                    self.scroll_engine.recalculate_positions();
                    let margin = self.scroll_engine.default_item_height() * 1.5;
                    while let Some(spawn_y) =
                        self.scroll_engine.needs_new_player_with_margin(margin)
                    {
                        if let Some(mut handle) = self.preloaded_player.take() {
                            handle.set_muted(self.settings.scroll_mode_muted);
                            handle.set_paused(false);
                            let item_h = self.calculate_player_scroll_height(&handle);
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.record_play_history(&handle.state.path);
                            self.players
                                .push(AppPlayer::new(handle, self.default_shuffle_mode));
                        } else {
                            break;
                        }
                    }
                    return self.trigger_preload_task();
                }
                Task::none()
            }
            Message::MinimizeWindow => {
                if let Some(id) = self.window.id {
                    iced::window::minimize(id, true)
                } else {
                    Task::none()
                }
            }
            Message::MaximizeWindow => {
                self.titlebar.last_click = None;
                self.titlebar.drag_pending = false;
                self.titlebar.press_origin = None;
                if let Some(id) = self.window.id {
                    iced::window::toggle_maximize(id)
                } else {
                    Task::none()
                }
            }
            Message::DragWindow => {
                if let Some(id) = self.window.id {
                    self.window.is_dragging = true;
                    self.window.last_drag_move = Some(Instant::now());
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    iced::window::drag(id)
                } else {
                    Task::none()
                }
            }
            Message::DragResize(direction) => {
                if let Some(id) = self.window.id {
                    iced::window::drag_resize(id, direction)
                } else {
                    Task::none()
                }
            }
            Message::CloseApp => {
                self.save_session_state();
                self.cleanup_scroll_mode();
                self.players.clear();
                self.preloaded_player = None;
                if let Some(id) = self.window.id {
                    let _: Task<Message> = iced::window::close(id);
                }
                #[cfg(not(test))]
                std::process::exit(0);
                #[cfg(test)]
                Task::none()
            }
            Message::ModifiersChanged(modifiers) => {
                self.window.is_alt_pressed = modifiers.alt();
                Task::none()
            }
            Message::KeyPressed(key, status) => self.handle_key_pressed(key, status),
            Message::KeyReleased(key) => {
                if key == Key::Named(iced::keyboard::key::Named::Alt)
                    || key == Key::Named(iced::keyboard::key::Named::AltGraph)
                {
                    self.window.is_alt_pressed = false;
                }
                Task::none()
            }
            Message::CursorMoved(win_id, pos) => self.handle_cursor_moved(win_id, pos),
            Message::CursorLeft => {
                if !self.window.is_dragging && !self.titlebar.drag_pending {
                    self.window.cursor_position = Point::new(-1000.0, -1000.0);
                    self.titlebar.hover_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        let current_fade =
                            self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.overlay.ticks = self.overlay.ticks.min(current_fade);
                    }
                }
                Task::none()
            }
            Message::TitleBarPressed => {
                let now = Instant::now();
                if let Some(last_click) = self.titlebar.last_click {
                    if now.duration_since(last_click) < Duration::from_millis(400) {
                        self.titlebar.last_click = None;
                        self.titlebar.drag_pending = false;
                        self.window.is_dragging = false;
                        self.titlebar.press_origin = None;
                        return self.update(Message::MaximizeWindow);
                    }
                }
                self.titlebar.last_click = Some(now);
                self.titlebar.drag_pending = true;
                self.titlebar.press_origin = Some(self.window.cursor_position);
                self.titlebar.show = true;
                self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                Task::none()
            }
            Message::LeftClickReleased => {
                let was_dragging = self.window.is_dragging;
                self.window.is_dragging = false;
                self.titlebar.drag_pending = false;
                self.titlebar.press_origin = None;
                self.window.last_drag_move = None;
                if was_dragging {
                    if self.is_point_in_titlebar(self.window.cursor_position) {
                        self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    } else {
                        self.titlebar.hide_ticks = TITLEBAR_FADE_TICKS;
                    }
                } else if !self.is_point_in_titlebar(self.window.cursor_position) {
                    self.titlebar.hide_ticks = self.titlebar.hide_ticks.min(TITLEBAR_FADE_TICKS);
                }
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                Task::none()
            }
            Message::RightClickPressed(win_id) => {
                self.window.id = Some(win_id);
                self.modals.menu = true;
                self.titlebar.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::none()
            }
            Message::ToggleDropdownMenu => {
                self.titlebar.show_dropdown_menu = !self.titlebar.show_dropdown_menu;
                if self.titlebar.show_dropdown_menu {
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.titlebar.slide_ticks = crate::app::TITLEBAR_SLIDE_TICKS;
                    self.titlebar.dropdown_menu_slide_ticks = 0;
                    self.hovered_player_id = None;
                    self.overlay.ticks = 0;
                } else {
                    self.titlebar.dropdown_menu_slide_ticks = 0;
                    if !self.is_point_in_titlebar(self.window.cursor_position) {
                        self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    }
                }
                Task::none()
            }
            Message::CloseDropdownMenu => {
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.dropdown_menu_slide_ticks = 0;
                if !self.is_point_in_titlebar(self.window.cursor_position) {
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                }
                Task::none()
            }
            Message::SetWindowOpacity(opacity) => {
                self.settings.window_opacity = opacity.clamp(0.05, 1.0);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleAlwaysOnTop => {
                self.settings.is_always_on_top = !self.settings.is_always_on_top;
                if !self.settings.is_always_on_top {
                    self.window.ghost_passthrough_active = false;
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                self.overlay.toast_message = Some(if self.settings.is_always_on_top {
                    self.t("wazoo.always_on_top_enabled")
                } else {
                    self.t("wazoo.always_on_top_disabled")
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;

                if let Some(id) = self.window.id {
                    let level = if self.settings.is_always_on_top {
                        iced::window::Level::AlwaysOnTop
                    } else {
                        iced::window::Level::Normal
                    };
                    let level_task = iced::window::set_level(id, level);
                    let passthrough_task = if self.settings.is_always_on_top {
                        if !self.window.is_focused && !self.is_modal_or_menu_open() {
                            self.window.ghost_passthrough_active = true;
                            iced::window::enable_mouse_passthrough(id)
                        } else {
                            Task::none()
                        }
                    } else {
                        iced::window::disable_mouse_passthrough(id)
                    };
                    return Task::batch([level_task, passthrough_task]);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Throttles frame rendering to ~30 FPS (every 2nd tick) when the window is unfocused or obscured.
    pub(crate) fn should_throttle_unfocused_frame(&mut self) -> bool {
        if !self.window.is_focused {
            self.window.unfocused_frame_ticks = self.window.unfocused_frame_ticks.wrapping_add(1);
            !self.window.unfocused_frame_ticks.is_multiple_of(2)
        } else {
            false
        }
    }

    /// Ticks titlebar visibility, slide transitions, hover delays, and window dragging state.
    pub(crate) fn tick_titlebar_animation(&mut self) {
        if self.window.is_dragging
            || self.titlebar.drag_pending
            || self.is_point_in_titlebar(self.window.cursor_position)
            || self.titlebar.show_dropdown_menu
        {
            if self.window.is_dragging || self.titlebar.drag_pending {
                let is_still_moving = self
                    .window
                    .last_drag_move
                    .map(|t| t.elapsed() < Duration::from_millis(300))
                    .unwrap_or(false);
                if !is_still_moving
                    && !self.titlebar.drag_pending
                    && !self.is_point_in_titlebar(self.window.cursor_position)
                {
                    self.window.is_dragging = false;
                    self.window.last_drag_move = None;
                    if self.titlebar.hide_ticks > TITLEBAR_FADE_TICKS {
                        self.titlebar.hide_ticks = TITLEBAR_FADE_TICKS;
                    }
                } else {
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.titlebar.hover_ticks = 0;
                    self.titlebar.slide_ticks = TITLEBAR_SLIDE_TICKS;
                }
            } else if self.titlebar.show_dropdown_menu {
                self.titlebar.show = true;
                self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                self.titlebar.hover_ticks = 0;
                self.titlebar.slide_ticks = TITLEBAR_SLIDE_TICKS;
                if self.titlebar.dropdown_menu_slide_ticks < DROPDOWN_MENU_SLIDE_TICKS {
                    self.titlebar.dropdown_menu_slide_ticks += 1;
                }
            } else if self.titlebar.show {
                self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                if self.titlebar.slide_ticks < TITLEBAR_SLIDE_TICKS {
                    self.titlebar.slide_ticks += 1;
                }
            } else {
                self.titlebar.hover_ticks += 1;
                if self.titlebar.hover_ticks >= TITLEBAR_SHOW_DELAY_TICKS {
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.titlebar.hover_ticks = 0;
                    self.titlebar.slide_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        let current_fade =
                            self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.overlay.ticks = self.overlay.ticks.min(current_fade);
                    }
                }
            }
        } else {
            self.titlebar.hover_ticks = 0;
            self.titlebar.dropdown_menu_slide_ticks = 0;
            if self.titlebar.hide_ticks > 0 {
                self.titlebar.hide_ticks -= 1;
                if self.titlebar.hide_ticks == 0 {
                    self.titlebar.show = false;
                    self.titlebar.slide_ticks = 0;
                }
            }
        }
    }
}
