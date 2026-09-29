/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Buttons & Interactive Controls
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::button};

// Titlebar Badge Style (Wazoo pill button)
pub fn titlebar_badge_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgb(0.08, 0.08, 0.08),
        _ => COLOR_BADGE_BG,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 16.0,
                bottom_right: 16.0,
                bottom_left: 0.0,
            },
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn titlebar_badge_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let base = titlebar_badge_style(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Titlebar Brand Link Style ("ALKALI" button)
pub fn titlebar_brand_link_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (Color::from_rgba(1.0, 1.0, 1.0, 0.12), Color::WHITE),
        button::Status::Pressed => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.20),
            Color::from_rgb(0.9, 0.9, 0.9),
        ),
        _ => (Color::TRANSPARENT, Color::from_rgba(1.0, 1.0, 1.0, 0.70)),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: iced::border::Radius::from(4.0),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn titlebar_brand_link_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let base = titlebar_brand_link_style(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Window Controls
pub fn window_control_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (COLOR_BTN_BG, Color::WHITE),
        _ => (Color::TRANSPARENT, COLOR_TEXT_MUTED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn window_control_button_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let base = window_control_button_style(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

pub fn close_window_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered | button::Status::Pressed => (COLOR_BTN_CLOSE_HOVER, Color::WHITE),
        _ => (Color::TRANSPARENT, COLOR_TEXT_MUTED),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn close_window_button_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let base = close_window_button_style(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Menu Items (Dropdown and Menu Modal)
pub fn menu_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgb(0.20, 0.20, 0.20),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Folder Group Header Button (File Drawer collapsible header)
pub fn folder_group_header_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.24, 0.24, 0.24),
        button::Status::Pressed => Color::from_rgb(0.28, 0.28, 0.28),
        _ => Color::from_rgb(0.16, 0.16, 0.16),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Player Control Buttons (Play, Pause, Next, Mute)
pub fn player_control_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.2),
        button::Status::Pressed => Color::from_rgba(1.0, 1.0, 1.0, 0.3),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn player_control_button_style_with_alpha(
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let base = player_control_button_style(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// CC Button Style
pub fn cc_button_style(is_enabled: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.2),
            button::Status::Pressed => Color::from_rgba(1.0, 1.0, 1.0, 0.3),
            _ => Color::TRANSPARENT,
        };
        let text_color = if is_enabled {
            COLOR_PRIMARY
        } else {
            COLOR_TEXT_MUTED
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn cc_button_style_with_alpha(
    is_enabled: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = cc_button_style(is_enabled);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Action Button (standard dark UI button)
pub fn action_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => COLOR_BTN_HOVER,
        button::Status::Pressed => Color::from_rgb(0.15, 0.15, 0.15),
        button::Status::Disabled => Color::from_rgba(0.2, 0.2, 0.2, 0.5),
        _ => COLOR_BTN_BG,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.27, 0.27, 0.27),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Folder Chip in Search Modal
pub fn folder_chip_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, border_color, border_width) = if is_active {
            let (bg, border) = match status {
                button::Status::Hovered => (COLOR_BTN_BG, COLOR_PRIMARY_BORDER),
                button::Status::Pressed => (Color::from_rgb(0.14, 0.14, 0.14), COLOR_PRIMARY),
                _ => (COLOR_CARD_BG, COLOR_PRIMARY),
            };
            (bg, border, 1.5)
        } else {
            let (bg, border) = match status {
                button::Status::Hovered => (COLOR_BTN_BG, Color::from_rgb(0.35, 0.35, 0.35)),
                button::Status::Pressed => (Color::from_rgb(0.14, 0.14, 0.14), COLOR_BORDER),
                _ => (COLOR_CARD_BG, COLOR_BORDER),
            };
            (bg, border, 1.0)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                radius: 20.0.into(),
                width: border_width,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Tab Button in Settings Modal
pub fn settings_tab_button_style(
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color, border_color) = if is_active {
            (
                Color::from_rgba(0.259, 0.722, 0.514, 0.16),
                COLOR_PRIMARY,
                COLOR_PRIMARY,
            )
        } else {
            match status {
                button::Status::Hovered => (COLOR_BTN_HOVER, Color::WHITE, COLOR_BORDER),
                button::Status::Pressed => (
                    Color::from_rgb(0.14, 0.14, 0.14),
                    Color::WHITE,
                    COLOR_BORDER,
                ),
                _ => (Color::TRANSPARENT, COLOR_TEXT_MUTED, Color::TRANSPARENT),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Small Reset Button in Settings Modal (↺)
pub fn settings_reset_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Hovered => (COLOR_BTN_HOVER, COLOR_PRIMARY, COLOR_PRIMARY_BORDER),
        button::Status::Pressed => (COLOR_BTN_BG, COLOR_PRIMARY, COLOR_PRIMARY),
        _ => (COLOR_CARD_BG, COLOR_TEXT_DIM, COLOR_BORDER),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Tag Delete (✕) button
pub fn tag_delete_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let color = match status {
        button::Status::Hovered => Color::WHITE,
        _ => Color::from_rgb8(0xa1, 0xa1, 0xaa),
    };
    button::Style {
        background: None,
        text_color: color,
        border: Border::default(),
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Emerald Green Search Button (matching wazoo-desktop)
pub fn search_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.30, 0.78, 0.56),
        button::Status::Pressed => Color::from_rgb(0.22, 0.65, 0.46),
        _ => COLOR_PRIMARY,
    };
    let shadow = match status {
        button::Status::Hovered => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.50),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        button::Status::Pressed => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        _ => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.40),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 5.0,
        },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::from_rgb(0.06, 0.06, 0.06),
        border: Border {
            radius: 6.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow,
        ..Default::default()
    }
}

pub fn audio_track_button_style_with_alpha(
    is_open: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, border_color) = if is_open {
            (Color::from_rgba(0.15, 0.24, 0.18, 0.95), COLOR_PRIMARY)
        } else {
            match status {
                button::Status::Hovered => (
                    Color::from_rgba(0.24, 0.24, 0.27, 0.9),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.8),
                ),
                button::Status::Pressed => (Color::from_rgba(0.18, 0.24, 0.20, 0.9), COLOR_PRIMARY),
                _ => (
                    Color::from_rgba(0.12, 0.12, 0.14, 0.75),
                    Color::from_rgba(0.35, 0.35, 0.35, 0.5),
                ),
            }
        };
        button::Style {
            background: Some(Background::Color(with_alpha(bg, alpha))),
            text_color: with_alpha(Color::WHITE, alpha),
            border: Border {
                radius: 4.0.into(),
                width: 1.0,
                color: with_alpha(border_color, alpha),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn transcript_track_button_style(
    is_open: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, border_color) = if is_open {
            (Color::from_rgba(0.15, 0.24, 0.18, 0.95), COLOR_PRIMARY)
        } else {
            match status {
                button::Status::Hovered => (
                    Color::from_rgba(0.24, 0.24, 0.27, 0.9),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.8),
                ),
                button::Status::Pressed => (Color::from_rgba(0.18, 0.24, 0.20, 0.9), COLOR_PRIMARY),
                _ => (
                    Color::from_rgba(0.14, 0.14, 0.17, 0.85),
                    Color::from_rgba(0.30, 0.30, 0.35, 0.6),
                ),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn audio_menu_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (
            Background::Color(Color::from_rgba(0.25, 0.25, 0.28, 0.95)),
            Color::WHITE,
        ),
        button::Status::Pressed => (
            Background::Color(Color::from_rgba(0.20, 0.20, 0.22, 0.95)),
            COLOR_PRIMARY,
        ),
        _ => (
            Background::Color(Color::TRANSPARENT),
            Color::from_rgb(0.85, 0.85, 0.85),
        ),
    };
    button::Style {
        background: Some(bg),
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

pub fn audio_menu_selected_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Background::Color(Color::from_rgba(0.26, 0.72, 0.51, 0.35)),
        _ => Background::Color(Color::from_rgba(0.26, 0.72, 0.51, 0.20)),
    };
    button::Style {
        background: Some(bg),
        text_color: COLOR_PRIMARY,
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: Color::from_rgba(0.26, 0.72, 0.51, 0.5),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Bookmark Item Button Style
pub fn bookmark_item_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgba(0.22, 0.22, 0.22, 0.85),
        button::Status::Pressed => Color::from_rgba(0.16, 0.16, 0.16, 0.95),
        _ => Color::from_rgba(0.13, 0.13, 0.13, 0.65),
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

// Primary Action Button Style
pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Color::from_rgb(0.3, 0.78, 0.58),
        button::Status::Pressed => Color::from_rgb(0.2, 0.65, 0.45),
        _ => COLOR_PRIMARY,
    };
    let shadow = match status {
        button::Status::Hovered => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        button::Status::Pressed => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        _ => Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 5.0,
        },
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::from_rgb(0.06, 0.06, 0.06),
        border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        shadow,
        ..Default::default()
    }
}

// Transcript Button Style (Controls bar TX, matches CC button style)
#[allow(dead_code)]
pub fn transcript_button_style(is_open: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    cc_button_style(is_open)
}

pub fn transcript_button_style_with_alpha(
    is_open: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    cc_button_style_with_alpha(is_open, alpha)
}

// Transcript Cue Row Button Style
pub fn transcript_cue_button_style(
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        if is_active {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(0.26, 0.72, 0.51, 0.28),
                button::Status::Pressed => Color::from_rgba(0.26, 0.72, 0.51, 0.35),
                _ => Color::from_rgba(0.26, 0.72, 0.51, 0.18),
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(0.26, 0.72, 0.51, 0.6),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.26, 0.72, 0.51, 0.15),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                ..Default::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                button::Status::Pressed => Color::from_rgba(1.0, 1.0, 1.0, 0.14),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: Color::from_rgb(0.85, 0.85, 0.85),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
                ..Default::default()
            }
        }
    }
}
