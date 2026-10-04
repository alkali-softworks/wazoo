/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * 3D Cube & Cube Overlay Reducer
 *
 * Implements message handlers, physics ticks, and lifecycle state transitions for floating 3D cubes (Mode 8)
 * and the borderless desktop Cube Overlay Mode (Mode 9).
 */

use crate::app::WazooApp;
use crate::message::Message;
use iced::Task;
use wazoo_core::PlaybackMode;

impl WazooApp {
    /// Handles message dispatch, settings changes, and state transitions for 3D cubes and Cube Overlay Mode.
    pub(crate) fn update_cube(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleCubeScreensaver => {
                self.modals.menu = false;
                self.titlebar.show_dropdown_menu = false;
                if self.cube.is_present() {
                    let exit_task = self.dismiss_all_cubes();
                    self.overlay.show_toast(self.t("toast.cube_removed"));
                    return exit_task;
                } else {
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        self.cleanup_scroll_mode();
                        self.settings.playback_mode = PlaybackMode::Normal;
                        self.restore_grid_players_to_target(self.settings.player_count);
                    }
                    self.spawn_new_cube();
                    self.overlay.focus_border_ticks = crate::app::FOCUS_BORDER_TICKS;
                    let key = self.settings.keybinds.menu_hint(&self.settings.keybinds.toggle_cube);
                    self.overlay
                        .show_toast(self.t_with("toast.cube_added", &[("key", &key)]));
                }
                Task::none()
            }
            Message::ToggleDesktopCubeScreensaver => {
                self.modals.close_all();
                self.drawers.close_all();
                self.titlebar.show_dropdown_menu = false;

                if self.cube.desktop_overlay {
                    self.exit_desktop_cube_overlay()
                } else {
                    // Clean up scroll mode if active so Scroll Mode never runs invisibly behind Mode 9
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        self.cleanup_scroll_mode();
                        self.settings.playback_mode = PlaybackMode::Normal;
                        self.restore_grid_players_to_target(self.settings.player_count);
                    }

                    // Save pre-desktop window bounds and maximized state so we can restore them upon exit
                    self.cube.pre_desktop_bounds = Some(self.settings.window_bounds.clone());
                    self.cube.was_maximized_before_desktop = self.window.is_maximized;

                    // Exit fullscreen mode if active so Mode 9 desktop overlay runs in windowed/maximized mode
                    let exit_fullscreen_task = if self.window.is_fullscreen {
                        self.titlebar.last_click = None;
                        self.titlebar.drag_pending = false;
                        self.titlebar.press_origin = None;
                        self.window.is_fullscreen = false;
                        if let Some(id) = self.window.id {
                            iced::window::set_mode(id, iced::window::Mode::Windowed)
                        } else {
                            Task::none()
                        }
                    } else {
                        Task::none()
                    };

                    self.window.is_maximized = true;

                    // Track whether the cube was already active before entering Mode 9
                    let was_active = self.cube.is_present();
                    self.cube.spawned_for_desktop = !was_active;

                    // Ensure cube is enabled and at least one cube is bouncing
                    if !was_active {
                        self.spawn_new_cube();
                    }

                    self.cube.desktop_overlay = true;

                    // Suspend background decoding and audio for all invisible regular players (0% CPU waste)
                    for p in &mut self.players {
                        if !p.is_cube {
                            p.was_paused_before_desktop = !p.is_playing();
                            p.set_paused(true);
                        }
                    }

                    let key = self.settings.keybinds.menu_hint(&self.settings.keybinds.toggle_cube_overlay);
                    self.overlay
                        .show_toast(self.t_with("toast.cube_overlay_enabled", &[("key", &key)]));

                    if let Some(id) = self.window.id {
                        let level_task =
                            iced::window::set_level(id, iced::window::Level::AlwaysOnTop);
                        let passthrough_task = iced::window::enable_mouse_passthrough(id);
                        let maximize_task = iced::window::maximize(id, true);
                        Task::batch([exit_fullscreen_task, level_task, passthrough_task, maximize_task])
                    } else {
                        exit_fullscreen_task
                    }
                }
            }
            Message::ToggleCubeSheen => {
                self.settings.cube_sheen = !self.settings.cube_sheen;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SpawnCube => {
                self.modals.menu = false;
                self.titlebar.show_dropdown_menu = false;
                self.spawn_new_cube();
                self.overlay.focus_border_ticks = crate::app::FOCUS_BORDER_TICKS;
                let key = self.settings.keybinds.menu_hint(&self.settings.keybinds.toggle_cube);
                let count_str = self.cube.cubes.len().to_string();
                self.overlay.show_toast(self.t_with(
                    "toast.cube_added_num",
                    &[("count", &count_str), ("key", &key)],
                ));
                Task::none()
            }
            Message::RemoveCube => {
                if self.cube.desktop_overlay && self.cube.cubes.len() <= 1 {
                    self.cube.spawned_for_desktop = true;
                    return self.exit_desktop_cube_overlay();
                }

                if let Some(removed) = self.cube.remove_cube() {
                    if let Some(pid) = removed.player_id {
                        if let Some(pos) = self.players.player_index(pid) {
                            let mut p = self.players.remove(pos);
                            p.stop();
                            if self.focused_idx >= self.players.len() && !self.players.is_empty() {
                                self.focused_idx = self.players.len() - 1;
                            }
                        }
                    }
                }
                if self.cube.cubes.is_empty() {
                    let was_desktop = self.cube.desktop_overlay;
                    let exit_task = self.dismiss_all_cubes();
                    if !was_desktop {
                        self.overlay.show_toast(self.t("toast.cube_removed"));
                    }
                    return exit_task;
                } else {
                    let count_str = self.cube.cubes.len().to_string();
                    self.overlay.show_toast(self.t_with(
                        "toast.cube_removed_remaining",
                        &[("count", &count_str)],
                    ));
                }
                Task::none()
            }
            Message::ClearCubes => {
                self.modals.menu = false;
                self.titlebar.show_dropdown_menu = false;
                let exit_task = self.dismiss_all_cubes();
                self.overlay.show_toast(self.t("toast.cube_all_removed"));
                exit_task
            }
            Message::SetCubeSpeed(speed) => {
                let clamped = speed.clamp(0.2, 4.0);
                self.cube.speed_multiplier = clamped;
                self.settings.cube_speed = clamped;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCubeSize(size) => {
                let clamped = size.clamp(0.4, 3.0);
                self.cube.size_multiplier = clamped;
                self.settings.cube_size = clamped;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Advances physics, multi-axis 3D rotations, and screen edge collisions for 3D cubes.
    pub(crate) fn tick_cube_screensaver(&mut self) {
        if self.cube.is_present() {
            let w = self.settings.window_bounds.width as f32;
            let h = self.settings.window_bounds.height as f32;
            self.cube.tick(w, h);
        }
    }

    /// Spawns a new independent video player and 3D cube instance, focusing it.
    pub(crate) fn spawn_new_cube(&mut self) {
        let w = self.settings.window_bounds.width as f32;
        let h = self.settings.window_bounds.height as f32;
        let cube_pid = self.spawn_cube_player();
        let count = self.cube.cubes.len();
        self.cube.spawn_cube_with_player(w, h, count, cube_pid);
        if let Some(pid) = cube_pid {
            if let Some(pos) = self.players.player_index(pid) {
                self.focused_idx = pos;
            }
        }
    }

    /// Completely dismisses all 3D cubes, cleans up their players, and exits Mode 9 if active.
    pub(crate) fn dismiss_all_cubes(&mut self) -> Task<Message> {
        let exit_task = self.exit_desktop_cube_overlay();
        self.cube.clear();
        self.cleanup_cube_players();
        exit_task
    }

    /// Disables Cube Overlay Mode (Mode 9), restoring normal window level,
    /// turning off click-passthrough, unpausing regular players, and dismissing the cube
    /// if it was spawned exclusively for Cube Overlay Mode.
    pub(crate) fn exit_desktop_cube_overlay(&mut self) -> Task<Message> {
        if !self.cube.desktop_overlay {
            return Task::none();
        }
        self.cube.desktop_overlay = false;

        let was_maximized = self.cube.was_maximized_before_desktop;
        let pre_bounds = self.cube.pre_desktop_bounds.take();

        // If the cube was spawned solely for Mode 9, dismiss it completely
        if self.cube.spawned_for_desktop {
            self.cube.clear();
            self.cleanup_cube_players();
        }

        // Resume regular grid players that were actively playing before entering Mode 9
        for p in &mut self.players {
            if !p.is_cube && !p.was_paused_before_desktop {
                p.set_paused(false);
            }
        }

        self.overlay.show_toast(self.t("toast.cube_overlay_disabled"));

        if let Some(id) = self.window.id {
            let level = if self.settings.is_always_on_top {
                iced::window::Level::AlwaysOnTop
            } else {
                iced::window::Level::Normal
            };
            let level_task = iced::window::set_level(id, level);
            self.window.ghost_passthrough_active = false;
            let passthrough_task = iced::window::disable_mouse_passthrough(id);

            let restore_task = if !was_maximized {
                self.window.is_maximized = false;
                let unmaximize_task = iced::window::maximize(id, false);
                let geometry_task = if let Some(prev) = pre_bounds {
                    self.settings.window_bounds = prev.clone();
                    let resize_task = iced::window::resize(id, iced::Size::new(prev.width as f32, prev.height as f32));
                    let move_task = iced::window::move_to(id, iced::Point::new(prev.x as f32, prev.y as f32));
                    Task::batch([resize_task, move_task])
                } else {
                    Task::none()
                };
                Task::batch([unmaximize_task, geometry_task])
            } else {
                Task::none()
            };

            Task::batch([level_task, passthrough_task, restore_task])
        } else {
            if !was_maximized {
                self.window.is_maximized = false;
                if let Some(prev) = pre_bounds {
                    self.settings.window_bounds = prev;
                }
            }
            Task::none()
        }
    }
}
