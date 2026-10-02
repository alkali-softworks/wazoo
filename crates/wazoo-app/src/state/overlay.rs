/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Player Overlay, Toasts & Focus State
 */

use crate::app::{PLAYER_OVERLAY_FADE_TICKS, PLAYER_OVERLAY_HIDE_TICKS};

#[derive(Debug, Clone, Default)]
pub struct OverlayState {
    pub ticks: usize,
    pub fade_in_ticks: usize,
    pub title_pill_ticks: usize,
    pub focus_border_ticks: usize,
    pub toast_message: Option<String>,
    pub toast_time_remaining: usize,
    pub spinner_ticks: u32,
}

impl OverlayState {
    pub fn spinner_angle(&self) -> f32 {
        ((self.spinner_ticks * 12) % 360) as f32
    }

    /// Computes the active frame index for the color-static TV static animation.
    /// Advances every 2 ticks (~30ms) at 60Hz to match the original GIF animation timing.
    pub fn color_static_frame_index(&self, total_frames: usize) -> usize {
        if total_frames == 0 {
            0
        } else {
            (self.spinner_ticks as usize / 2) % total_frames
        }
    }
    pub fn trigger(&mut self) {
        if self.ticks == 0 {
            self.fade_in_ticks = 0;
        } else if self.ticks < PLAYER_OVERLAY_FADE_TICKS {
            self.fade_in_ticks = self.ticks;
        }
        self.ticks = PLAYER_OVERLAY_HIDE_TICKS;
    }

    pub fn alpha(&self) -> f32 {
        let fade_in = if self.fade_in_ticks < PLAYER_OVERLAY_FADE_TICKS {
            self.fade_in_ticks as f32 / PLAYER_OVERLAY_FADE_TICKS as f32
        } else {
            1.0
        };

        let fade_out = if self.ticks == 0 {
            0.0
        } else if self.ticks < PLAYER_OVERLAY_FADE_TICKS {
            self.ticks as f32 / PLAYER_OVERLAY_FADE_TICKS as f32
        } else {
            1.0
        };

        (fade_in * fade_out).clamp(0.0, 1.0)
    }

    pub fn show_toast(&mut self, msg: impl Into<String>, duration_secs: usize) {
        self.toast_message = Some(msg.into());
        // Toast duration in seconds (typically DEFAULT_TOAST_SECS = 2, LONG_TOAST_SECS = 3).
        // If a caller mistakenly passed 60-FPS frame ticks (> 10), convert to seconds.
        self.toast_time_remaining = if duration_secs > 10 {
            (duration_secs / 60).clamp(1, 4)
        } else {
            duration_secs.clamp(1, 10)
        };
    }

    pub fn clear_toast(&mut self) {
        self.toast_message = None;
        self.toast_time_remaining = 0;
    }
}
