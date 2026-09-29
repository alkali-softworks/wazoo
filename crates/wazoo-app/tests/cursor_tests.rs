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
