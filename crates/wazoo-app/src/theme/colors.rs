/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Colors, Fonts & Palettes
 */

use iced::{
    Background, Color, Element, Pixels,
    widget::{Stack, container, text},
};
use wazoo_core::{ThemeSettings, parse_hex_rgb};

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

#[inline]
pub fn color_from_hex_or(hex: &str, fallback: Color) -> Color {
    if let Some((r, g, b)) = parse_hex_rgb(hex) {
        Color::from_rgb8(r, g, b)
    } else {
        fallback
    }
}

#[inline]
pub fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

#[inline]
pub fn relative_luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

#[inline]
pub fn is_light_color(c: Color) -> bool {
    relative_luminance(c) > 0.5
}

#[inline]
pub fn contrast_text(bg: Color) -> Color {
    if relative_luminance(bg) > 0.48 {
        Color::from_rgb(0.06, 0.06, 0.08)
    } else {
        Color::WHITE
    }
}

#[inline]
pub fn srgb_channel_linear(c: f32) -> f32 {
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub fn wcag_luminance(c: Color) -> f32 {
    0.2126 * srgb_channel_linear(c.r)
        + 0.7152 * srgb_channel_linear(c.g)
        + 0.0722 * srgb_channel_linear(c.b)
}

#[inline]
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = wcag_luminance(a);
    let lb = wcag_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Returns `accent` when it meets WCAG AA contrast (>= 4.5:1) against `bg`,
/// otherwise falls back to `preferred_text` (if readable) or `contrast_text(bg)`.
#[inline]
pub fn readable_accent_with_fallback(accent: Color, preferred_text: Color, bg: Color) -> Color {
    if contrast_ratio(accent, bg) >= 4.5 {
        accent
    } else if contrast_ratio(preferred_text, bg) >= 4.5 {
        preferred_text
    } else {
        contrast_text(bg)
    }
}

/// Returns `pal.primary` when readable on `bg` (>= 4.5:1), otherwise falls back to `pal.text` or `contrast_text(bg)`.
#[inline]
pub fn readable_accent_on(pal: ThemePalette, bg: Color) -> Color {
    readable_accent_with_fallback(pal.primary, pal.text, bg)
}

/// Returns `pal.primary` when readable on `hud_bg` (>= 4.5:1), otherwise falls back to `pal.hud_text` or `contrast_text(hud_bg)`.
#[inline]
pub fn readable_hud_accent_on(pal: ThemePalette, hud_bg: Color) -> Color {
    readable_accent_with_fallback(pal.primary, pal.hud_text, hud_bg)
}

/// Computes a muted text color that blends toward the surface while preserving readable
/// WCAG AA contrast on light themes (where gamma-encoded sRGB mixes wash out much faster).
fn compute_muted_text(text: Color, modal_bg: Color, card_bg: Color, surface_light: bool) -> Color {
    let base_mix = if surface_light { 0.26 } else { 0.53 };
    let candidate = mix_color(text, modal_bg, base_mix);
    if !surface_light || contrast_ratio(candidate, card_bg) >= 4.5 || contrast_ratio(text, card_bg) < 4.5 {
        return candidate;
    }

    let mut lo = 0.0f32;
    let mut hi = base_mix;
    for _ in 0..12 {
        let mid = (lo + hi) * 0.5;
        if contrast_ratio(mix_color(text, modal_bg, mid), card_bg) >= 4.5 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    mix_color(text, modal_bg, lo)
}

#[inline]
pub fn shift_surface(base: Color, amount: f32) -> Color {
    if is_light_color(base) {
        mix_color(base, Color::BLACK, amount)
    } else {
        mix_color(base, Color::WHITE, amount)
    }
}

/// Complete resolved theme palette derived from `ThemeSettings`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemePalette {
    pub primary: Color,
    pub primary_border: Color,
    pub primary_hover: Color,
    pub primary_pressed: Color,
    pub primary_on_text: Color,
    pub accent_rgb: (u8, u8, u8),
    pub titlebar_bg: Color,
    pub titlebar_text: Color,
    pub titlebar_brand_text: Color,
    pub titlebar_icon_color: Color,
    pub badge_bg: Color,
    pub badge_hover: Color,
    pub badge_text: Color,
    pub modal_bg: Color,
    pub card_bg: Color,
    pub btn_bg: Color,
    pub btn_hover: Color,
    pub btn_pressed: Color,
    pub btn_close_hover: Color,
    pub border: Color,
    pub border_hover: Color,
    pub drawer_bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub hud_bg: Color,
    pub hud_hover: Color,
    pub hud_pressed: Color,
    pub hud_text: Color,
    pub hud_opacity: f32,
    pub empty_bg: Color,
    pub track_bg: Color,
    pub overlay_dark: Color,
    pub corner_radius: f32,
    pub focus_ring_width: f32,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::from_settings(&ThemeSettings::default())
    }
}

impl ThemePalette {
    pub fn from_settings(theme: &ThemeSettings) -> Self {
        let accent_rgb = parse_hex_rgb(&theme.accent_color).unwrap_or((0x42, 0xb8, 0x83));
        let primary = Color::from_rgb8(accent_rgb.0, accent_rgb.1, accent_rgb.2);
        let primary_border = mix_color(primary, Color::WHITE, 0.14);
        let primary_hover = mix_color(primary, Color::WHITE, 0.08);
        let primary_pressed = mix_color(primary, Color::BLACK, 0.14);
        let primary_on_text = contrast_text(primary);

        let modal_bg = color_from_hex_or(&theme.surface_color, COLOR_MODAL_BG);
        let surface_light = is_light_color(modal_bg);
        let card_bg = shift_surface(modal_bg, 0.049);
        let btn_bg = shift_surface(modal_bg, 0.089);
        let border = shift_surface(modal_bg, 0.142);
        let border_hover = shift_surface(modal_bg, 0.25);
        let btn_hover = shift_surface(modal_bg, 0.165);
        let btn_pressed = if surface_light {
            mix_color(modal_bg, Color::BLACK, 0.16)
        } else {
            mix_color(modal_bg, Color::BLACK, 0.22)
        };
        let drawer_bg = btn_bg;
        let empty_bg = if surface_light {
            mix_color(modal_bg, Color::WHITE, 0.25)
        } else {
            mix_color(modal_bg, Color::BLACK, 0.58)
        };

        let titlebar_bg = color_from_hex_or(&theme.titlebar_color, COLOR_TITLEBAR_BG);
        let titlebar_contrast = contrast_text(titlebar_bg);
        let titlebar_text = titlebar_contrast;
        let titlebar_brand_text = with_alpha(titlebar_contrast, 0.72);
        let titlebar_icon_color = mix_color(titlebar_contrast, titlebar_bg, 0.20);

        let badge_bg = color_from_hex_or(&theme.titlebar_badge_color, COLOR_BADGE_BG);
        let badge_hover = shift_surface(badge_bg, 0.08);
        let badge_text = contrast_text(badge_bg);

        let text = color_from_hex_or(&theme.text_color, Color::WHITE);
        let text_dim = mix_color(text, modal_bg, if surface_light { 0.12 } else { 0.15 });
        let text_muted = compute_muted_text(text, modal_bg, card_bg, surface_light);

        let hud_bg = color_from_hex_or(&theme.hud_color, Color::from_rgb(0.08, 0.08, 0.10));
        let hud_hover = shift_surface(hud_bg, 0.08);
        let hud_pressed = mix_color(hud_bg, Color::BLACK, 0.18);
        let hud_text = contrast_text(hud_bg);
        let hud_opacity = theme.hud_opacity.clamp(0.25, 1.0);

        let track_bg = with_alpha(text, 0.20);

        Self {
            primary,
            primary_border,
            primary_hover,
            primary_pressed,
            primary_on_text,
            accent_rgb,
            titlebar_bg,
            titlebar_text,
            titlebar_brand_text,
            titlebar_icon_color,
            badge_bg,
            badge_hover,
            badge_text,
            modal_bg,
            card_bg,
            btn_bg,
            btn_hover,
            btn_pressed,
            btn_close_hover: COLOR_BTN_CLOSE_HOVER,
            border,
            border_hover,
            drawer_bg,
            text,
            text_dim,
            text_muted,
            hud_bg,
            hud_hover,
            hud_pressed,
            hud_text,
            hud_opacity,
            empty_bg,
            track_bg,
            overlay_dark: COLOR_OVERLAY_DARK,
            corner_radius: theme.corner_radius.clamp(0.0, 2.0),
            focus_ring_width: theme.focus_ring_width.clamp(0.0, 8.0),
        }
    }

    #[inline]
    pub fn radius(self, base: f32) -> f32 {
        (base * self.corner_radius).max(0.0)
    }

    #[inline]
    pub fn accent_hex(self) -> String {
        let (r, g, b) = self.accent_rgb;
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    pub fn to_iced_theme(self) -> iced::Theme {
        iced::Theme::custom(
            "Wazoo",
            iced::theme::Palette {
                background: self.modal_bg,
                text: self.text,
                primary: self.primary,
                success: self.primary,
                warning: Color::from_rgb(0.95, 0.65, 0.2),
                danger: self.btn_close_hover,
            },
        )
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use wazoo_core::ThemePreset;

    #[test]
    fn test_light_theme_muted_text_contrast() {
        for preset in [ThemePreset::Strawberry, ThemePreset::FrostLight] {
            let mut settings = ThemeSettings::default();
            settings.apply_preset(preset);
            let pal = ThemePalette::from_settings(&settings);
            assert!(
                contrast_ratio(pal.text_muted, pal.modal_bg) >= 4.5,
                "{preset:?} text_muted contrast on modal_bg too low: {}",
                contrast_ratio(pal.text_muted, pal.modal_bg)
            );
            assert!(
                contrast_ratio(pal.text_muted, pal.card_bg) >= 4.5,
                "{preset:?} text_muted contrast on card_bg too low: {}",
                contrast_ratio(pal.text_muted, pal.card_bg)
            );
        }
    }
}
