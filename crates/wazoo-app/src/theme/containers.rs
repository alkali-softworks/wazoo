/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Containers, Cards, Badges & Overlays
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::container};

// Menu Dropdown Container (Titlebar Logo Dropdown)
pub fn menu_dropdown_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.modal_bg)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: pal.radius(8.0),
                bottom_left: pal.radius(8.0),
            },
            width: 1.0,
            color: pal.border,
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
pub fn tag_input_box_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.card_bg)),
        border: Border {
            radius: pal.radius(6.0).into(),
            width: 1.0,
            color: pal.border,
        },
        text_color: Some(pal.text),
        ..Default::default()
    }
}

// Tag Chip container
pub fn tag_chip_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.btn_bg)),
        border: Border {
            radius: pal.radius(4.0).into(),
            width: 1.0,
            color: pal.border,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        text_color: Some(pal.text),
        ..Default::default()
    }
}

// Audio Menu Card Style
pub fn audio_menu_card_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(with_alpha(
            pal.hud_bg,
            (pal.hud_opacity + 0.08).min(0.98),
        ))),
        border: Border {
            radius: pal.radius(8.0).into(),
            width: 1.0,
            color: with_alpha(pal.border, 0.8),
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
pub fn modal_card_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.modal_bg)),
        border: Border {
            radius: pal.radius(8.0).into(),
            width: 1.0,
            color: pal.border,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 25.0,
        },
        ..Default::default()
    }
}

// Settings Section Card Style
pub fn settings_card_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.card_bg)),
        border: Border {
            radius: pal.radius(8.0).into(),
            width: 1.0,
            color: pal.border,
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

// Active Player Focus Ring (colored ring overlaid on top of video)
pub fn focus_ring_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: None,
        border: Border {
            radius: 0.0.into(),
            width: pal.focus_ring_width,
            color: pal.primary,
        },
        ..Default::default()
    }
}

pub fn title_pill_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(with_alpha(
            pal.hud_bg,
            pal.hud_opacity * alpha,
        ))),
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
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.55 * alpha),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

pub fn controls_overlay_style_with_alpha(
    pal: ThemePalette,
    alpha: f32,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(with_alpha(
            pal.hud_bg,
            pal.hud_opacity * alpha,
        ))),
        border: Border {
            radius: pal.radius(14.0).into(),
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
pub fn drag_overlay_card_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(with_alpha(pal.hud_bg, 0.85))),
        border: Border {
            radius: pal.radius(14.0).into(),
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
pub fn notice_pill_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(with_alpha(pal.hud_bg, 0.78))),
        border: Border {
            radius: pal.radius(8.0).into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

// Top Scan Toast Banner Style
pub fn scan_toast_banner_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(with_alpha(pal.hud_bg, 0.85))),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: with_alpha(pal.border, 0.5),
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
pub fn file_picker_drawer_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(pal.drawer_bg)),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: pal.border,
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
pub fn welcome_card_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| container::Style {
        background: Some(Background::Color(with_alpha(pal.modal_bg, 0.85))),
        border: Border {
            radius: pal.radius(12.0).into(),
            width: 1.0,
            color: pal.border,
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
pub fn transcript_time_badge_style(
    pal: ThemePalette,
    is_active: bool,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| {
        if is_active {
            container::Style {
                background: Some(Background::Color(pal.primary)),
                border: Border {
                    radius: pal.radius(4.0).into(),
                    ..Default::default()
                },
                text_color: Some(pal.primary_on_text),
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(Background::Color(with_alpha(pal.text, 0.08))),
                border: Border {
                    radius: pal.radius(4.0).into(),
                    width: 1.0,
                    color: with_alpha(pal.text, 0.06),
                },
                text_color: Some(pal.text_muted),
                ..Default::default()
            }
        }
    }
}

// Transcript Count Pill Badge Style
pub fn transcript_count_badge_style(pal: ThemePalette) -> impl Fn(&Theme) -> container::Style {
    move |_theme: &Theme| {
        let bg = mix_color(pal.drawer_bg, pal.primary, 0.18);
        container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: pal.radius(12.0).into(),
                width: 1.0,
                color: with_alpha(pal.primary, 0.55),
            },
            text_color: Some(readable_accent_on(pal, bg)),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::buttons::{
        audio_menu_item_style, settings_reset_button_style, settings_tab_button_style,
        settings_toggle_card_style,
    };
    use iced::widget::button;
    use wazoo_core::{ThemePreset, ThemeSettings};

    #[test]
    fn test_accent_badges_and_tabs_contrast_across_themes() {
        let mut custom_dark = ThemeSettings::default();
        custom_dark.accent_color = "#b01078".to_string();
        let mut custom_light = ThemeSettings::default();
        custom_light.apply_preset(ThemePreset::FrostLight);
        custom_light.accent_color = "#ffea00".to_string();

        let mut palettes: Vec<ThemePalette> = ThemePreset::ALL
            .iter()
            .map(|&preset| {
                let mut settings = ThemeSettings::default();
                settings.apply_preset(preset);
                ThemePalette::from_settings(&settings)
            })
            .collect();
        palettes.push(ThemePalette::from_settings(&custom_dark));
        palettes.push(ThemePalette::from_settings(&custom_light));

        let theme = Theme::Dark;
        for pal in palettes {
            let badge = transcript_count_badge_style(pal)(&theme);
            let Some(Background::Color(bg)) = badge.background else {
                panic!("expected solid color background on transcript count badge");
            };
            let fg = badge.text_color.expect("expected text color");
            assert!(
                contrast_ratio(fg, bg) >= 4.5,
                "transcript_count_badge_style contrast too low: {}",
                contrast_ratio(fg, bg)
            );

            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
            ] {
                let tab = settings_tab_button_style(pal, true)(&theme, status);
                let Some(Background::Color(tab_bg)) = tab.background else {
                    panic!("expected solid background on active settings tab");
                };
                assert!(
                    contrast_ratio(tab.text_color, tab_bg) >= 4.5,
                    "settings_tab_button_style contrast too low: {}",
                    contrast_ratio(tab.text_color, tab_bg)
                );

                let card = settings_toggle_card_style(pal, true)(&theme, status);
                let Some(Background::Color(card_bg)) = card.background else {
                    panic!("expected solid background on active settings toggle card");
                };
                assert!(
                    contrast_ratio(card.text_color, card_bg) >= 4.5,
                    "settings_toggle_card_style contrast too low: {}",
                    contrast_ratio(card.text_color, card_bg)
                );

                let reset = settings_reset_button_style(pal)(&theme, status);
                let Some(Background::Color(reset_bg)) = reset.background else {
                    panic!("expected solid background on reset button");
                };
                assert!(
                    contrast_ratio(reset.text_color, reset_bg) >= 4.5,
                    "settings_reset_button_style contrast too low: {}",
                    contrast_ratio(reset.text_color, reset_bg)
                );

                let menu = audio_menu_item_style(pal, true)(&theme, status);
                let Some(Background::Color(menu_bg)) = menu.background else {
                    panic!("expected solid background on selected audio menu item");
                };
                assert!(
                    contrast_ratio(menu.text_color, menu_bg) >= 4.5,
                    "audio_menu_item_style contrast too low: {}",
                    contrast_ratio(menu.text_color, menu_bg)
                );
            }
        }
    }
}
