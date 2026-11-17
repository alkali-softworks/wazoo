/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Text Inputs & Selection Controls
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, overlay::menu, widget::text_input};

// Text Input Style
pub fn dark_input_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme: &Theme, status: text_input::Status| {
        let border_color = match status {
            text_input::Status::Focused { .. } => pal.primary,
            text_input::Status::Hovered => pal.border_hover,
            _ => pal.border,
        };
        text_input::Style {
            background: Background::Color(pal.card_bg),
            border: Border {
                radius: pal.radius(4.0).into(),
                width: 1.0,
                color: border_color,
            },
            icon: pal.text_muted,
            placeholder: pal.text_muted,
            value: pal.text,
            selection: with_alpha(pal.primary, 0.4),
        }
    }
}

// Transparent Text Input (inside tag input box)
pub fn transparent_input_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme: &Theme, _status: text_input::Status| text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: 0.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        icon: pal.text_muted,
        placeholder: pal.text_muted,
        value: pal.text,
        selection: with_alpha(pal.primary, 0.4),
    }
}

pub fn dark_pick_list_style(
    pal: ThemePalette,
) -> impl Fn(&Theme, iced::widget::pick_list::Status) -> iced::widget::pick_list::Style {
    move |_theme: &Theme, status: iced::widget::pick_list::Status| {
        let border_color = match status {
            iced::widget::pick_list::Status::Opened { .. } => pal.primary,
            iced::widget::pick_list::Status::Hovered => pal.border_hover,
            _ => pal.border,
        };
        iced::widget::pick_list::Style {
            text_color: pal.text,
            placeholder_color: pal.text_muted,
            handle_color: pal.text_dim,
            background: Background::Color(pal.card_bg),
            border: Border {
                radius: pal.radius(4.0).into(),
                width: 1.0,
                color: border_color,
            },
        }
    }
}

pub fn dark_pick_list_menu_style(pal: ThemePalette) -> impl Fn(&Theme) -> menu::Style {
    move |_theme: &Theme| menu::Style {
        background: Background::Color(pal.modal_bg),
        border: Border {
            radius: pal.radius(6.0).into(),
            width: 1.0,
            color: pal.border,
        },
        text_color: pal.text,
        selected_text_color: pal.primary_on_text,
        selected_background: Background::Color(pal.primary),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 16.0,
        },
    }
}
