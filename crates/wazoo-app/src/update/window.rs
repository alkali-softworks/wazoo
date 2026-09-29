/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Lifecycle & Hardware Input Reducer
 */

use crate::app::{
    DEFAULT_TOAST_SECS, PLAYER_OVERLAY_FADE_TICKS, TITLEBAR_FADE_TICKS, TITLEBAR_HIDE_TICKS,
    WazooApp,
};
use crate::message::Message;
use iced::{Point, Task, keyboard::Key};
use std::time::{Duration, Instant};
use wazoo_core::PlaybackMode;

impl WazooApp {
    pub(crate) fn update_window(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowIdReceived(id) => {
                self.window_id = Some(id);
                let focus_task = iced::window::gain_focus(id);
                let level_task = if self.settings.is_always_on_top {
                    iced::window::set_level(id, iced::window::Level::AlwaysOnTop)
                } else {
                    Task::none()
                };
                Task::batch([focus_task, level_task])
            }
            Message::GainWindowFocus => {
                if let Some(id) = self.window_id {
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
                self.is_window_focused = true;
                self.unfocused_frame_ticks = 0;
                self.is_alt_pressed = false;
                if self.settings.is_always_on_top && self.ghost_passthrough_active {
                    self.ghost_passthrough_active = false;
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    if let Some(id) = self.window_id {
                        return iced::window::disable_mouse_passthrough(id);
                    }
                }
                Task::none()
            }
            Message::WindowUnfocused => {
                self.is_window_focused = false;
                self.is_alt_pressed = false;
                if !self.is_window_dragging {
                    self.cursor_position = Point::new(-1000.0, -1000.0);
                    self.titlebar_drag_pending = false;
                    self.titlebar_press_origin = None;
                    self.titlebar_hover_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        let current_fade =
                            self.player_overlay_fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.player_overlay_ticks = self.player_overlay_ticks.min(current_fade);
                    }
                }
                if self.window_bounds_dirty {
                    self.window_bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                if self.settings.is_always_on_top
                    && !self.ghost_passthrough_active
                    && !self.is_modal_or_menu_open()
                {
                    self.ghost_passthrough_active = true;
                    if let Some(id) = self.window_id {
                        return iced::window::enable_mouse_passthrough(id);
                    }
                }
                Task::none()
            }
            Message::WindowMoved(id, point) => {
                self.window_id = Some(id);
                let new_x = point.x as i32;
                let new_y = point.y as i32;
                if self.settings.window_bounds.x != new_x || self.settings.window_bounds.y != new_y {
                    self.settings.window_bounds.x = new_x;
                    self.settings.window_bounds.y = new_y;
                    self.window_bounds_dirty = true;
                }
                if self.is_window_dragging {
                    self.last_window_drag_move = Some(Instant::now());
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                }
                Task::none()
            }
            Message::WindowResized(id, size) => {
                self.window_id = Some(id);
                let new_w = (size.width as u32).clamp(200, 7680);
                let new_h = (size.height as u32).clamp(150, 4320);
                if self.settings.window_bounds.width != new_w
                    || self.settings.window_bounds.height != new_h
                {
                    self.settings.window_bounds.width = new_w;
                    self.settings.window_bounds.height = new_h;
                    self.window_bounds_dirty = true;
                }
                self.scroll_engine.set_window_size(size.width, size.height);
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    for p in &self.players {
                        let item_h = self.calculate_player_scroll_height(p);
                        self.scroll_engine.update_height(p.id, item_h);
                    }
                    self.scroll_engine.recalculate_positions();
                    let margin = self.scroll_engine.default_item_height() * 1.5;
                    while let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(margin) {
                        if let Some(mut handle) = self.preloaded_player.take() {
                            let item_h = self.calculate_player_scroll_height(&handle);
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.record_play_history(&handle.state.path);
                            self.players.push(handle);
                        } else {
                            break;
                        }
                    }
                    return self.trigger_preload_task();
                }
                Task::none()
            }
            Message::MinimizeWindow => {
                if let Some(id) = self.window_id {
                    iced::window::minimize(id, true)
                } else {
                    Task::none()
                }
            }
            Message::MaximizeWindow => {
                self.last_titlebar_click = None;
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
                if let Some(id) = self.window_id {
                    iced::window::toggle_maximize(id)
                } else {
                    Task::none()
                }
            }
            Message::DragWindow => {
                if let Some(id) = self.window_id {
                    self.is_window_dragging = true;
                    self.last_window_drag_move = Some(Instant::now());
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    iced::window::drag(id)
                } else {
                    Task::none()
                }
            }
            Message::DragResize(direction) => {
                if let Some(id) = self.window_id {
                    iced::window::drag_resize(id, direction)
                } else {
                    Task::none()
                }
            }
            Message::CloseApp => {
                self.save_session_state();
                if let Some(id) = self.window_id {
                    iced::window::close(id)
                } else {
                    std::process::exit(0);
                }
            }
            Message::ModifiersChanged(modifiers) => {
                self.is_alt_pressed = modifiers.alt();
                Task::none()
            }
            Message::KeyPressed(key, status) => self.handle_key_pressed(key, status),
            Message::KeyReleased(key) => {
                if key == Key::Named(iced::keyboard::key::Named::Alt)
                    || key == Key::Named(iced::keyboard::key::Named::AltGraph)
                {
                    self.is_alt_pressed = false;
                }
                Task::none()
            }
            Message::CursorMoved(win_id, pos) => self.handle_cursor_moved(win_id, pos),
            Message::CursorLeft => {
                if !self.is_window_dragging && !self.titlebar_drag_pending {
                    self.cursor_position = Point::new(-1000.0, -1000.0);
                    self.titlebar_hover_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        let current_fade =
                            self.player_overlay_fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        self.player_overlay_ticks = self.player_overlay_ticks.min(current_fade);
                    }
                }
                Task::none()
            }
            Message::TitleBarPressed => {
                let now = Instant::now();
                if let Some(last_click) = self.last_titlebar_click {
                    if now.duration_since(last_click) < Duration::from_millis(400) {
                        self.last_titlebar_click = None;
                        self.titlebar_drag_pending = false;
                        self.is_window_dragging = false;
                        self.titlebar_press_origin = None;
                        return self.update(Message::MaximizeWindow);
                    }
                }
                self.last_titlebar_click = Some(now);
                self.titlebar_drag_pending = true;
                self.titlebar_press_origin = Some(self.cursor_position);
                self.show_titlebar = true;
                self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                Task::none()
            }
            Message::LeftClickReleased => {
                let was_dragging = self.is_window_dragging;
                self.is_window_dragging = false;
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
                self.last_window_drag_move = None;
                if was_dragging {
                    if self.is_point_in_titlebar(self.cursor_position) {
                        self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    } else {
                        self.titlebar_hide_ticks = TITLEBAR_FADE_TICKS;
                    }
                } else if !self.is_point_in_titlebar(self.cursor_position) {
                    self.titlebar_hide_ticks = self.titlebar_hide_ticks.min(TITLEBAR_FADE_TICKS);
                }
                if self.window_bounds_dirty {
                    self.window_bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                Task::none()
            }
            Message::RightClickPressed(win_id) => {
                self.window_id = Some(win_id);
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.player_overlay_fade_in_ticks = 0;
                Task::none()
            }
            Message::ToggleDropdownMenu => {
                self.show_dropdown_menu = !self.show_dropdown_menu;
                if self.show_dropdown_menu {
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.titlebar_slide_ticks = crate::app::TITLEBAR_SLIDE_TICKS;
                    self.dropdown_menu_slide_ticks = 0;
                    self.hovered_player_id = None;
                    self.player_overlay_ticks = 0;
                } else {
                    self.dropdown_menu_slide_ticks = 0;
                    if !self.is_point_in_titlebar(self.cursor_position) {
                        self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    }
                }
                Task::none()
            }
            Message::CloseDropdownMenu => {
                self.show_dropdown_menu = false;
                self.dropdown_menu_slide_ticks = 0;
                if !self.is_point_in_titlebar(self.cursor_position) {
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
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
                    self.ghost_passthrough_active = false;
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                self.toast_message = Some(if self.settings.is_always_on_top {
                    self.t("wazoo.always_on_top_enabled")
                } else {
                    self.t("wazoo.always_on_top_disabled")
                });
                self.toast_time_remaining = DEFAULT_TOAST_SECS;

                if let Some(id) = self.window_id {
                    let level = if self.settings.is_always_on_top {
                        iced::window::Level::AlwaysOnTop
                    } else {
                        iced::window::Level::Normal
                    };
                    let level_task = iced::window::set_level(id, level);
                    let passthrough_task = if self.settings.is_always_on_top {
                        if !self.is_window_focused && !self.is_modal_or_menu_open() {
                            self.ghost_passthrough_active = true;
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
}
