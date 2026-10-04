use iced::mouse;
use iced::window::Direction;
use iced::{Point, Rectangle, Size};
use wazoo_app::cursor::{determine_resize_direction, direction_to_interaction};

#[test]
fn test_determine_resize_direction_edges_and_corners() {
    let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0));
    let border = 6.0;
    let corner = 14.0;

    // Inside center: None
    assert!(determine_resize_direction(bounds, Point::new(400.0, 300.0), border, corner).is_none());

    // North edge
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(400.0, 2.0), border, corner),
        Some(Direction::North)
    ));

    // South edge
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(400.0, 598.0), border, corner),
        Some(Direction::South)
    ));

    // West edge
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(2.0, 300.0), border, corner),
        Some(Direction::West)
    ));

    // East edge
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(798.0, 300.0), border, corner),
        Some(Direction::East)
    ));

    // NorthWest corner
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(2.0, 2.0), border, corner),
        Some(Direction::NorthWest)
    ));
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(10.0, 2.0), border, corner),
        Some(Direction::NorthWest)
    ));
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(2.0, 10.0), border, corner),
        Some(Direction::NorthWest)
    ));

    // NorthEast corner
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(798.0, 2.0), border, corner),
        Some(Direction::NorthEast)
    ));
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(790.0, 2.0), border, corner),
        Some(Direction::NorthEast)
    ));

    // SouthWest corner
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(2.0, 598.0), border, corner),
        Some(Direction::SouthWest)
    ));

    // SouthEast corner
    assert!(matches!(
        determine_resize_direction(bounds, Point::new(798.0, 598.0), border, corner),
        Some(Direction::SouthEast)
    ));
}

#[test]
fn test_direction_to_interaction_mappings() {
    assert_eq!(
        direction_to_interaction(Direction::North),
        mouse::Interaction::ResizingVertically
    );
    assert_eq!(
        direction_to_interaction(Direction::South),
        mouse::Interaction::ResizingVertically
    );
    assert_eq!(
        direction_to_interaction(Direction::East),
        mouse::Interaction::ResizingHorizontally
    );
    assert_eq!(
        direction_to_interaction(Direction::West),
        mouse::Interaction::ResizingHorizontally
    );
    assert_eq!(
        direction_to_interaction(Direction::NorthWest),
        mouse::Interaction::ResizingDiagonallyDown
    );
    assert_eq!(
        direction_to_interaction(Direction::SouthEast),
        mouse::Interaction::ResizingDiagonallyDown
    );
    assert_eq!(
        direction_to_interaction(Direction::NorthEast),
        mouse::Interaction::ResizingDiagonallyUp
    );
    assert_eq!(
        direction_to_interaction(Direction::SouthWest),
        mouse::Interaction::ResizingDiagonallyUp
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_desktop_cursor_detection() {
    // If running in an environment with Cinnamon/GNOME/etc., detect_desktop_cursor_theme should find a theme.
    if let Some(theme) = wazoo_app::platform::detect_desktop_cursor_theme() {
        assert!(!theme.trim().is_empty());
    }

    if let Some(size) = wazoo_app::platform::detect_desktop_cursor_size() {
        assert!(size > 0);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_init_linux_cursor_env_custom_override() {
    unsafe {
        std::env::set_var("WAZOO_CURSOR_THEME", "TestCustomCursorTheme");
        std::env::set_var("WAZOO_CURSOR_SIZE", "42");
    }

    wazoo_app::platform::init_linux_cursor_env(false);

    assert_eq!(
        std::env::var("XCURSOR_THEME").unwrap_or_default(),
        "TestCustomCursorTheme"
    );
    assert_eq!(
        std::env::var("XCURSOR_SIZE").unwrap_or_default(),
        "42"
    );

    // Clean up test environment variables
    unsafe {
        std::env::remove_var("WAZOO_CURSOR_THEME");
        std::env::remove_var("WAZOO_CURSOR_SIZE");
    }
}
