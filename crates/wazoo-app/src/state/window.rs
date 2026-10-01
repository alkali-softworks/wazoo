/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Lifecycle & Cursor State
 */

use iced::Point;
use std::time::{Duration, Instant};

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
    pub last_resize_time: Option<Instant>,
}

impl WindowState {
    /// Window resize debounce duration: video frame rendering is suspended while
    /// the window is actively being resized, ensuring stutter-free border dragging.
    pub const RESIZE_DEBOUNCE_DURATION: Duration = Duration::from_millis(200);

    pub fn is_resizing(&self) -> bool {
        self.last_resize_time
            .map(|t| t.elapsed() < Self::RESIZE_DEBOUNCE_DURATION)
            .unwrap_or(false)
    }

    pub fn mark_resized(&mut self) {
        self.last_resize_time = Some(Instant::now());
    }
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
            last_resize_time: None,
        }
    }
}
