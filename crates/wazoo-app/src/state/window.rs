/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Lifecycle & Cursor State
 */

use iced::Point;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub id: Option<iced::window::Id>,
    pub cursor_position: Point,
    pub is_dragging: bool,
    pub last_drag_move: Option<Instant>,
    pub is_focused: bool,
    pub unfocused_frame_ticks: u32,
    pub bounds_dirty: bool,
    pub ghost_passthrough_active: bool,
    pub is_alt_pressed: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            id: None,
            cursor_position: Point::new(-1000.0, -1000.0),
            is_dragging: false,
            last_drag_move: None,
            is_focused: true,
            unfocused_frame_ticks: 0,
            bounds_dirty: false,
            ghost_passthrough_active: false,
            is_alt_pressed: false,
        }
    }
}
