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
    /// Handles messages for window management, borderless drag-resizing, cursor tracking,
    /// titlebar auto-hiding, hardware keyboard key events, and mouse interactions.
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
                if self.cube.desktop_overlay {
                    if let Some(id) = self.window.id {
                        return iced::window::enable_mouse_passthrough(id);
                    }
                } else if self.window.clickthru {
                    self.window.clickthru = false;
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
                if self.cube.desktop_overlay {
                    if let Some(id) = self.window.id {
                        return iced::window::enable_mouse_passthrough(id);
                    }
                } else if self.settings.is_always_on_top
                    && self.settings.window_opacity < 0.99
                    && !self.window.clickthru
                    && !self.is_modal_or_menu_open()
                {
                    self.window.clickthru = true;
                    if let Some(id) = self.window.id {
                        return iced::window::enable_mouse_passthrough(id);
                    }
                }
                Task::none()
            }
            Message::WindowMoved(id, point) => {
                self.window.id = Some(id);
                if self.window.is_resizing() {
                    self.window.mark_resized();
                }
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
                self.window.mark_resized();
                let new_w = (size.width as u32).clamp(200, 7680);
                let new_h = (size.height as u32).clamp(150, 4320);
                if self.settings.window_bounds.width != new_w
                    || self.settings.window_bounds.height != new_h
                {
                    self.settings.window_bounds.width = new_w;
                    self.settings.window_bounds.height = new_h;
                    if !self.cube.desktop_overlay {
                        self.window.bounds_dirty = true;
                    }
                }
                self.scroll_engine.set_window_size(size.width, size.height);
                if self.settings.playback_mode == PlaybackMode::Scroll && !self.window.is_resizing() {
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
                self.window.is_maximized = !self.window.is_maximized;
                if let Some(id) = self.window.id {
                    iced::window::toggle_maximize(id)
                } else {
                    Task::none()
                }
            }
            Message::ToggleFullscreen => {
                self.titlebar.last_click = None;
                self.titlebar.drag_pending = false;
                self.titlebar.press_origin = None;
                self.window.is_fullscreen = !self.window.is_fullscreen;
                let mode = if self.window.is_fullscreen {
                    iced::window::Mode::Fullscreen
                } else {
                    iced::window::Mode::Windowed
                };
                if self.window.is_fullscreen {
                    let key = self
                        .settings
                        .keybinds
                        .menu_hint(&self.settings.keybinds.toggle_fullscreen);
                    self.overlay
                        .show_toast(self.t_with("toast.fullscreen_enabled", &[("key", &key)]));
                } else {
                    self.overlay.show_toast(self.t("toast.fullscreen_disabled"));
                }
                if let Some(id) = self.window.id {
                    iced::window::set_mode(id, mode)
                } else {
                    Task::none()
                }
            }
            Message::FitWindow => {
                if self.window.is_fullscreen {
                    return Task::none();
                }

                let grid_players: Vec<&AppPlayer> =
                    self.players.iter().filter(|p| !p.is_cube).collect();
                if grid_players.is_empty() {
                    return Task::none();
                }

                let player_aspect = self
                    .focused_player()
                    .filter(|p| !p.is_cube)
                    .and_then(|p| p.aspect_ratio())
                    .or_else(|| grid_players.iter().find_map(|p| p.aspect_ratio()));

                let Some(tile_ar) = player_aspect else {
                    return Task::none();
                };

                if tile_ar <= 0.05 {
                    return Task::none();
                }

                let (cols, rows) = match self.settings.playback_mode {
                    PlaybackMode::Scroll => (1, 1),
                    _ => match self.settings.layout {
                        wazoo_core::LayoutMode::Row => (grid_players.len().max(1), 1),
                        wazoo_core::LayoutMode::Column => (1, grid_players.len().max(1)),
                        wazoo_core::LayoutMode::Grid => {
                            let count = grid_players.len();
                            if count <= 1 {
                                (1, 1)
                            } else if count == 2 {
                                (2, 1)
                            } else if count == 3 {
                                (1, 1)
                            } else {
                                let cols = if count <= 4 {
                                    2
                                } else if count <= 9 {
                                    3
                                } else {
                                    4
                                };
                                let rows = count.div_ceil(cols);
                                (cols, rows)
                            }
                        }
                    },
                };

                let target_player_ar = tile_ar * (cols as f32 / rows as f32);
                if target_player_ar <= 0.05 {
                    return Task::none();
                }

                let drawer_width = if self.drawers.show_file_picker || self.drawers.show_history_drawer {
                    420.0
                } else if self.drawers.show_transcript {
                    440.0
                } else {
                    0.0
                };

                let current_total_w = self.settings.window_bounds.width as f32;
                let current_total_h = self.settings.window_bounds.height as f32;

                let current_player_w = (current_total_w - drawer_width).max(50.0);
                let current_player_h = current_total_h.max(50.0);
                let current_ar = current_player_w / current_player_h;

                let (new_player_w, new_player_h) = if current_ar > target_player_ar {
                    // Pillarbox: extra space on left & right.
                    // Shrink width to fit video height without altering vertical scale.
                    (current_player_h * target_player_ar, current_player_h)
                } else {
                    // Letterbox: extra space on top & bottom.
                    // Shrink height to fit video width without altering horizontal scale.
                    (current_player_w, current_player_w / target_player_ar)
                };

                let mut final_total_w = (new_player_w + drawer_width).round();
                let mut final_total_h = new_player_h.round();

                if final_total_w < 200.0 {
                    final_total_w = 200.0;
                    let p_w = (final_total_w - drawer_width).max(50.0);
                    final_total_h = (p_w / target_player_ar).round();
                }
                if final_total_h < 150.0 {
                    final_total_h = 150.0;
                    let p_h = final_total_h;
                    final_total_w = (p_h * target_player_ar + drawer_width).round();
                }

                let clamped_w = (final_total_w as u32).clamp(200, 7680);
                let clamped_h = (final_total_h as u32).clamp(150, 4320);

                self.settings.window_bounds.width = clamped_w;
                self.settings.window_bounds.height = clamped_h;
                self.window.bounds_dirty = true;
                self.window.mark_resized();
                self.scroll_engine.set_window_size(clamped_w as f32, clamped_h as f32);

                let resize_size = iced::Size::new(clamped_w as f32, clamped_h as f32);
                if let Some(id) = self.window.id {
                    iced::window::resize(id, resize_size)
                } else {
                    iced::window::oldest().then(move |maybe_id| {
                        if let Some(id) = maybe_id {
                            iced::window::resize(id, resize_size)
                        } else {
                            Task::none()
                        }
                    })
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
                if self.window.is_fullscreen {
                    return Task::none();
                }
                self.window.mark_resized();
                if let Some(id) = self.window.id {
                    iced::window::drag_resize(id, direction)
                } else {
                    Task::none()
                }
            }
            Message::AltDragResize => {
                if self.window.is_fullscreen {
                    return Task::none();
                }
                let direction = self.calculate_alt_resize_direction();
                self.update(Message::DragResize(direction))
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
                let was_resizing = self.window.is_resizing();
                self.window.is_dragging = false;
                self.titlebar.drag_pending = false;
                self.titlebar.press_origin = None;
                self.window.last_drag_move = None;
                self.window.last_resize_time = None;
                if was_resizing {
                    self.update_player_frames();
                    self.sync_scroll_item_heights();
                }
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

                // If user clicked directly on any active 3D cube, focus its independent player
                if !was_dragging && !was_resizing && !self.is_any_modal_open() && self.cube.is_present() {
                    let click_pos = self.window.cursor_position;
                    let szm = self.cube.size_multiplier.clamp(0.4, 3.0);
                    for cube in &self.cube.cubes {
                        let radius = cube.size * szm * 1.15;
                        let dx = click_pos.x - cube.x;
                        let dy = click_pos.y - cube.y;
                        if dx * dx + dy * dy <= radius * radius {
                            if let Some(pid) = cube.player_id {
                                return self.update(Message::PlayerClicked(pid));
                            }
                        }
                    }
                }

                Task::none()
            }
            Message::RightClickPressed(win_id) => {
                self.window.id = Some(win_id);
                if self.window.is_alt_pressed {
                    return self.update(Message::AltDragResize);
                }
                self.modals.menu = true;
                self.titlebar.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::none()
            }
            Message::RightClickReleased => {
                let was_resizing = self.window.is_resizing();
                self.window.last_resize_time = None;
                if was_resizing {
                    self.update_player_frames();
                    self.sync_scroll_item_heights();
                }
                if self.window.bounds_dirty {
                    self.window.bounds_dirty = false;
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
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

                if let Some(id) = self.window.id {
                    if self.settings.is_always_on_top
                        && !self.window.is_focused
                        && !self.is_modal_or_menu_open()
                    {
                        if self.settings.window_opacity < 0.99 {
                            self.window.clickthru = true;
                            return iced::window::enable_mouse_passthrough(id);
                        } else if self.window.clickthru {
                            self.window.clickthru = false;
                            return iced::window::disable_mouse_passthrough(id);
                        }
                    }
                }
                Task::none()
            }
            Message::CyclePinMode => {
                // 4-stage cycle:
                // 1st press: Always on top + 100% opacity (no clickthru)
                // 2nd press: Always on top + 40% opacity + clickthru
                // 3rd press: Always on top + 15% opacity + clickthru
                // 4th press: Toggle back to normal
                let click_through = self.t("common.click_through");
                let (next_pinned, next_opacity, clickthru, label) = if !self.settings.is_always_on_top {
                    (true, 1.0f32, false, "100%".to_string())
                } else if self.settings.window_opacity > 0.60 {
                    (true, 0.40f32, true, format!("40% • {click_through}"))
                } else if self.settings.window_opacity > 0.25 {
                    (true, 0.15f32, true, format!("15% • {click_through}"))
                } else {
                    (false, 1.0f32, false, String::new())
                };

                self.settings.is_always_on_top = next_pinned;
                self.settings.window_opacity = next_opacity;
                if !next_pinned || !clickthru {
                    self.window.clickthru = false;
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                self.overlay.toast_message = Some(if next_pinned {
                    format!("{} ({label})", self.t("wazoo.always_on_top_enabled"))
                } else {
                    self.t("wazoo.always_on_top_disabled")
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;

                if let Some(id) = self.window.id {
                    let level = if next_pinned {
                        iced::window::Level::AlwaysOnTop
                    } else {
                        iced::window::Level::Normal
                    };
                    let level_task = iced::window::set_level(id, level);
                    let passthrough_task = if next_pinned && clickthru {
                        if !self.window.is_focused && !self.is_modal_or_menu_open() {
                            self.window.clickthru = true;
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
            Message::ToggleAlwaysOnTop => {
                self.settings.is_always_on_top = !self.settings.is_always_on_top;
                if !self.settings.is_always_on_top {
                    self.window.clickthru = false;
                    self.settings.window_opacity = 1.0;
                }
                let _ = self.config_mgr.save_settings(&self.settings);

                let is_pinned = self.settings.is_always_on_top;
                let has_clickthru = is_pinned && self.settings.window_opacity < 0.99;

                self.overlay.toast_message = Some(if is_pinned {
                    if has_clickthru {
                        format!(
                            "{} ({}% • {})",
                            self.t("wazoo.always_on_top_enabled"),
                            (self.settings.window_opacity * 100.0).round() as u32,
                            self.t("common.click_through")
                        )
                    } else {
                        format!("{} (100%)", self.t("wazoo.always_on_top_enabled"))
                    }
                } else {
                    self.t("wazoo.always_on_top_disabled")
                });
                self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;

                if let Some(id) = self.window.id {
                    let level = if is_pinned {
                        iced::window::Level::AlwaysOnTop
                    } else {
                        iced::window::Level::Normal
                    };
                    let level_task = iced::window::set_level(id, level);
                    let passthrough_task = if has_clickthru {
                        if !self.window.is_focused && !self.is_modal_or_menu_open() {
                            self.window.clickthru = true;
                            iced::window::enable_mouse_passthrough(id)
                        } else {
                            Task::none()
                        }
                    } else {
                        self.window.clickthru = false;
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
        if self.cube.desktop_overlay {
            return false;
        }
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

    /// Calculates the drag-resize direction based on cursor position within the window.
    /// Divides the window into a 3x3 grid (9 zones) matching Cinnamon / Linux window manager conventions:
    /// - Corners resize diagonally (NorthWest, NorthEast, SouthWest, SouthEast).
    /// - Outer edges resize cardinally (North, South, East, West).
    /// - Center zone delegates to the nearest quadrant diagonal.
    pub fn calculate_alt_resize_direction(&self) -> iced::window::Direction {
        let w = (self.settings.window_bounds.width as f32).max(1.0);
        let h = (self.settings.window_bounds.height as f32).max(1.0);
        let pos = self.window.cursor_position;

        let col = if pos.x < w / 3.0 {
            0 // Left
        } else if pos.x < 2.0 * w / 3.0 {
            1 // Center
        } else {
            2 // Right
        };

        let row = if pos.y < h / 3.0 {
            0 // Top
        } else if pos.y < 2.0 * h / 3.0 {
            1 // Middle
        } else {
            2 // Bottom
        };

        match (row, col) {
            (0, 0) => iced::window::Direction::NorthWest,
            (0, 1) => iced::window::Direction::North,
            (0, 2) => iced::window::Direction::NorthEast,
            (1, 0) => iced::window::Direction::West,
            (1, 2) => iced::window::Direction::East,
            (2, 0) => iced::window::Direction::SouthWest,
            (2, 1) => iced::window::Direction::South,
            (2, 2) => iced::window::Direction::SouthEast,
            (1, 1) => {
                // In the center 1/9th, pick the quadrant of the window the cursor is in
                let is_left = pos.x < w / 2.0;
                let is_top = pos.y < h / 2.0;
                match (is_top, is_left) {
                    (true, true) => iced::window::Direction::NorthWest,
                    (true, false) => iced::window::Direction::NorthEast,
                    (false, true) => iced::window::Direction::SouthWest,
                    (false, false) => iced::window::Direction::SouthEast,
                }
            }
            _ => iced::window::Direction::SouthEast,
        }
    }
}
