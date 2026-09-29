/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Colors, Fonts & Palettes
 */

use iced::{
    Background, Color, Element, Pixels,
    widget::{Stack, container, text},
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
