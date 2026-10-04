/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Containers, Cards, Badges & Overlays
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::container};

// Menu Dropdown Container (Titlebar Logo Dropdown)
pub fn menu_dropdown_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_MODAL_BG)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 8.0,
                bottom_left: 8.0,
            },
            width: 1.0,
            color: COLOR_BORDER,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.5, 0.5),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

// Tag Input Box (Outer container styled like input)
pub fn tag_input_box_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x2a, 0x2a, 0x2a))),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgb8(0x3f, 0x3f, 0x3f),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

// Tag Chip container
pub fn tag_chip_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(0x3a, 0x3a, 0x3a))),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgb8(0x4f, 0x4f, 0x4f),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

// Audio Menu Card Style
pub fn audio_menu_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.09, 0.09, 0.11, 0.96))),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.25, 0.25, 0.28, 0.8),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: Vector::new(0.0, -2.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

// Modal Card Style
pub fn modal_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_MODAL_BG)),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.2, 0.2, 0.2),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 25.0,
        },
        ..Default::default()
    }
}

// Modal Overlay Backdrop
pub fn modal_backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
        ..Default::default()
    }
}

// Player Container Style (solid black background scaled with opacity)
pub fn player_container_style(opacity: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, opacity))),
        ..Default::default()
    }
}

// Active Player Focus Ring (3px #42b883 emerald ring overlaid on top of video)
pub fn focus_ring_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            radius: 0.0.into(),
            width: 3.0,
            color: COLOR_PRIMARY,
        },
        ..Default::default()
    }
}

pub fn title_pill_style_with_alpha(alpha: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(
            0.08,
            0.08,
            0.10,
            0.88 * alpha,
        ))),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 14.0,
                bottom_right: 14.0,
                bottom_left: 0.0,
            },
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.55 * alpha),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

pub fn controls_overlay_style_with_alpha(alpha: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(
            0.08,
            0.08,
            0.10,
            0.88 * alpha,
        ))),
        border: Border {
            radius: 14.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.55 * alpha),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

// Drag to Move Overlay Card Style (Alt Drag Overlay)
pub fn drag_overlay_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.80))),
        border: Border {
            radius: 14.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

// Center Floating Notice Style
pub fn notice_pill_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_OVERLAY_DARK)),
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

// Top Scan Toast Banner Style
pub fn scan_toast_banner_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}

// File Picker Side Drawer
pub fn file_picker_drawer_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_DRAWER_BG)),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.27, 0.27, 0.27),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: Vector::new(-4.0, 0.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

// Welcome Card Style
pub fn welcome_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.2, 0.2, 0.2),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    }
}

// Transcript Time Badge Style
pub fn transcript_time_badge_style(is_active: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| {
        if is_active {
            container::Style {
                background: Some(Background::Color(COLOR_PRIMARY)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: Some(Color::from_rgb(0.08, 0.08, 0.08)),
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                },
                text_color: Some(Color::from_rgb(0.65, 0.65, 0.65)),
                ..Default::default()
            }
        }
    }
}

// Transcript Count Pill Badge Style
pub fn transcript_count_badge_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.26, 0.72, 0.51, 0.15))),
        border: Border {
            radius: 12.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.26, 0.72, 0.51, 0.35),
        },
        text_color: Some(COLOR_PRIMARY),
        ..Default::default()
    }
}
