/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Buttons & Interactive Controls
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::button};

// Titlebar Badge Style (Wazoo pill button)
pub fn titlebar_badge_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => pal.badge_hover,
            _ => pal.badge_bg,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.badge_text,
            border: Border {
                radius: iced::border::Radius {
                    top_left: 0.0,
                    top_right: pal.radius(16.0),
                    bottom_right: pal.radius(16.0),
                    bottom_left: 0.0,
                },
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn titlebar_badge_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = titlebar_badge_style(pal);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Video Player Title Pill Button Style
pub fn title_pill_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, shadow_color) = match status {
            button::Status::Hovered => (
                with_alpha(pal.hud_hover, (pal.hud_opacity + 0.07).min(0.98)),
                Color::from_rgba(0.0, 0.0, 0.0, 0.65),
            ),
            button::Status::Pressed => (
                with_alpha(pal.hud_pressed, (pal.hud_opacity + 0.10).min(0.99)),
                Color::from_rgba(0.0, 0.0, 0.0, 0.70),
            ),
            _ => (
                with_alpha(pal.hud_bg, pal.hud_opacity),
                Color::from_rgba(0.0, 0.0, 0.0, 0.55),
            ),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.hud_text,
            border: Border {
                radius: iced::border::Radius {
                    top_left: 0.0,
                    top_right: pal.radius(14.0),
                    bottom_right: pal.radius(14.0),
                    bottom_left: 0.0,
                },
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow {
                color: shadow_color,
                offset: Vector::new(0.0, 4.0),
                blur_radius: 18.0,
            },
            ..Default::default()
        }
    }
}

pub fn title_pill_button_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = title_pill_button_style(pal);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            shadow: Shadow {
                color: with_alpha(base.shadow.color, alpha),
                ..base.shadow
            },
            ..base
        }
    }
}

pub fn title_pill_genre_button_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, border_width, border_color) = match status {
            button::Status::Hovered => (
                with_alpha(pal.hud_text, 0.24 * alpha),
                1.0,
                with_alpha(pal.hud_text, 0.35 * alpha),
            ),
            button::Status::Pressed => (
                with_alpha(pal.hud_text, 0.32 * alpha),
                1.0,
                with_alpha(pal.hud_text, 0.50 * alpha),
            ),
            _ => (
                with_alpha(pal.hud_text, 0.12 * alpha),
                0.0,
                Color::TRANSPARENT,
            ),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: with_alpha(pal.hud_text, alpha),
            border: Border {
                radius: pal.radius(4.0).into(),
                width: border_width,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Titlebar Brand Link Style ("ALKALI" button)
pub fn titlebar_brand_link_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered => (with_alpha(pal.titlebar_text, 0.12), pal.titlebar_text),
            button::Status::Pressed => (
                with_alpha(pal.titlebar_text, 0.20),
                mix_color(pal.titlebar_text, pal.titlebar_bg, 0.10),
            ),
            _ => (Color::TRANSPARENT, pal.titlebar_brand_text),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: iced::border::Radius::from(pal.radius(4.0)),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn titlebar_brand_link_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = titlebar_brand_link_style(pal);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Window Controls (in modals and general UI)
pub fn window_control_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered | button::Status::Pressed => (pal.btn_bg, pal.text),
            _ => (Color::TRANSPARENT, pal.text_muted),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn window_control_button_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered | button::Status::Pressed => (
                with_alpha(pal.titlebar_text, 0.14),
                pal.titlebar_text,
            ),
            _ => (Color::TRANSPARENT, pal.titlebar_icon_color),
        };
        button::Style {
            background: Some(Background::Color(with_alpha(bg, alpha))),
            text_color: with_alpha(text_color, alpha),
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn close_window_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered | button::Status::Pressed => {
                (pal.btn_close_hover, Color::WHITE)
            }
            _ => (Color::TRANSPARENT, pal.text_muted),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn close_window_button_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered | button::Status::Pressed => {
                (pal.btn_close_hover, Color::WHITE)
            }
            _ => (Color::TRANSPARENT, pal.titlebar_icon_color),
        };
        button::Style {
            background: Some(Background::Color(with_alpha(bg, alpha))),
            text_color: with_alpha(text_color, alpha),
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Menu Items (Dropdown and Menu Modal)
pub fn menu_item_style(pal: ThemePalette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => pal.btn_bg,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(4.0).into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Folder Group Header Button (File Drawer collapsible header)
pub fn folder_group_header_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => pal.btn_hover,
            button::Status::Pressed => pal.border,
            _ => pal.card_bg,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(5.0).into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Player Control Buttons (Play, Pause, Next, Mute)
pub fn player_control_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => with_alpha(pal.hud_text, 0.2),
            button::Status::Pressed => with_alpha(pal.hud_text, 0.3),
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.hud_text,
            border: Border {
                radius: pal.radius(4.0).into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn player_control_button_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = player_control_button_style(pal);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// CC Button Style
pub fn cc_button_style(
    pal: ThemePalette,
    is_enabled: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => with_alpha(pal.hud_text, 0.2),
            button::Status::Pressed => with_alpha(pal.hud_text, 0.3),
            _ => Color::TRANSPARENT,
        };
        let text_color = if is_enabled {
            pal.primary
        } else {
            with_alpha(pal.hud_text, 0.55)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: pal.radius(4.0).into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn cc_button_style_with_alpha(
    pal: ThemePalette,
    is_enabled: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let base_fn = cc_button_style(pal, is_enabled);
    move |theme, status| {
        let base = base_fn(theme, status);
        button::Style {
            background: base.background.map(|bg| background_with_alpha(bg, alpha)),
            text_color: with_alpha(base.text_color, alpha),
            ..base
        }
    }
}

// Action Button (standard UI button)
pub fn action_button_style(pal: ThemePalette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => pal.btn_hover,
            button::Status::Pressed => pal.btn_pressed,
            button::Status::Disabled => with_alpha(pal.btn_bg, 0.5),
            _ => pal.btn_bg,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(4.0).into(),
                width: 1.0,
                color: pal.border,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Folder Chip in Search Modal
pub fn folder_chip_style(
    pal: ThemePalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, border_color, border_width) = if is_active {
            let (bg, border) = match status {
                button::Status::Hovered => (pal.btn_bg, pal.primary_border),
                button::Status::Pressed => (pal.btn_pressed, pal.primary),
                _ => (pal.card_bg, pal.primary),
            };
            (bg, border, 1.5)
        } else {
            let (bg, border) = match status {
                button::Status::Hovered => (pal.btn_bg, pal.border_hover),
                button::Status::Pressed => (pal.btn_pressed, pal.border),
                _ => (pal.card_bg, pal.border),
            };
            (bg, border, 1.0)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(20.0).into(),
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
    pal: ThemePalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color, border_color) = if is_active {
            let bg = mix_color(pal.modal_bg, pal.primary, 0.16);
            (bg, readable_accent_on(pal, bg), pal.primary)
        } else {
            match status {
                button::Status::Hovered => (pal.btn_hover, pal.text, pal.border),
                button::Status::Pressed => (pal.btn_pressed, pal.text, pal.border),
                _ => (Color::TRANSPARENT, pal.text_muted, Color::TRANSPARENT),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: pal.radius(6.0).into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Reusable Settings Toggle Card Button Style
pub fn settings_toggle_card_style(
    pal: ThemePalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = if is_active {
            match status {
                button::Status::Hovered => mix_color(pal.btn_bg, pal.primary, 0.22),
                button::Status::Pressed => mix_color(pal.btn_bg, pal.primary, 0.30),
                _ => mix_color(pal.btn_bg, pal.primary, 0.14),
            }
        } else {
            match status {
                button::Status::Hovered => pal.btn_hover,
                button::Status::Pressed => pal.btn_pressed,
                _ => pal.btn_bg,
            }
        };
        let text_color = if contrast_ratio(pal.text, bg) >= 4.5 {
            pal.text
        } else {
            contrast_text(bg)
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: pal.radius(8.0).into(),
                width: 1.0,
                color: if is_active { pal.primary } else { pal.border },
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

/// Small Reset Button in Settings Modal (↺)
pub fn settings_reset_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color, border_color) = match status {
            button::Status::Hovered => (
                pal.btn_hover,
                readable_accent_on(pal, pal.btn_hover),
                pal.primary_border,
            ),
            button::Status::Pressed => (
                pal.btn_bg,
                readable_accent_on(pal, pal.btn_bg),
                pal.primary,
            ),
            _ => (pal.card_bg, pal.text_dim, pal.border),
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: pal.radius(4.0).into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Tag Delete (✕) button
pub fn tag_delete_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let color = match status {
            button::Status::Hovered => pal.text,
            _ => pal.text_muted,
        };
        button::Style {
            background: None,
            text_color: color,
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Search Input Clear (✕) button
pub fn search_clear_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, text_color) = match status {
            button::Status::Hovered => (
                Some(Background::Color(with_alpha(pal.text, 0.15))),
                pal.text,
            ),
            button::Status::Pressed => (
                Some(Background::Color(with_alpha(pal.text, 0.25))),
                pal.text,
            ),
            _ => (None, pal.text_muted),
        };
        button::Style {
            background: bg,
            text_color,
            border: Border {
                radius: pal.radius(4.0).into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Primary Accent Search Button
pub fn search_button_style(pal: ThemePalette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => pal.primary_hover,
            button::Status::Pressed => pal.primary_pressed,
            _ => pal.primary,
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
            text_color: pal.primary_on_text,
            border: Border {
                radius: pal.radius(6.0).into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow,
            ..Default::default()
        }
    }
}

pub fn audio_track_button_style_with_alpha(
    pal: ThemePalette,
    is_open: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, border_color) = if is_open {
            (
                mix_color(pal.hud_bg, pal.primary, 0.18),
                pal.primary,
            )
        } else {
            match status {
                button::Status::Hovered => (
                    with_alpha(pal.hud_hover, 0.9),
                    with_alpha(pal.hud_text, 0.5),
                ),
                button::Status::Pressed => (
                    mix_color(pal.hud_bg, pal.primary, 0.22),
                    pal.primary,
                ),
                _ => (
                    with_alpha(pal.hud_bg, 0.75),
                    with_alpha(pal.hud_text, 0.30),
                ),
            }
        };
        button::Style {
            background: Some(Background::Color(with_alpha(bg, alpha))),
            text_color: with_alpha(pal.hud_text, alpha),
            border: Border {
                radius: pal.radius(4.0).into(),
                width: 1.0,
                color: with_alpha(border_color, alpha),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn transcript_track_button_style(
    pal: ThemePalette,
    is_open: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, border_color) = if is_open {
            (
                mix_color(pal.card_bg, pal.primary, 0.16),
                pal.primary,
            )
        } else {
            match status {
                button::Status::Hovered => (pal.btn_hover, pal.border_hover),
                button::Status::Pressed => (
                    mix_color(pal.card_bg, pal.primary, 0.20),
                    pal.primary,
                ),
                _ => (pal.card_bg, pal.border),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(6.0).into(),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

pub fn audio_menu_item_style(
    pal: ThemePalette,
    is_selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        if is_selected {
            let bg = match status {
                button::Status::Hovered => mix_color(pal.hud_bg, pal.primary, 0.35),
                _ => mix_color(pal.hud_bg, pal.primary, 0.20),
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: readable_hud_accent_on(pal, bg),
                border: Border {
                    radius: pal.radius(4.0).into(),
                    width: 1.0,
                    color: with_alpha(pal.primary, 0.5),
                },
                shadow: Shadow::default(),
                ..Default::default()
            }
        } else {
            let (bg, text_color) = match status {
                button::Status::Hovered => (
                    Background::Color(with_alpha(pal.hud_hover, 0.95)),
                    pal.hud_text,
                ),
                button::Status::Pressed => (
                    Background::Color(with_alpha(pal.hud_pressed, 0.95)),
                    readable_hud_accent_on(pal, pal.hud_pressed),
                ),
                _ => (
                    Background::Color(Color::TRANSPARENT),
                    with_alpha(pal.hud_text, 0.85),
                ),
            };
            button::Style {
                background: Some(bg),
                text_color,
                border: Border {
                    radius: pal.radius(4.0).into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                ..Default::default()
            }
        }
    }
}

// Bookmark Item Button Style
pub fn bookmark_item_button_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => pal.btn_bg,
            button::Status::Pressed => pal.btn_pressed,
            _ => pal.card_bg,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: pal.text,
            border: Border {
                radius: pal.radius(6.0).into(),
                width: 1.0,
                color: with_alpha(pal.border, 0.7),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}

// Primary Action Button Style
pub fn primary_button_style(pal: ThemePalette) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => pal.primary_hover,
            button::Status::Pressed => pal.primary_pressed,
            _ => pal.primary,
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
            text_color: pal.primary_on_text,
            border: Border {
                radius: pal.radius(6.0).into(),
                ..Default::default()
            },
            shadow,
            ..Default::default()
        }
    }
}

pub fn transcript_button_style_with_alpha(
    pal: ThemePalette,
    is_open: bool,
    alpha: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    cc_button_style_with_alpha(pal, is_open, alpha)
}

// Transcript Cue Row Button Style
pub fn transcript_cue_button_style(
    pal: ThemePalette,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        if is_active {
            let bg = match status {
                button::Status::Hovered => with_alpha(pal.primary, 0.28),
                button::Status::Pressed => with_alpha(pal.primary, 0.35),
                _ => with_alpha(pal.primary, 0.18),
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: pal.text,
                border: Border {
                    radius: pal.radius(6.0).into(),
                    width: 1.0,
                    color: with_alpha(pal.primary, 0.6),
                },
                shadow: Shadow {
                    color: with_alpha(pal.primary, 0.15),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                ..Default::default()
            }
        } else {
            let bg = match status {
                button::Status::Hovered => with_alpha(pal.text, 0.08),
                button::Status::Pressed => with_alpha(pal.text, 0.14),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: pal.text_dim,
                border: Border {
                    radius: pal.radius(6.0).into(),
                    ..Default::default()
                },
                shadow: Shadow::default(),
                ..Default::default()
            }
        }
    }
}
