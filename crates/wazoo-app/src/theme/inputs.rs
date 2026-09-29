/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Theme Text Inputs & Selection Controls
 */

use super::colors::*;
use iced::{Background, Border, Color, Shadow, Theme, Vector, overlay::menu, widget::text_input};

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
