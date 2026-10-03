/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * 3D Video Cube Screensaver Tests
 *
 * Validates 3D cube physics, screen edge collision bouncing, neon palette cycling,
 * multi-cube management, uniform buffer byte layout, and application message integration.
 */

use wazoo_app::state::cube::{BouncingCube, CubeState, NEON_PALETTE};
use wazoo_media::cube::CubeUniforms;

#[test]
fn test_cube_uniforms_memory_layout() {
    assert_eq!(
        std::mem::size_of::<CubeUniforms>(),
        80,
        "CubeUniforms must be exactly 80 bytes for WGSL 16-byte alignment"
    );
}

#[test]
fn test_cube_state_spawn_and_toggle() {
    let mut state = CubeState::new();
    assert!(!state.enabled);
    assert!(state.cubes.is_empty());

    // Spawn first cube
    state.spawn_cube(1280.0, 720.0, 0);
    assert!(state.enabled);
    assert_eq!(state.cubes.len(), 1);

    let cube = &state.cubes[0];
    assert_eq!(cube.id, 1);
    assert!(cube.x > 0.0 && cube.x < 1280.0);
    assert!(cube.y > 0.0 && cube.y < 720.0);
    assert_ne!(cube.vx, 0.0);
    assert_ne!(cube.vy, 0.0);
    assert_eq!(cube.edge_color, NEON_PALETTE[0]);

    // Toggle off
    let is_on = state.toggle(1280.0, 720.0, 0);
    assert!(!is_on);
    assert!(!state.enabled);
    assert!(state.cubes.is_empty());

    // Toggle on
    let is_on = state.toggle(1280.0, 720.0, 0);
    assert!(is_on);
    assert!(state.enabled);
    assert_eq!(state.cubes.len(), 1);
}

#[test]
fn test_cube_multi_spawn_and_clear() {
    let mut state = CubeState::new();
    state.spawn_cube(1920.0, 1080.0, 0);
    state.spawn_cube(1920.0, 1080.0, 1);
    state.spawn_cube(1920.0, 1080.0, 2);

    assert_eq!(state.cubes.len(), 3);
    assert_eq!(state.cubes[0].player_index, 0);
    assert_eq!(state.cubes[1].player_index, 1);
    assert_eq!(state.cubes[2].player_index, 2);

    // Each cube has a unique ID and varied trajectory
    assert_ne!(state.cubes[0].id, state.cubes[1].id);
    assert_ne!(state.cubes[1].id, state.cubes[2].id);

    // Remove one cube
    state.remove_cube();
    assert_eq!(state.cubes.len(), 2);
    assert!(state.enabled);

    // Clear all
    state.clear();
    assert!(state.cubes.is_empty());
    assert!(!state.enabled);
}

#[test]
fn test_cube_screensaver_bounce_physics() {
    let mut state = CubeState::new();
    let win_w = 1000.0;
    let win_h = 800.0;

    // Test right wall bounce
    let right_cube = BouncingCube {
        id: 1,
        x: 960.0,
        y: 400.0,
        vx: 15.0,
        vy: 0.0,
        size: 50.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        vrx: 0.01,
        vry: 0.01,
        vrz: 0.01,
        edge_color: NEON_PALETTE[0],
        color_index: 0,
        player_index: 0,
        player_id: None,
        bounce_count: 0,
    };

    state.cubes.push(right_cube);
    state.enabled = true;

    // Tick advances into the right edge
    state.tick(win_w, win_h);
    let bounced = &state.cubes[0];

    assert!(
        bounced.vx < 0.0,
        "Velocity x should be negative after right wall bounce"
    );
    assert_eq!(bounced.bounce_count, 1);
    assert_eq!(bounced.color_index, 1);
    assert_eq!(bounced.edge_color, NEON_PALETTE[1]);
    assert!(bounced.x <= win_w - 50.0);

    // Test top wall bounce
    state.cubes[0].y = 40.0;
    state.cubes[0].vy = -20.0;
    state.tick(win_w, win_h);

    assert!(
        state.cubes[0].vy > 0.0,
        "Velocity y should be positive after top wall bounce"
    );
    assert_eq!(state.cubes[0].bounce_count, 2);
    assert_eq!(state.cubes[0].color_index, 2);
    assert_eq!(state.cubes[0].edge_color, NEON_PALETTE[2]);
}

#[test]
fn test_cube_speed_and_size_multipliers() {
    let mut state = CubeState::new();
    state.spawn_cube(1000.0, 1000.0, 0);

    let initial_x = state.cubes[0].x;
    let vx = state.cubes[0].vx;

    // With 1.0x speed
    state.speed_multiplier = 1.0;
    state.tick(1000.0, 1000.0);
    let step_1 = state.cubes[0].x - initial_x;
    assert!((step_1 - vx).abs() < 0.01);

    // With 2.0x speed
    let before_2x = state.cubes[0].x;
    state.speed_multiplier = 2.0;
    state.tick(1000.0, 1000.0);
    let step_2 = state.cubes[0].x - before_2x;
    assert!((step_2 - (vx * 2.0)).abs() < 0.01);
}

#[test]
fn test_cube_messages_and_state_transitions() {
    let mut state = CubeState::new();

    // Toggle message simulation
    assert!(!state.enabled);
    state.toggle(1920.0, 1080.0, 0);
    assert!(state.enabled);
    assert_eq!(state.cubes.len(), 1);

    // Spawn another
    state.spawn_cube(1920.0, 1080.0, 1);
    assert_eq!(state.cubes.len(), 2);

    // Set speed & size
    state.speed_multiplier = 2.5;
    assert_eq!(state.speed_multiplier, 2.5);
    state.size_multiplier = 1.5;
    assert_eq!(state.size_multiplier, 1.5);

    // Clear
    state.clear();
    assert!(!state.enabled);
    assert!(state.cubes.is_empty());
}

#[test]
fn test_cube_independent_player_support() {
    let mut state = CubeState::new();
    state.spawn_cube_with_player(1920.0, 1080.0, 0, Some(42));

    assert_eq!(state.cubes.len(), 1);
    assert_eq!(state.cubes[0].player_id, Some(42));

    state.spawn_cube_with_player(1920.0, 1080.0, 1, Some(43));
    assert_eq!(state.cubes.len(), 2);
    assert_eq!(state.cubes[1].player_id, Some(43));

    state.remove_cube();
    assert_eq!(state.cubes.len(), 1);
    assert_eq!(state.cubes[0].player_id, Some(42));

    state.clear();
    assert!(state.cubes.is_empty());
}

#[test]
fn test_cube_scale_and_velocity_persist_to_settings() {
    let (mut app, _) = wazoo_app::app::new_test_app();
    assert_eq!(app.cube.speed_multiplier, 1.0);
    assert_eq!(app.cube.size_multiplier, 1.0);
    assert_eq!(app.settings.cube_speed, 1.0);
    assert_eq!(app.settings.cube_size, 1.0);

    // Update cube velocity and scale via messages
    let _ = app.update(wazoo_app::message::Message::SetCubeSpeed(2.3));
    let _ = app.update(wazoo_app::message::Message::SetCubeSize(1.8));

    assert_eq!(app.cube.speed_multiplier, 2.3);
    assert_eq!(app.settings.cube_speed, 2.3);
    assert_eq!(app.cube.size_multiplier, 1.8);
    assert_eq!(app.settings.cube_size, 1.8);

    // Verify settings file on disk immediately contains the persisted values
    let reloaded = app.config_mgr.load_settings();
    assert_eq!(reloaded.cube_speed, 2.3);
    assert_eq!(reloaded.cube_size, 1.8);

    // Calling save_session_state keeps cube settings fully in sync
    app.cube.speed_multiplier = 3.1;
    app.cube.size_multiplier = 0.8;
    app.save_session_state();

    let reloaded2 = app.config_mgr.load_settings();
    assert_eq!(reloaded2.cube_speed, 3.1);
    assert_eq!(reloaded2.cube_size, 0.8);
}

#[test]
fn test_cube_focus_green_outline_flash() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // Toggle cube screensaver
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(app.cube.enabled);
    assert_eq!(app.cube.cubes.len(), 1);
    assert!(app.overlay.focus_border_ticks > 0);

    // Verify rendering cube overlay while flashing focus
    {
        let _view_flashing = app.view_cube_overlay();
    }

    // Cycle focus through players using Tab
    let _ = app.update(wazoo_app::message::Message::CycleFocusedPlayer);
    let _ = app.update(wazoo_app::message::Message::CycleFocusedPlayer);
    assert!(app.overlay.focus_border_ticks > 0);

    // Simulate overlay ticks fading out focus border
    for _ in 0..30 {
        let _ = app.update(wazoo_app::message::Message::VideoFrameTick);
    }
    assert_eq!(app.overlay.focus_border_ticks, 0);

    // Verify rendering cube overlay with zero border ticks (no green outline)
    {
        let _view_idle = app.view_cube_overlay();
    }
}

#[test]
fn test_scroll_mode_toggle_with_active_cube_preserves_grid_and_cube_players() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_scroll_cube_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let f2 = tmp.join("v2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy()),
        wazoo_core::VideoRecord::new(2, "S2", f2.to_string_lossy()),
    ];

    // Add regular grid player
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 1);
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(app.cube.enabled);
    assert_eq!(app.cube.cubes.len(), 1);
    assert_eq!(app.players.iter().filter(|p| p.is_cube).count(), 1);
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 1);

    // Focus the cube player (tab to cube)
    if let Some(pos) = app.players.iter().position(|p| p.is_cube) {
        app.focused_idx = pos;
    }
    assert!(app.players[app.focused_idx].is_cube);

    // Press '5' (ToggleScrollMode) while cube is active
    let _ = app.update(wazoo_app::message::Message::ToggleScrollMode);
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Scroll);
    // Grid player must still exist in scroll mode, and cube player must be preserved!
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 1);
    assert_eq!(app.players.iter().filter(|p| p.is_cube).count(), 1);

    // Press '5' again (ToggleScrollMode to return to normal)
    let _ = app.update(wazoo_app::message::Message::ToggleScrollMode);
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Normal);

    // Both regular grid player and cube player MUST still be present!
    assert_eq!(
        app.players.iter().filter(|p| !p.is_cube).count(),
        1,
        "Regular grid player must be preserved!"
    );
    assert_eq!(
        app.players.iter().filter(|p| p.is_cube).count(),
        1,
        "Cube player must be preserved!"
    );

    // View must render normally
    {
        let _view = app.view();
    }
}

#[test]
fn test_desktop_cube_screensaver_toggle_key_9_and_escape() {
    let (mut app, _) = wazoo_app::app::new_test_app();
    assert!(!app.cube.desktop_overlay);

    // Toggle Desktop Cube Screensaver via message / key 9
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert!(app.cube.enabled);
    assert_eq!(app.cube.cubes.len(), 1);

    // View should render desktop overlay stack without crashing
    {
        let _view = app.view();
    }

    // Key 9 again toggles off
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(!app.cube.desktop_overlay);

    // Toggle back on
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    // Escape exits desktop overlay
    let _ = app.update(wazoo_app::message::Message::EscapePressed);
    assert!(!app.cube.desktop_overlay);
}

#[test]
fn test_desktop_cube_key_event_routing() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // Send key '9'
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("9".into()),
        iced::event::Status::Ignored,
    ));
    assert!(app.cube.desktop_overlay);

    // Send key '9' again
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("9".into()),
        iced::event::Status::Ignored,
    ));
    assert!(!app.cube.desktop_overlay);
}

#[test]
fn test_desktop_cube_pauses_background_players_to_save_cpu() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_desktop_cpu_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy())];
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 1);

    // Initial regular player is playing (not paused)
    let grid_idx = app.players.iter().position(|p| !p.is_cube).unwrap();
    assert!(app.players[grid_idx].is_playing());

    // Turn ON mode 9 (Desktop Cube Screensaver)
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    // Grid player must now be paused to prevent CPU/GPU decoding waste!
    assert!(!app.players[grid_idx].is_playing());

    // Cube player must NOT be paused
    let cube_idx = app.players.iter().position(|p| p.is_cube).unwrap();
    assert!(app.players[cube_idx].is_playing());

    // Turn OFF mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(!app.cube.desktop_overlay);

    // Grid player must automatically resume playing
    assert!(app.players[grid_idx].is_playing());
}

#[test]
fn test_mode_9_mutually_exclusive_with_player_modes_1_to_4() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_mode_9_excl_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy())];
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 2);

    // Enter Mode 9 (Desktop Cube Screensaver)
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert!(app.cube.enabled);

    // Press Mode 1 (SetPlayerCount(1)) - MUST exit Mode 9
    let _ = app.update(wazoo_app::message::Message::SetPlayerCount(1));
    assert!(!app.cube.desktop_overlay, "Pressing 1 must exit Mode 9");
    assert!(!app.cube.enabled, "Cube must be dismissed when exiting Mode 9 via player modes");
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 1);
    assert!(app.players[0].is_playing(), "Regular player must be unpaused");

    // Enter Mode 9 again
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    // Press Mode 3 (SetPlayerCount(3)) - MUST exit Mode 9
    let _ = app.update(wazoo_app::message::Message::SetPlayerCount(3));
    assert!(!app.cube.desktop_overlay, "Pressing 3 must exit Mode 9");
    assert_eq!(app.players.iter().filter(|p| !p.is_cube).count(), 3);
}

#[test]
fn test_mode_9_mutually_exclusive_with_scroll_mode_5() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_mode_9_scroll_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy())];
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);

    // 1. Enter Mode 9, then press 5 (ToggleScrollMode) -> must exit Mode 9 and start scroll mode
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    let _ = app.update(wazoo_app::message::Message::ToggleScrollMode);
    assert!(!app.cube.desktop_overlay, "Pressing 5 must exit Mode 9");
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Scroll);
    assert!(!app.cube.enabled, "Cube must be dismissed when entering Scroll Mode");

    // 2. While in Scroll Mode, press 9 (ToggleDesktopCubeScreensaver) -> must clean up Scroll Mode
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert_ne!(app.settings.playback_mode, wazoo_core::PlaybackMode::Scroll, "Entering Mode 9 cleans up Scroll Mode");
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Normal);
}

#[test]
fn test_mode_9_mixes_fine_with_modes_6_and_7() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_mode_9_mix_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy())];
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);

    // Enter Mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    // Mode 6 (Flip Mode) toggles without exiting Mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleFlipMode);
    assert!(app.cube.desktop_overlay, "Mode 6 must not exit Mode 9");
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Flip);

    let _ = app.update(wazoo_app::message::Message::ToggleFlipMode);
    assert!(app.cube.desktop_overlay);
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Normal);

    // Mode 7 (CRT Filter) toggles without exiting Mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleCrtFilter);
    assert!(app.cube.desktop_overlay, "Mode 7 must not exit Mode 9");
    assert!(app.settings.crt_enabled);

    let _ = app.update(wazoo_app::message::Message::ToggleCrtFilter);
    assert!(app.cube.desktop_overlay);
    assert!(!app.settings.crt_enabled);
}

#[test]
fn test_mode_9_and_mode_8_clean_transitions() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    let tmp = std::env::temp_dir().join(format!(
        "wazoo_mode_8_9_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&tmp);
    let f1 = tmp.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy())];
    let _ = app.update(wazoo_app::message::Message::AddNewPlayer);

    // Case 1: Enter Mode 9 directly from normal app
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert!(app.cube.spawned_for_desktop);

    // Pressing 8 while in Mode 9 cleanly dismisses the cube and exits Mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(!app.cube.desktop_overlay);
    assert!(!app.cube.enabled);
    assert!(app.cube.cubes.is_empty());

    // Case 2: Start in Mode 8 (in-app cube)
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(app.cube.enabled);
    assert!(!app.cube.desktop_overlay);

    // Switch from Mode 8 to Mode 9
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert!(!app.cube.spawned_for_desktop, "Cube was already active before Mode 9");

    // Exit Mode 9 by pressing 9 -> returns to Mode 8!
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(!app.cube.desktop_overlay);
    assert!(app.cube.enabled, "Cube returns to Mode 8 in-app screensaver");

    // Dismiss Mode 8
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(!app.cube.enabled);
}

#[test]
fn test_mode_9_exits_on_modal_or_drawer_open() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // 1. Enter Mode 9, open Settings modal (F2) -> Mode 9 must exit
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    let _ = app.update(wazoo_app::message::Message::OpenSettingsModal);
    assert!(!app.cube.desktop_overlay, "Opening settings modal must exit Mode 9");
    assert!(app.modals.settings);

    // Close settings modal
    let _ = app.update(wazoo_app::message::Message::CloseSettingsModal);

    // 2. Enter Mode 9, open Help modal (F1) -> Mode 9 must exit
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    let _ = app.update(wazoo_app::message::Message::OpenHelpModal);
    assert!(!app.cube.desktop_overlay, "Opening help modal must exit Mode 9");
    assert!(app.modals.help);

    // Close help modal
    let _ = app.update(wazoo_app::message::Message::CloseHelpModal);

    // 3. Enter Mode 9, toggle File Picker (h/f) -> Mode 9 must exit
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    let _ = app.update(wazoo_app::message::Message::ToggleFilePicker);
    assert!(!app.cube.desktop_overlay, "Opening file picker must exit Mode 9");
    assert!(app.drawers.show_file_picker);
}

#[test]
fn test_mode_9_toast_auto_hides_in_few_seconds() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // Toggle Desktop Screensaver (Mode 9)
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);

    // Toast should be visible with uniform default duration (2 seconds)
    assert!(app.overlay.toast_message.is_some());
    let remaining = app.overlay.toast_time_remaining;
    assert_eq!(
        remaining,
        wazoo_app::app::DEFAULT_TOAST_SECS,
        "Toast duration should match uniform DEFAULT_TOAST_SECS"
    );

    // After 1 second tick (WatchdogTick)
    let _ = app.update(wazoo_app::message::Message::WatchdogTick);
    assert_eq!(app.overlay.toast_time_remaining, remaining - 1);
    assert!(app.overlay.toast_message.is_some());

    // Tick remaining seconds until expiry
    for _ in 0..remaining - 1 {
        let _ = app.update(wazoo_app::message::Message::WatchdogTick);
    }

    // Now the toast must have automatically hidden
    assert_eq!(app.overlay.toast_time_remaining, 0);
    assert!(
        app.overlay.toast_message.is_none(),
        "Toast message must be None after watchdog timer ticks expire"
    );
}

#[test]
fn test_cube_present_hijacks_bookmark_keys_for_add_remove_cube() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // 1. Initially cubes are NOT present
    assert!(!app.cube.is_present());
    assert_eq!(app.cube.cubes.len(), 0);

    // 2. Spawn 3D cube (press 8)
    let _ = app.update(wazoo_app::message::Message::ToggleCubeScreensaver);
    assert!(app.cube.is_present());
    assert_eq!(app.cube.cubes.len(), 1);

    // 3. Pressing "+" should hijack bookmark key to spawn a 2nd cube
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("+".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 2);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Added 3D Cube #2! (Press [8] to toggle)")
    );

    // 4. Pressing "=" should also spawn a 3rd cube
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("=".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 3);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Added 3D Cube #3! (Press [8] to toggle)")
    );

    // 5. Pressing "-" should remove a cube (from 3 to 2)
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("-".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 2);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("3D Cube Removed (2 Remaining)")
    );

    // 6. Pressing "-" removes another cube (from 2 to 1)
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("-".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 1);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("3D Cube Removed (1 Remaining)")
    );

    // 7. Pressing "-" removes the last cube and exits cube screensaver
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("-".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 0);
    assert!(!app.cube.is_present());
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("3D Cube Removed")
    );

    // 8. With cubes now dismissed, pressing "+" no longer adds a cube (reverts to bookmark action)
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("+".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 0);
    assert!(!app.cube.is_present());

    // 9. Verify same hijacking works in Mode 9 Desktop Screensaver
    let _ = app.update(wazoo_app::message::Message::ToggleDesktopCubeScreensaver);
    assert!(app.cube.desktop_overlay);
    assert!(app.cube.is_present());
    assert_eq!(app.cube.cubes.len(), 1);

    // Press "+" in Mode 9 -> spawns 2nd cube
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("+".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 2);

    // Press "-" in Mode 9 -> removes 1 cube
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("-".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 1);

    // Press "-" again in Mode 9 -> removes last cube and exits Mode 9!
    let _ = app.update(wazoo_app::message::Message::KeyPressed(
        iced::keyboard::Key::Character("-".into()),
        iced::event::Status::Ignored,
    ));
    assert_eq!(app.cube.cubes.len(), 0);
    assert!(!app.cube.is_present());
    assert!(!app.cube.desktop_overlay);
}

#[test]
fn test_cube_sheen_toggle_and_persistence() {
    let (mut app, _) = wazoo_app::app::new_test_app();
    assert!(!app.settings.cube_sheen, "Cube sheen should default to false (disabled)");

    // Toggle sheen ON
    let _ = app.update(wazoo_app::message::Message::ToggleCubeSheen);
    assert!(app.settings.cube_sheen);

    // Verify it is saved to disk
    let reloaded = app.config_mgr.load_settings();
    assert!(reloaded.cube_sheen);

    // Toggle sheen back OFF
    let _ = app.update(wazoo_app::message::Message::ToggleCubeSheen);
    assert!(!app.settings.cube_sheen);

    let reloaded2 = app.config_mgr.load_settings();
    assert!(!reloaded2.cube_sheen);
}
