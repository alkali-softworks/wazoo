use iced::{
    widget::{button, container, slider, text_input},
    Background, Border, Color, Shadow, Theme, Vector,
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
pub const COLOR_BLUE_ACTIVE: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3b82f6
pub const COLOR_BLUE_BORDER: Color = Color::from_rgb(0.376, 0.647, 0.980); // #60a5fa
pub const COLOR_TRACK_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.2);
pub const COLOR_OVERLAY_DARK: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.4);
pub const COLOR_DRAWER_BG: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333

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

// Menu Items (Dropdown and Menu Modal)
pub fn menu_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgb(0.12, 0.12, 0.12),
        _ => Color::TRANSPARENT,
    };
    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border::default(),
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

// Folder Chip in Search Modal
pub fn folder_chip_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        let (bg, border_color) = if is_active {
            (COLOR_BLUE_ACTIVE, COLOR_BLUE_BORDER)
        } else {
            match status {
                button::Status::Hovered => (COLOR_BTN_BG, Color::from_rgb(0.35, 0.35, 0.35)),
                _ => (COLOR_CARD_BG, COLOR_BORDER),
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                radius: 20.0.into(),
                width: 1.0,
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
        text_input::Status::Focused { .. } => Color::from_rgb(0.4, 0.4, 0.4),
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

// Progress Bar Slider Style (Vue emerald green #42b883)
pub fn progress_slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(COLOR_PRIMARY),
                Background::Color(COLOR_TRACK_BG),
            ),
            width: 22.0,
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
pub fn title_pill_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(COLOR_OVERLAY_DARK)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 8.0,
                bottom_right: 8.0,
                bottom_left: 0.0,
            },
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

// Controls Bottom Overlay Style
pub fn controls_overlay_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
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
