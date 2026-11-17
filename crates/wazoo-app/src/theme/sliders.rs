/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Sliders & Scrubbers
 */

use super::colors::*;
use iced::{Background, Border, Color, Theme, widget::slider};

pub fn progress_slider_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme, _status| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(with_alpha(pal.primary, alpha)),
                Background::Color(with_alpha(pal.track_bg, alpha)),
            ),
            width: 24.0,
            border: Border {
                radius: pal.radius(3.0).into(),
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

// Settings Modal Slider Style (with visible handle thumb and accent rail)
pub fn settings_slider_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme: &Theme, status: slider::Status| {
        let handle_color = match status {
            slider::Status::Hovered | slider::Status::Dragged => pal.text,
            _ => pal.text_dim,
        };
        slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(pal.primary),
                    Background::Color(pal.border),
                ),
                width: 6.0,
                border: Border {
                    radius: pal.radius(3.0).into(),
                    ..Default::default()
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 6.0 },
                background: Background::Color(handle_color),
                border_width: 2.0,
                border_color: pal.primary,
            },
        }
    }
}

// Volume Bar Slider Style
pub fn volume_slider_style(pal: ThemePalette) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme: &Theme, _status: slider::Status| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(pal.primary),
                Background::Color(pal.track_bg),
            ),
            width: 8.0,
            border: Border {
                radius: pal.radius(3.0).into(),
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
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, slider::Status) -> slider::Style {
    move |_theme, _status| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(with_alpha(pal.primary, alpha)),
                Background::Color(with_alpha(pal.track_bg, alpha)),
            ),
            width: 8.0,
            border: Border {
                radius: pal.radius(3.0).into(),
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
