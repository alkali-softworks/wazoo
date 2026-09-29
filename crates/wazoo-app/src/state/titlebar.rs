/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Titlebar & Dropdown Menu State
 */

use crate::app::{DROPDOWN_MENU_SLIDE_TICKS, TITLEBAR_FADE_TICKS, TITLEBAR_SLIDE_TICKS};
use iced::Point;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct TitlebarState {
    pub show: bool,
    pub hide_ticks: usize,
    pub hover_ticks: usize,
    pub slide_ticks: usize,
    pub show_dropdown_menu: bool,
    pub dropdown_menu_slide_ticks: usize,
    pub press_origin: Option<Point>,
    pub drag_pending: bool,
    pub last_click: Option<Instant>,
}

impl TitlebarState {
    pub fn slide_progress(&self) -> f32 {
        if self.show {
            (self.slide_ticks as f32 / TITLEBAR_SLIDE_TICKS as f32).min(1.0)
        } else if self.hide_ticks <= TITLEBAR_FADE_TICKS {
            (self.hide_ticks as f32 / TITLEBAR_FADE_TICKS as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn dropdown_menu_slide_progress(&self) -> f32 {
        if self.show_dropdown_menu {
            (self.dropdown_menu_slide_ticks as f32 / DROPDOWN_MENU_SLIDE_TICKS as f32).min(1.0)
        } else {
            0.0
        }
    }

    pub fn alpha(&self) -> f32 {
        if self.show {
            1.0
        } else if self.hide_ticks > 0 && self.hide_ticks <= TITLEBAR_FADE_TICKS {
            self.hide_ticks as f32 / TITLEBAR_FADE_TICKS as f32
        } else {
            0.0
        }
    }
}
