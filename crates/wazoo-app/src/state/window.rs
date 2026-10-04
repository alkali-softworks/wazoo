/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Lifecycle & Cursor State
 */

use iced::Point;
use std::time::{Duration, Instant};

/// Tracks host window ID, geometry state, cursor coordinates, and input interaction flags.
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Active iced window ID assigned by the OS shell upon creation.
    pub id: Option<iced::window::Id>,
    /// Last reported physical cursor position in window coordinates.
    pub cursor_position: Point,
    /// Indicates whether the window titlebar is actively being dragged by the mouse.
    pub is_dragging: bool,
    /// Timestamp of the last drag move event used to rate-limit window move calls.
    pub last_drag_move: Option<Instant>,
    /// Whether the operating system window currently holds desktop focus.
    pub is_focused: bool,
    /// Frame counter used to throttle playback rendering when window is unfocused.
    pub unfocused_frame_ticks: u32,
    /// Set when window size or position changes; debounces saving bounds to disk.
    pub bounds_dirty: bool,
    /// When true, mouse clicks pass through the window directly to the underlying OS desktop.
    pub clickthru: bool,
    /// Whether the Alt modifier key is currently held down.
    pub is_alt_pressed: bool,
    /// Timestamp of the most recent window resize event.
    pub last_resize_time: Option<Instant>,
    /// Whether the application window is currently in fullscreen display mode.
    pub is_fullscreen: bool,
    /// Whether the application window is currently maximized.
    pub is_maximized: bool,
}

impl WindowState {
    /// Window resize debounce duration: video frame rendering is suspended while
    /// the window is actively being resized, ensuring stutter-free border dragging.
    pub const RESIZE_DEBOUNCE_DURATION: Duration = Duration::from_millis(200);

    /// Returns true if the window was resized within the debounce cooldown duration.
    #[inline]
    pub fn is_resizing(&self) -> bool {
        self.last_resize_time
            .map(|t| t.elapsed() < Self::RESIZE_DEBOUNCE_DURATION)
            .unwrap_or(false)
    }

    /// Records the current timestamp as the most recent window resize event.
    #[inline]
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
            clickthru: false,
            is_alt_pressed: false,
            last_resize_time: None,
            is_fullscreen: false,
            is_maximized: false,
        }
    }
}
