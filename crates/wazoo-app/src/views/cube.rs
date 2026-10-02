/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * 3D Video Cube Screensaver View
 *
 * Renders the floating 3D rotating bouncing cubes over the application layer.
 */

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use iced::Element;
use wazoo_media::cube::{CubeInstance, CubeProgram, cube_shader};
use wazoo_media::pipeline::FrameData;

use crate::app::WazooApp;
use crate::message::Message;

impl WazooApp {
    pub fn view_cube_overlay(&self) -> Element<'_, Message> {
        let mut instances = Vec::new();
        let fallback_frame = Arc::new(Mutex::new(FrameData {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            new_frame: false,
            frame_seq: 0,
        }));
        let fallback_alive = Arc::new(AtomicBool::new(true));
        let focused_pid = self.focused_player_id();
        let is_flashing_focus = self.overlay.focus_border_ticks > 0;

        for cube in &self.cube.cubes {
            let (frame, alive, player_id) = if let Some(pid) = cube.player_id {
                if let Some(p) = self.players.player(pid) {
                    (p.frame(), p.alive(), p.id as u64)
                } else if let Some(p) = self.players.iter().find(|p| p.is_cube).or_else(|| self.players.first()) {
                    (p.frame(), p.alive(), p.id as u64)
                } else {
                    (Arc::clone(&fallback_frame), Arc::clone(&fallback_alive), 0)
                }
            } else if let Some(p) = self.players.iter().find(|p| p.is_cube).or_else(|| self.players.first()) {
                (p.frame(), p.alive(), p.id as u64)
            } else {
                (Arc::clone(&fallback_frame), Arc::clone(&fallback_alive), 0)
            };

            let is_this_cube_focused = if let Some(pid) = cube.player_id {
                Some(pid) == focused_pid
            } else if let Some(focused_p) = self.focused_player() {
                focused_p.is_cube
            } else {
                false
            };

            let edge_color = if is_this_cube_focused && is_flashing_focus {
                let alpha = if self.overlay.focus_border_ticks > 4 {
                    1.0
                } else {
                    self.overlay.focus_border_ticks as f32 / 4.0
                };
                // Emerald green matching COLOR_PRIMARY [0.26, 0.72, 0.51] (#42b883)
                [0.26, 0.72, 0.51, alpha]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };

            instances.push(CubeInstance {
                cube_id: cube.id,
                x: cube.x,
                y: cube.y,
                size: cube.size * self.cube.size_multiplier,
                rx: cube.rx,
                ry: cube.ry,
                rz: cube.rz,
                edge_color,
                player_id,
                frame,
                alive,
                crt_enabled: self.settings.crt_enabled,
                opacity: 1.0,
            });
        }

        let program = CubeProgram::new(instances);
        cube_shader(program).into()
    }
}
