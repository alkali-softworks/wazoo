use super::common::{new_backend_app, press_char};
use iced::Point;
use wazoo_app::app::new_test_app;
use wazoo_app::message::Message;
use wazoo_app::state::PipCorner;
use wazoo_core::VideoRecord;

#[test]
fn test_is_point_in_titlebar() {
    let (mut app, _) = new_test_app();
    app.settings.window_bounds.width = 800;
    app.settings.window_bounds.height = 600;

    // Inside titlebar bounds (0 <= x <= 800, 0 <= y < 35)
    assert!(app.is_point_in_titlebar(Point::new(0.0, 0.0)));
    assert!(app.is_point_in_titlebar(Point::new(400.0, 20.0)));
    assert!(app.is_point_in_titlebar(Point::new(800.0, 34.9)));

    // Outside titlebar bounds
    assert!(!app.is_point_in_titlebar(Point::new(400.0, 35.0)));
    assert!(!app.is_point_in_titlebar(Point::new(400.0, 100.0)));
    assert!(!app.is_point_in_titlebar(Point::new(400.0, -1.0)));
    assert!(!app.is_point_in_titlebar(Point::new(-10.0, 10.0)));
    assert!(!app.is_point_in_titlebar(Point::new(801.0, 10.0)));
}

#[test]
fn test_pin_keybind_dispatch() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 1.0);

    // 1st press 'p': Always on top + 100% opacity (no clickthru even when unfocused)
    press_char(&mut app, "p");
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 1.0);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Always on top enabled")
    );
    let _ = app.update(Message::WindowUnfocused);
    assert!(!app.window.clickthru);

    // 2nd press 'p': Always on top + 40% opacity (clickthru when unfocused)
    press_char(&mut app, "p");
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 0.40);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Always on top enabled (40% Opacity)")
    );
    let _ = app.update(Message::WindowUnfocused);
    assert!(app.window.clickthru);
    let _ = app.update(Message::WindowFocused);
    assert!(!app.window.clickthru);

    // 3rd press 'p': Always on top + 15% opacity (clickthru when unfocused)
    press_char(&mut app, "p");
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 0.15);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Always on top enabled (15% Opacity)")
    );
    let _ = app.update(Message::WindowUnfocused);
    assert!(app.window.clickthru);

    // 4th press 'p': Toggle back to normal
    press_char(&mut app, "p");
    assert!(!app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 1.0);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Always on top disabled")
    );
    assert!(!app.window.clickthru);
}

#[test]
fn test_window_resize_suspends_frame_rendering_and_debounces() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: not resizing
    assert!(!app.window.is_resizing());
    assert!(app.window.last_resize_time.is_none());

    // WindowResized marks window as resizing
    let _ = app.update(Message::WindowResized(
        win_id,
        iced::Size::new(960.0, 540.0),
    ));
    assert!(app.window.is_resizing());
    assert!(app.window.last_resize_time.is_some());

    // Tick runs cleanly while resizing (skipping video frames and scroll sync)
    let _ = app.update(Message::Tick);
    assert!(app.window.is_resizing());

    // LeftClickReleased resets resize state immediately
    let _ = app.update(Message::LeftClickReleased);
    assert!(!app.window.is_resizing());
    assert!(app.window.last_resize_time.is_none());

    // DragResize also activates resizing state
    let _ = app.update(Message::DragResize(iced::window::Direction::East));
    assert!(app.window.is_resizing());
    assert!(app.window.last_resize_time.is_some());

    // Simulating elapsed debounce duration expires the resizing state
    app.window.last_resize_time = Some(
        std::time::Instant::now()
            - wazoo_app::state::WindowState::RESIZE_DEBOUNCE_DURATION
            - std::time::Duration::from_millis(10),
    );
    assert!(!app.window.is_resizing());
}

#[test]
fn test_video_tile_layout_engine_pauses_during_resize() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: not resizing
    assert!(!app.window.is_resizing());

    // WindowResized marks window as resizing
    let _ = app.update(Message::WindowResized(
        win_id,
        iced::Size::new(1280.0, 720.0),
    ));
    assert!(app.window.is_resizing());

    // Cursor movement during resize is ignored to prevent titlebar/hover thrashing
    let _ = app.update(Message::CursorMoved(win_id, Point::new(10.0, 10.0)));
    assert_eq!(app.titlebar.hover_ticks, 0);

    // Player hover messages during resize are suppressed
    let _ = app.update(Message::PlayerHovered(1));
    assert_eq!(app.hovered_player_id, None);

    // Player unhover messages during resize are suppressed
    let _ = app.update(Message::PlayerUnhovered(1));
    assert_eq!(app.hovered_player_id, None);

    // WindowMoved also keeps resize debounce alive
    let _ = app.update(Message::WindowMoved(win_id, Point::new(100.0, 100.0)));
    assert!(app.window.is_resizing());

    // Root view during resize renders smoothly without titlebar injection
    {
        let _root_view = app.view();
    }

    // Releasing left click immediately exits resizing and flushes layout updates
    let _ = app.update(Message::LeftClickReleased);
    assert!(!app.window.is_resizing());
}

#[test]
fn test_unfocused_window_ticks_cube_and_animations_at_full_speed() {
    let (mut app, _) = new_test_app();
    app.settings.window_bounds.width = 800;
    app.settings.window_bounds.height = 600;

    // Spawn a 3D cube
    app.cube.spawn_cube(800.0, 600.0, 0);
    assert!(app.cube.is_present());
    let initial_x = app.cube.cubes[0].x;
    let initial_ry = app.cube.cubes[0].ry;

    // Window is unfocused
    app.window.is_focused = false;

    // Tick 1: cube advances position and rotation
    let _ = app.update(Message::Tick);
    let x1 = app.cube.cubes[0].x;
    let ry1 = app.cube.cubes[0].ry;
    assert_ne!(x1, initial_x);
    assert_ne!(ry1, initial_ry);

    // Tick 2: unfocused video frames would be throttled, BUT cube still advances!
    let _ = app.update(Message::Tick);
    let x2 = app.cube.cubes[0].x;
    let ry2 = app.cube.cubes[0].ry;
    assert_ne!(x2, x1);
    assert_ne!(ry2, ry1);

    // The displacement in tick 2 matches tick 1 (runs at consistent 60 Hz, not half-speed!)
    let delta1 = (x1 - initial_x).abs();
    let delta2 = (x2 - x1).abs();
    assert!((delta1 - delta2).abs() < 1e-4);
}

#[test]
fn test_fullscreen_toggle_and_escape() {
    let (mut app, _) = new_test_app();
    assert!(!app.window.is_fullscreen);

    // 1. Toggle fullscreen ON via Message
    let _ = app.update(Message::ToggleFullscreen);
    assert!(app.window.is_fullscreen);
    assert!(app.overlay.toast_message.is_some());
    assert!(
        app.overlay
            .toast_message
            .as_ref()
            .unwrap()
            .contains("Fullscreen Enabled")
    );
    assert!(
        app.overlay
            .toast_message
            .as_ref()
            .unwrap()
            .contains("[F11]")
    );

    // 2. Escape exits fullscreen when no modals/drawers are open
    let _ = app.update(Message::EscapePressed);
    assert!(!app.window.is_fullscreen);
    assert!(
        app.overlay
            .toast_message
            .as_ref()
            .unwrap()
            .contains("Fullscreen Disabled")
    );

    // 3. Rebind toggle_fullscreen and verify toast reflects the new key
    app.settings.keybinds.toggle_fullscreen = "f12".to_string();
    let _ = app.update(Message::ToggleFullscreen);
    assert!(app.window.is_fullscreen);
    assert!(
        app.overlay
            .toast_message
            .as_ref()
            .unwrap()
            .contains("[F12]")
    );
    let _ = app.update(Message::ToggleFullscreen);
    assert!(!app.window.is_fullscreen);
    app.settings.keybinds.toggle_fullscreen = "F11".to_string();

    // 4. F11 key input toggles fullscreen
    use iced::keyboard::key::Named;
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Named(Named::F11),
        iced::event::Status::Ignored,
    ));
    assert!(app.window.is_fullscreen);

    // 5. Opening a drawer or modal while in fullscreen, then pressing Escape closes drawer first without exiting fullscreen
    let _ = app.update(Message::ToggleFilePicker);
    assert!(app.drawers.show_file_picker);
    let _ = app.update(Message::EscapePressed);
    assert!(!app.drawers.show_file_picker);
    assert!(
        app.window.is_fullscreen,
        "Closing drawer should not exit fullscreen"
    );

    // 6. Subsequent Escape exits fullscreen
    let _ = app.update(Message::EscapePressed);
    assert!(!app.window.is_fullscreen);
}

#[test]
fn test_fit_window_keybind_and_resizing() {
    let (mut app, temp_dir) = new_backend_app("wazoo_fit_window_test");

    let f1 = temp_dir.path().join("test_video.mp4");
    let _ = std::fs::File::create(&f1);
    let path1 = f1.to_string_lossy().to_string();

    app.available_videos = vec![VideoRecord::new(1, "Test Video", &path1)];
    app.default_shuffle_mode = false;

    // Start 1 player
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);

    // Set 16:9 frame dimensions on player (e.g. 1920x1080)
    {
        let frame_arc = app.players[0].frame();
        let mut guard = frame_arc.lock().unwrap();
        guard.width = 1920;
        guard.height = 1080;
    }
    assert_eq!(app.players[0].aspect_ratio(), Some(1920.0 / 1080.0));

    // Case 1: Pillarbox (window is too wide, e.g. 2560x1080 as in user photo)
    app.settings.window_bounds.width = 2560;
    app.settings.window_bounds.height = 1080;

    // Press 'w' key
    press_char(&mut app, "w");

    // Window width shrinks to 1920 to eliminate horizontal black bars, height remains 1080
    assert_eq!(app.settings.window_bounds.width, 1920);
    assert_eq!(app.settings.window_bounds.height, 1080);

    // Case 2: Letterbox (window is too tall, e.g. 1920x1500)
    app.settings.window_bounds.width = 1920;
    app.settings.window_bounds.height = 1500;

    // Press 'W' key (uppercase)
    press_char(&mut app, "W");

    // Window height shrinks to 1080 to eliminate vertical black bars, width remains 1920
    assert_eq!(app.settings.window_bounds.width, 1920);
    assert_eq!(app.settings.window_bounds.height, 1080);

    // Case 3: With drawer open (e.g. file picker drawer 420px wide)
    app.drawers.show_file_picker = true;
    app.settings.window_bounds.width = 3000;
    app.settings.window_bounds.height = 1080;

    let _ = app.update(Message::FitWindow);

    // Total width = player width (1920) + drawer width (420) = 2340
    assert_eq!(app.settings.window_bounds.width, 2340);
    assert_eq!(app.settings.window_bounds.height, 1080);

    app.drawers.show_file_picker = false;

    // Case 4: Fullscreen guard (does not resize in fullscreen mode)
    app.window.is_fullscreen = true;
    app.settings.window_bounds.width = 2560;
    app.settings.window_bounds.height = 1080;
    let _ = app.update(Message::FitWindow);
    assert_eq!(app.settings.window_bounds.width, 2560);
    app.window.is_fullscreen = false;
}

#[test]
fn test_fit_window_multi_player_layouts() {
    let (mut app, temp_dir) = new_backend_app("wazoo_fit_multi_test");

    let f1 = temp_dir.path().join("v1.mp4");
    let f2 = temp_dir.path().join("v2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        VideoRecord::new(1, "V1", f1.to_string_lossy().to_string()),
        VideoRecord::new(2, "V2", f2.to_string_lossy().to_string()),
    ];
    app.default_shuffle_mode = false;

    // Start 2 players
    let _ = app.update(Message::SetPlayerCount(2));
    assert_eq!(app.players.len(), 2);

    for p in &mut app.players {
        let frame_arc = p.frame();
        let mut guard = frame_arc.lock().unwrap();
        guard.width = 1920;
        guard.height = 1080;
    }

    // Row layout: 2 side-by-side tiles -> target window aspect ratio = 2 * (16/9) = 32/9
    app.settings.layout = wazoo_core::LayoutMode::Row;
    app.settings.window_bounds.width = 4000;
    app.settings.window_bounds.height = 1000;
    let _ = app.update(Message::FitWindow);
    // target_ar = (16.0 / 9.0) * 2.0 = 32.0 / 9.0 ≈ 3.5555
    // current_ar = 4000 / 1000 = 4.0 > 3.5555
    // new_width = 1000 * (32/9) ≈ 3556
    assert_eq!(app.settings.window_bounds.width, 3556);
    assert_eq!(app.settings.window_bounds.height, 1000);

    // Column layout: 2 stacked tiles -> target window aspect ratio = (16/9) / 2 = 8/9
    app.settings.layout = wazoo_core::LayoutMode::Column;
    app.settings.window_bounds.width = 1600;
    app.settings.window_bounds.height = 2000;
    let _ = app.update(Message::FitWindow);
    // target_ar = (16.0 / 9.0) / 2.0 = 8.0 / 9.0 ≈ 0.8888
    // current_ar = 1600 / 2000 = 0.8 < 0.8888 (too tall)
    // new_height = 1600 / (8/9) = 1800
    assert_eq!(app.settings.window_bounds.width, 1600);
    assert_eq!(app.settings.window_bounds.height, 1800);
}

#[test]
fn test_calculate_alt_resize_direction_all_zones() {
    let (mut app, _) = new_test_app();
    app.settings.window_bounds.width = 900;
    app.settings.window_bounds.height = 600;

    // Top row (row = 0)
    app.window.cursor_position = Point::new(100.0, 100.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::NorthWest
    ));

    app.window.cursor_position = Point::new(450.0, 100.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::North
    ));

    app.window.cursor_position = Point::new(800.0, 100.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::NorthEast
    ));

    // Middle row (row = 1) - Outer edges
    app.window.cursor_position = Point::new(100.0, 300.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::West
    ));

    app.window.cursor_position = Point::new(800.0, 300.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::East
    ));

    // Bottom row (row = 2)
    app.window.cursor_position = Point::new(100.0, 500.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::SouthWest
    ));

    app.window.cursor_position = Point::new(450.0, 500.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::South
    ));

    app.window.cursor_position = Point::new(800.0, 500.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::SouthEast
    ));

    // Center zone (row = 1, col = 1) delegates to nearest quadrant
    // Top-left of center: x < 450, y < 300 -> NorthWest
    app.window.cursor_position = Point::new(350.0, 250.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::NorthWest
    ));

    // Top-right of center: x >= 450, y < 300 -> NorthEast
    app.window.cursor_position = Point::new(550.0, 250.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::NorthEast
    ));

    // Bottom-left of center: x < 450, y >= 300 -> SouthWest
    app.window.cursor_position = Point::new(350.0, 350.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::SouthWest
    ));

    // Bottom-right of center: x >= 450, y >= 300 -> SouthEast
    app.window.cursor_position = Point::new(550.0, 350.0);
    assert!(matches!(
        app.calculate_alt_resize_direction(),
        iced::window::Direction::SouthEast
    ));
}

#[test]
fn test_alt_drag_resize_and_right_click_interception() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();

    // 1. Regular right-click without Alt opens context menu modal
    app.window.is_alt_pressed = false;
    assert!(!app.modals.menu);
    let _ = app.update(Message::RightClickPressed(win_id));
    assert!(app.modals.menu);
    assert_eq!(app.window.id, Some(win_id));

    // Close menu modal
    app.modals.menu = false;

    // 2. Alt + Right-click intercepts menu, triggers AltDragResize instead
    app.window.is_alt_pressed = true;
    app.settings.window_bounds.width = 900;
    app.settings.window_bounds.height = 600;
    app.window.cursor_position = Point::new(800.0, 500.0); // SouthEast corner

    let _ = app.update(Message::RightClickPressed(win_id));
    assert!(
        !app.modals.menu,
        "Context menu modal should NOT be opened when Alt is pressed"
    );
    assert!(
        app.window.is_resizing(),
        "Alt + Right-click should mark window as resizing"
    );
    assert!(app.window.last_resize_time.is_some());

    // 3. RightClickReleased resets resize state immediately
    let _ = app.update(Message::RightClickReleased);
    assert!(
        !app.window.is_resizing(),
        "RightClickReleased should reset is_resizing immediately"
    );
    assert!(app.window.last_resize_time.is_none());

    // 4. AltDragResize in fullscreen mode is suppressed
    app.window.is_fullscreen = true;
    let _ = app.update(Message::AltDragResize);
    assert!(
        !app.window.is_resizing(),
        "Resize should not trigger when in fullscreen mode"
    );
}

#[test]
fn test_shift_p_pip_always_on_top_and_corner_cycling() {
    let (mut app, _) = new_test_app();
    app.window.monitor_size = Some(iced::Size::new(1920.0, 1080.0));
    app.settings.window_bounds = wazoo_core::WindowBounds {
        x: 100,
        y: 100,
        width: 1280,
        height: 720,
    };
    app.window.is_fullscreen = true;
    app.window.is_maximized = true;

    assert!(!app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, None);

    // 1st Shift+P: activates always-on-top, exits fullscreen/maximized, resizes to PiP (720x405), moves to Bottom-Left
    let shift_mods = iced::keyboard::Modifiers::SHIFT;
    let _ = app.update(Message::KeyPressedWithModifiers(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
        shift_mods,
    ));

    assert!(app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 1.0);
    assert!(!app.window.clickthru);
    assert!(!app.window.is_fullscreen);
    assert!(!app.window.is_maximized);
    assert_eq!(app.window.pip_corner, Some(PipCorner::BottomLeft));
    assert_eq!(app.settings.window_bounds.width, 720);
    assert_eq!(app.settings.window_bounds.height, 405);
    assert_eq!(app.settings.window_bounds.x, 0);
    assert_eq!(app.settings.window_bounds.y, 1080 - 405);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Always on top enabled (PiP: Bottom-Left)")
    );

    // 2nd Shift+P: moves to Top-Left, keeps always-on-top active
    let _ = app.update(Message::KeyPressedWithModifiers(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
        shift_mods,
    ));
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, Some(PipCorner::TopLeft));
    assert_eq!(app.settings.window_bounds.x, 0);
    assert_eq!(app.settings.window_bounds.y, 0);

    // 3rd Shift+P: moves to Top-Right
    let _ = app.update(Message::KeyPressedWithModifiers(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
        shift_mods,
    ));
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, Some(PipCorner::TopRight));
    assert_eq!(app.settings.window_bounds.x, 1920 - 720);
    assert_eq!(app.settings.window_bounds.y, 0);

    // 4th Shift+P: moves to Bottom-Right
    let _ = app.update(Message::KeyPressedWithModifiers(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
        shift_mods,
    ));
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, Some(PipCorner::BottomRight));
    assert_eq!(app.settings.window_bounds.x, 1920 - 720);
    assert_eq!(app.settings.window_bounds.y, 1080 - 405);

    // 5th Shift+P: wraps back to Bottom-Left
    let _ = app.update(Message::KeyPressedWithModifiers(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
        shift_mods,
    ));
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, Some(PipCorner::BottomLeft));
    assert_eq!(app.settings.window_bounds.x, 0);
    assert_eq!(app.settings.window_bounds.y, 1080 - 405);

    // Plain 'p' (Shift released) cycles pin opacity and eventually disables always-on-top, resetting pip_corner
    let _ = app.update(Message::ModifiersChanged(
        iced::keyboard::Modifiers::empty(),
    ));
    press_char(&mut app, "p");
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.settings.window_opacity, 0.40);

    press_char(&mut app, "p");
    assert_eq!(app.settings.window_opacity, 0.15);

    press_char(&mut app, "p");
    assert!(!app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, None);
}

#[test]
fn test_shift_p_pip_aspect_ratio_monitor_update_and_modal_guard() {
    let (mut app, temp_dir) = new_backend_app("wazoo_pip_test");

    let f1 = temp_dir.path().join("vertical.mp4");
    let _ = std::fs::File::create(&f1);
    app.available_videos = vec![VideoRecord::new(1, "V1", f1.to_string_lossy().to_string())];
    app.default_shuffle_mode = false;
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);

    // Simulate 9:16 vertical video frame
    {
        let frame_arc = app.players[0].frame();
        let mut guard = frame_arc.lock().unwrap();
        guard.width = 1080;
        guard.height = 1920;
    }

    // Modal open should block Shift+P shortcut
    app.modals.search = true;
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Shift),
        iced::event::Status::Ignored,
    ));
    press_char(&mut app, "p");
    assert!(!app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, None);
    app.modals.search = false;

    // Trigger Shift+P with Shift held (via Named::Shift) using fallback 1920x1080 monitor
    press_char(&mut app, "p");
    assert!(app.settings.is_always_on_top);
    assert_eq!(app.window.pip_corner, Some(PipCorner::BottomLeft));
    // Vertical 9:16 video caps at 60% of 1080 = 648 height, width = 648 * 9/16 = 365
    assert_eq!(app.settings.window_bounds.width, 365);
    assert_eq!(app.settings.window_bounds.height, 648);
    assert_eq!(app.settings.window_bounds.x, 0);
    assert_eq!(app.settings.window_bounds.y, 1080 - 648);

    // Async monitor_size callback updates bounds when actual monitor is 2560x1440
    let _ = app.update(Message::ApplyPipMonitorSize {
        corner: PipCorner::BottomLeft,
        resize: true,
        monitor_size: Some(iced::Size::new(2560.0, 1440.0)),
    });
    assert_eq!(app.window.monitor_size, Some(iced::Size::new(2560.0, 1440.0)));
    // On 2560x1440, max_h = clamp(1440*0.6, 270, 720) = 720, width = 720 * 9/16 = 405
    assert_eq!(app.settings.window_bounds.width, 405);
    assert_eq!(app.settings.window_bounds.height, 720);
    assert_eq!(app.settings.window_bounds.x, 0);
    assert_eq!(app.settings.window_bounds.y, 1440 - 720);

    // KeyReleased(Shift) clears shift state
    let _ = app.update(Message::KeyReleased(iced::keyboard::Key::Named(
        iced::keyboard::key::Named::Shift,
    )));
    assert!(!app.window.is_shift_pressed);
}
