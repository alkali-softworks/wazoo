/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Sliders & Scrubbers
 */

use super::colors::*;
use iced::{
    Background, Border, Color, Theme,
    widget::slider,
};

// Progress Bar Slider Style (Vue emerald green #42b883)
#[allow(dead_code)]
pub fn progress_slider_style(theme: &Theme, status: slider::Status) -> slider::Style {
    progress_slider_style_with_alpha(1.0)(theme, status)
}

pub fn progress_slider_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme, _status| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(with_alpha(COLOR_PRIMARY, alpha)),
                Background::Color(with_alpha(COLOR_TRACK_BG, alpha)),
            ),
            width: 24.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 0.0 },
            background: Background::Color(Color::TRANSPARENT),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

// Settings Modal Slider Style (with visible handle thumb and emerald rail)
pub fn settings_slider_style(_theme: &Theme, status: slider::Status) -> slider::Style {
    let handle_color = match status {
        slider::Status::Hovered | slider::Status::Dragged => Color::WHITE,
        _ => Color::from_rgb(0.9, 0.9, 0.9),
    };
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(COLOR_PRIMARY),
                Background::Color(Color::from_rgb(0.24, 0.24, 0.24)),
            ),
            width: 6.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 6.0 },
            background: Background::Color(handle_color),
            border_width: 2.0,
            border_color: COLOR_PRIMARY,
        },
    }
}

// Volume Bar Slider Style (Vue emerald green #42b883)
pub fn volume_slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(COLOR_PRIMARY),
                Background::Color(COLOR_TRACK_BG),
            ),
            width: 8.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 0.0 },
            background: Background::Color(Color::TRANSPARENT),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub fn volume_slider_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme, _status| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(with_alpha(COLOR_PRIMARY, alpha)),
                Background::Color(with_alpha(COLOR_TRACK_BG, alpha)),
            ),
            width: 8.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 0.0 },
            background: Background::Color(Color::TRANSPARENT),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}
