/*!
 * ==============================================================================
 * TUTORIAL LESSON: Styling & Design Systems in Iced 0.14
 * ==============================================================================
 *
 * In Iced 0.14, styling is declarative and type-safe.
 * Instead of string-based CSS stylesheets with class names, Iced uses Rust closures
 * and functions that take the current `&Theme` (and widget status like `button::Status`)
 * and return strongly-typed style structs like `button::Style` or `container::Style`.
 *
 * Key Advantages of Iced Styling in Rust:
 * 1. Zero runtime CSS parsing overhead — everything is compiled to native code.
 * 2. Type safety: compiler guarantees colors, borders, and shadows are valid.
 * 3. Dynamic theming: styles automatically re-evaluate when the active Theme changes.
 * 4. Micro-interactions: easy pattern matching on `Hovered`, `Pressed`, `Disabled`.
 */

use iced::{
    Background, Border, Color, Shadow, Theme, Vector,
    widget::{button, container},
};

// ==============================================================================
// Brand Color Constants
// ==============================================================================
// In Rust, `const` defines values computed at compile time.
// `Color::from_rgb` takes red, green, and blue components in the 0.0 to 1.0 range.
pub const COLOR_ACCENT: Color = Color::from_rgb(0.26, 0.72, 0.51); // #42b883 Emerald (Wazoo brand)
pub const COLOR_ACCENT_HOVER: Color = Color::from_rgb(0.35, 0.82, 0.60);
pub const COLOR_CARD_BG: Color = Color::from_rgb(0.13, 0.14, 0.17);
pub const COLOR_CARD_BORDER: Color = Color::from_rgb(0.22, 0.23, 0.27);
pub const COLOR_TEXT_MUTED: Color = Color::from_rgb(0.60, 0.62, 0.68);
pub const COLOR_TEXT_DIM: Color = Color::from_rgb(0.40, 0.42, 0.48);
pub const COLOR_DANGER: Color = Color::from_rgb(0.92, 0.28, 0.32);
pub const COLOR_DANGER_HOVER: Color = Color::from_rgb(1.00, 0.38, 0.42);

// ==============================================================================
// Container Styles
// ==============================================================================

/// A sleek modern card style with rounded corners and a subtle border.
///
/// RUST CONCEPT: Function Signature as a Callback
/// This function matches the signature `Fn(&Theme) -> container::Style`,
/// so it can be passed directly to `.style(style::card_style)`!
pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_CARD_BG)),
        border: Border {
            radius: iced::border::Radius::from(8.0),
            width: 1.0,
            color: COLOR_CARD_BORDER,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

/// A glowing highlight card style used for code snippets or tips.
pub fn highlight_box_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.26, 0.72, 0.51, 0.08))),
        border: Border {
            radius: iced::border::Radius::from(6.0),
            width: 1.0,
            color: Color::from_rgba(0.26, 0.72, 0.51, 0.35),
        },
        ..Default::default()
    }
}

/// A badge or pill style for small tag indicators.
pub fn badge_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.26, 0.72, 0.51, 0.15))),
        border: Border {
            radius: iced::border::Radius::from(12.0),
            width: 1.0,
            color: COLOR_ACCENT,
        },
        ..Default::default()
    }
}

// ==============================================================================
// Button Styles
// ==============================================================================

/// Primary button style with vibrant accent color and interactive hover/press states.
///
/// RUST CONCEPT: Pattern Matching on Enums (`match status`)
/// `button::Status` is an enum with variants: `Active`, `Hovered`, `Pressed`, `Disabled`.
/// Using exhaustive pattern matching, Rust guarantees at compile time that every possible
/// button state is explicitly handled!
pub fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (COLOR_ACCENT_HOVER, Color::BLACK),
        button::Status::Pressed => (COLOR_ACCENT, Color::BLACK),
        button::Status::Disabled => (Color::from_rgba(0.3, 0.3, 0.3, 0.5), COLOR_TEXT_MUTED),
        button::Status::Active => (COLOR_ACCENT, Color::BLACK),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: iced::border::Radius::from(6.0),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Secondary button style with transparent background and subtle border.
pub fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color, border_color) = match status {
        button::Status::Hovered => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            Color::WHITE,
            Color::from_rgb(0.45, 0.45, 0.50),
        ),
        button::Status::Pressed => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.14),
            Color::WHITE,
            COLOR_ACCENT,
        ),
        button::Status::Disabled => (
            Color::TRANSPARENT,
            COLOR_TEXT_DIM,
            Color::from_rgba(0.3, 0.3, 0.3, 0.3),
        ),
        button::Status::Active => (
            Color::from_rgba(1.0, 1.0, 1.0, 0.04),
            Color::WHITE,
            COLOR_CARD_BORDER,
        ),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: iced::border::Radius::from(6.0),
            width: 1.0,
            color: border_color,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Danger button style (e.g. for deleting records)
pub fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (COLOR_DANGER_HOVER, Color::WHITE),
        button::Status::Pressed => (COLOR_DANGER, Color::WHITE),
        button::Status::Disabled => (Color::from_rgba(0.3, 0.3, 0.3, 0.3), COLOR_TEXT_MUTED),
        button::Status::Active => (COLOR_DANGER, Color::WHITE),
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: iced::border::Radius::from(6.0),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        ..Default::default()
    }
}

/// Navigation Tab Button Style: Highlights active tab with accent border & background
pub fn tab_button_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    // RUST CONCEPT: Closures returning Closures
    // `move |theme, status|` captures the `is_active` boolean from the enclosing function.
    move |_theme, status| {
        let (bg, text_color, border_color) = if is_active {
            (
                Color::from_rgba(0.26, 0.72, 0.51, 0.18),
                COLOR_ACCENT,
                COLOR_ACCENT,
            )
        } else {
            match status {
                button::Status::Hovered => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                    Color::WHITE,
                    COLOR_CARD_BORDER,
                ),
                _ => (Color::TRANSPARENT, COLOR_TEXT_MUTED, Color::TRANSPARENT),
            }
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                radius: iced::border::Radius::from(6.0),
                width: 1.0,
                color: border_color,
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}
