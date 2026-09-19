/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Application Design System & Theme
 *
 * Defines colors, container styles, button appearances, inputs, modal backdrops,
 * focus rings, and visual styling rules for the dark ambient aesthetic.
 */

use iced::{
    Background, Border, Color, Element, Pixels, Shadow, Theme, Vector,
    overlay::menu,
    widget::{Stack, button, container, slider, text, text_input},
};

pub const COLOR_PRIMARY: Color = Color::from_rgb(0.259, 0.722, 0.514); // #42b883 Emerald
pub const COLOR_TITLEBAR_BG: Color = Color::from_rgb(0.122, 0.035, 0.184); // #1f092f Deep plum
pub const COLOR_BADGE_BG: Color = Color::BLACK;
pub const COLOR_MODAL_BG: Color = Color::from_rgb(0.122, 0.122, 0.122); // #1f1f1f
pub const COLOR_CARD_BG: Color = Color::from_rgb(0.165, 0.165, 0.165); // #2a2a2a
pub const COLOR_BORDER: Color = Color::from_rgb(0.247, 0.247, 0.247); // #3f3f3f
pub const COLOR_TEXT_MUTED: Color = Color::from_rgb(0.533, 0.533, 0.533); // #888888
pub const COLOR_TEXT_DIM: Color = Color::from_rgb(0.867, 0.867, 0.867); // #dddddd
pub const COLOR_BTN_BG: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333
pub const COLOR_BTN_HOVER: Color = Color::from_rgb(0.267, 0.267, 0.267); // #444444
pub const COLOR_BTN_CLOSE_HOVER: Color = Color::from_rgb(0.910, 0.067, 0.137); // #e81123
pub const COLOR_PRIMARY_BORDER: Color = Color::from_rgb(0.35, 0.80, 0.60);
pub const COLOR_TRACK_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.2);
pub const COLOR_OVERLAY_DARK: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.6);
pub const COLOR_DRAWER_BG: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333

pub const FONT_BOLD: iced::Font = iced::Font {
    family: iced::font::Family::SansSerif,
    weight: iced::font::Weight::Bold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

#[inline]
pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
}

#[inline]
pub fn background_with_alpha(bg: Background, alpha: f32) -> Background {
    match bg {
        Background::Color(c) => Background::Color(with_alpha(c, alpha)),
        Background::Gradient(g) => Background::Gradient(g),
    }
}

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
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
        ..Default::default()
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
// Text Input Style
pub fn dark_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => COLOR_PRIMARY,
        text_input::Status::Hovered => Color::from_rgb(0.35, 0.35, 0.35),
        _ => Color::from_rgb(0.27, 0.27, 0.27),
    };
    text_input::Style {
        background: Background::Color(Color::from_rgb(0.165, 0.165, 0.165)),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
        icon: COLOR_TEXT_MUTED,
        placeholder: COLOR_TEXT_MUTED,
        value: Color::WHITE,
        selection: Color::from_rgba(0.26, 0.72, 0.51, 0.4),
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

// Transparent Text Input (inside tag input box)
pub fn transparent_input_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: Color::from_rgb8(0x71, 0x71, 0x7a),
        placeholder: Color::from_rgb8(0x71, 0x71, 0x7a),
        value: Color::WHITE,
        selection: Color::from_rgba(0.26, 0.72, 0.51, 0.4),
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

pub fn dark_pick_list_style(
    _theme: &Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let border_color = match status {
        iced::widget::pick_list::Status::Opened { .. } => COLOR_PRIMARY,
        iced::widget::pick_list::Status::Hovered => Color::from_rgb(0.35, 0.35, 0.35),
        _ => Color::from_rgb(0.27, 0.27, 0.27),
    };
    iced::widget::pick_list::Style {
        text_color: Color::WHITE,
        placeholder_color: COLOR_TEXT_MUTED,
        handle_color: Color::from_rgb(0.7, 0.7, 0.7),
        background: Background::Color(Color::from_rgb(0.165, 0.165, 0.165)),
        border: Border {
            radius: 4.0.into(),
            width: 1.0,
            color: border_color,
        },
    }
}

pub fn dark_pick_list_menu_style(_theme: &Theme) -> menu::Style {
    menu::Style {
        background: Background::Color(COLOR_MODAL_BG),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: COLOR_BORDER,
        },
        text_color: Color::WHITE,
        selected_text_color: Color::from_rgb(0.06, 0.06, 0.06),
        selected_background: Background::Color(COLOR_PRIMARY),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
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

/// Renders bold white text with a significant, diffuse multi-layer ambient shadow
/// for high readability across varying bright and dark backgrounds.
pub fn diffuse_shadowed_text<'a, Message: 'a>(
    content: impl Into<std::borrow::Cow<'a, str>>,
    size: impl Into<Pixels>,
    alpha: f32,
) -> Element<'a, Message> {
    let s: std::borrow::Cow<'a, str> = content.into();
    let sz: Pixels = size.into();
    let text_val: String = s.into_owned();

    const MAX_SPREAD: f32 = 3.0;

    // Radially symmetric samples producing a smooth, diffuse Gaussian-like ambient halo with zero directional bias:
    // (dx, dy, base_alpha)
    const SHADOW_SAMPLES: &[(f32, f32, f32)] = &[
        // Core ring (r ≈ 1.0px)
        (0.0, 1.0, 0.20),
        (0.0, -1.0, 0.20),
        (-1.0, 0.0, 0.20),
        (1.0, 0.0, 0.20),
        (-0.7, 0.7, 0.18),
        (0.7, 0.7, 0.18),
        (-0.7, -0.7, 0.18),
        (0.7, -0.7, 0.18),
        // Mid ring (r ≈ 2.0px)
        (0.0, 2.0, 0.12),
        (0.0, -2.0, 0.12),
        (-2.0, 0.0, 0.12),
        (2.0, 0.0, 0.12),
        (-1.4, 1.4, 0.10),
        (1.4, 1.4, 0.10),
        (-1.4, -1.4, 0.10),
        (1.4, -1.4, 0.10),
        // Outer diffuse halo (r ≈ 3.0px)
        (0.0, 3.0, 0.06),
        (0.0, -3.0, 0.06),
        (-3.0, 0.0, 0.06),
        (3.0, 0.0, 0.06),
        (-2.1, 2.1, 0.05),
        (2.1, 2.1, 0.05),
        (-2.1, -2.1, 0.05),
        (2.1, -2.1, 0.05),
    ];

    let mut stack = Stack::new();

    // Push diffuse shadow layers underneath
    for &(dx, dy, sample_alpha) in SHADOW_SAMPLES {
        let layer = container(
            text(text_val.clone())
                .size(sz)
                .font(FONT_BOLD)
                .color(Color::from_rgba(0.0, 0.0, 0.0, sample_alpha * alpha)),
        )
        .padding(iced::Padding {
            top: MAX_SPREAD + dy,
            left: MAX_SPREAD + dx,
            right: MAX_SPREAD - dx,
            bottom: MAX_SPREAD - dy,
        });

        stack = stack.push(layer);
    }

    // Foreground bold white text centered in the same bounding envelope
    let foreground = container(
        text(text_val)
            .size(sz)
            .font(FONT_BOLD)
            .color(with_alpha(Color::WHITE, alpha)),
    )
    .padding(iced::Padding {
        top: MAX_SPREAD,
        left: MAX_SPREAD,
        right: MAX_SPREAD,
        bottom: MAX_SPREAD,
    });

    stack = stack.push(foreground);

    stack.into()
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

// Title Pill Style (top-left attached)
#[allow(dead_code)]
pub fn title_pill_style(theme: &Theme) -> container::Style {
    title_pill_style_with_alpha(1.0)(theme)
}

pub fn title_pill_style_with_alpha(alpha: f32) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(with_alpha(COLOR_OVERLAY_DARK, alpha))),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 10.0,
                bottom_right: 10.0,
                bottom_left: 0.0,
            },
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

// Controls Bottom Overlay Style
#[allow(dead_code)]
pub fn controls_overlay_style(theme: &Theme) -> container::Style {
    controls_overlay_style_with_alpha(1.0)(theme)
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
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.12 * alpha),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.55 * alpha),
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
