use super::common::{TempTestDir, press_char};
use wazoo_app::app::{WazooApp, new_test_app};
use wazoo_app::message::Message;
use wazoo_core::VideoRecord;

#[test]
fn test_session_videos_shuffle_persistence() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
    ];

    // Default should be shuffle mode
    assert!(app.default_shuffle_mode);

    // Toggle shuffle mode -> sequential
    let _ = app.update(Message::ToggleShuffleMode);
    assert!(!app.default_shuffle_mode);

    // Save session state
    app.save_session_state();

    // If there were players, session_videos records is_shuffle: false
    let loaded = app.config_mgr.load_settings();
    if let Some(first) = loaded.session_videos.first() {
        assert!(!first.is_shuffle);
    }
}

#[test]
fn test_scroll_mode_real_heights_and_recalculation() {
    let (mut app, _) = new_test_app();
    app.settings.window_bounds.width = 1920;
    app.settings.window_bounds.height = 1080;
    app.scroll_engine.set_window_size(1920.0, 1080.0);

    // Standard 16:9 widescreen video at 1920 width -> real height is 1080.0
    let h_16_9 = app.scroll_engine.item_height_for_aspect_ratio(16.0 / 9.0);
    assert!((h_16_9 - 1080.0).abs() < 1.0);

    // 4:3 video at 1920 width -> real height is 1440.0
    let h_4_3 = app.scroll_engine.item_height_for_aspect_ratio(4.0 / 3.0);
    assert!((h_4_3 - 1440.0).abs() < 1.0);

    // 9:16 vertical video at 1920 width -> real height is 3413.33
    let h_9_16 = app.scroll_engine.item_height_for_aspect_ratio(9.0 / 16.0);
    assert!((h_9_16 - 3413.33).abs() < 1.0);

    // When toggling scroll mode with items, stack initializes with real heights
    app.scroll_engine
        .init_stack_with_heights(&[(1, h_16_9), (2, h_4_3)]);
    assert_eq!(app.scroll_engine.items.get(&1).unwrap().y_pos, 0.0);
    assert_eq!(app.scroll_engine.items.get(&1).unwrap().height, h_16_9);
    assert_eq!(app.scroll_engine.items.get(&2).unwrap().y_pos, h_16_9);
    assert_eq!(app.scroll_engine.items.get(&2).unwrap().height, h_4_3);

    // Dynamic update on aspect ratio resolution
    let changed = app.scroll_engine.update_height(1, 1080.0);
    assert!(
        !changed,
        "Height did not change significantly, so returns false"
    );
    let changed = app.scroll_engine.update_height(1, 1200.0);
    assert!(changed, "Height changed by > 1px, so returns true");
    app.scroll_engine.recalculate_positions();
    assert_eq!(app.scroll_engine.items.get(&1).unwrap().y_pos, 0.0);
    assert_eq!(app.scroll_engine.items.get(&1).unwrap().height, 1200.0);
    assert_eq!(app.scroll_engine.items.get(&2).unwrap().y_pos, 1200.0);

    // Window resize updates window size and recalculates scroll item heights
    let _ = app.update(Message::WindowResized(
        iced::window::Id::unique(),
        iced::Size::new(1280.0, 720.0),
    ));
    assert_eq!(app.scroll_engine.window_width, 1280.0);
    assert_eq!(app.scroll_engine.window_height, 720.0);
    let resized_h_16_9 = app.scroll_engine.item_height_for_aspect_ratio(16.0 / 9.0);
    assert!((resized_h_16_9 - 720.0).abs() < 1.0);
}

#[test]
fn test_add_new_player_focuses_new_player_and_can_be_removed() {
    let _ = env_logger::builder().is_test(true).try_init();
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("wazoo_test_players");
    let f1 = temp_dir.path().join("v1.mp4");
    let f2 = temp_dir.path().join("v2.mp4");
    let f3 = temp_dir.path().join("v3.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);
    let _ = std::fs::File::create(&f3);

    app.available_videos = vec![
        VideoRecord::new(1, "V1", f1.to_string_lossy()),
        VideoRecord::new(2, "V2", f2.to_string_lossy()),
        VideoRecord::new(3, "V3", f3.to_string_lossy()),
    ];

    // Start with 1 player
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    assert_eq!(app.focused_idx, 0);
    let first_player_id = app.players[0].id;

    // Manually add a new player (such as pressing N)
    let _ = app.update(Message::AddNewPlayer);
    assert_eq!(app.players.len(), 2);
    assert_eq!(app.focused_idx, 1);
    let second_player_id = app.players[1].id;
    assert_ne!(first_player_id, second_player_id);
    assert_eq!(app.focused_player_id(), Some(second_player_id));
    assert_eq!(app.overlay.focus_border_ticks, 0);

    // Press X (RemoveFocusedPlayer) removes the newly added active player
    let _ = app.update(Message::RemoveFocusedPlayer);
    assert_eq!(app.players.len(), 1);
    assert_eq!(app.players[0].id, first_player_id);
    assert_eq!(app.focused_idx, 0);
    assert_eq!(app.focused_player_id(), Some(first_player_id));
    assert_eq!(app.overlay.focus_border_ticks, 0);
}

#[test]
fn test_prev_next_frame_keybind_dispatch() {
    let (mut app, _) = new_test_app();

    // Next frame ('.')
    press_char(&mut app, ".");
    assert!(app.overlay.ticks > 0);

    // Reset ticks to verify comma independently
    app.overlay.ticks = 0;

    // Prev frame (',')
    press_char(&mut app, ",");
    assert!(app.overlay.ticks > 0);
}

#[test]
fn test_player_by_id_helpers() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_player_helpers");
    let f1 = temp_dir.path().join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![VideoRecord::new(1, "V1", f1.to_string_lossy())];
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let p_id = app.players[0].id;

    // Test PlayerList helpers
    assert!(app.players.has_player(p_id));
    assert!(!app.players.has_player(999_999));
    assert_eq!(app.players.player_index(p_id), Some(0));
    assert_eq!(app.players.player_index(999_999), None);
    assert_eq!(app.players.player(p_id).map(|p| p.id), Some(p_id));
    assert!(app.players.player(999_999).is_none());

    // Test mutable player lookup
    if let Some(p) = app.players.player_mut(p_id) {
        assert_eq!(p.id, p_id);
    }
}

#[test]
fn test_mute_default_when_no_preference() {
    let (mut app, _) = new_test_app();
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Normal);

    let temp_dir = TempTestDir::new("test_mute_pref");
    let f1 = temp_dir.path().join("V1.mp4");
    let f2 = temp_dir.path().join("V2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        VideoRecord::new(1, "V1", f1.to_string_lossy()),
        VideoRecord::new(2, "V2", f2.to_string_lossy()),
    ];

    // Initial player should NOT be muted when opening app normally with no prior preference
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    assert!(!app.players[0].state.is_muted);

    let p0_id = app.players[0].id;

    // Advancing video preserves unmuted state
    let _ = app.update(Message::NextVideo(p0_id));
    assert!(!app.players[0].state.is_muted);

    // Playing file from drawer preserves unmuted state
    let _ = app.update(Message::PlayFileInFocused(f2.to_string_lossy().to_string()));
    assert!(!app.players[0].state.is_muted);
}

#[test]
fn test_mute_preference_preserved_when_user_explicitly_mutes() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_mute_explicit");
    let f1 = temp_dir.path().join("V1.mp4");
    let f2 = temp_dir.path().join("V2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        VideoRecord::new(1, "V1", f1.to_string_lossy()),
        VideoRecord::new(2, "V2", f2.to_string_lossy()),
    ];

    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let p0_id = app.players[0].id;
    assert!(!app.players[0].state.is_muted);

    // Explicitly mute player 0 (user sets preference)
    let _ = app.update(Message::TogglePlayerMute(p0_id));
    assert!(app.players[0].state.is_muted);

    // Navigating preserves user's explicit muted preference
    let _ = app.update(Message::NextVideo(p0_id));
    assert!(app.players[0].state.is_muted);
}

#[test]
fn test_scroll_mode_m_key_toggles_global_mute_and_spawns_with_preference() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("test_scroll_mute");
    let f1 = temp_dir.path().join("S1.mp4");
    let f2 = temp_dir.path().join("S2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        VideoRecord::new(1, "S1", f1.to_string_lossy()),
        VideoRecord::new(2, "S2", f2.to_string_lossy()),
    ];

    // Enter Scroll Mode -> scroll mode defaults to muted
    let _ = app.update(Message::ToggleScrollMode);
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Scroll);
    assert!(app.settings.scroll_mode_muted);
    assert!(app.scroll_engine.scroll_mode_muted);
    for p in &app.players {
        assert!(p.state.is_muted);
    }

    // Hit 'm' key -> ToggleMuteFocused in Scroll mode toggles scroll_mode_muted to false (unmuted)
    let _ = app.update(Message::ToggleMuteFocused);
    assert!(!app.settings.scroll_mode_muted);
    assert!(!app.scroll_engine.scroll_mode_muted);
    for p in &app.players {
        assert!(!p.state.is_muted);
    }
    if let Some(ref preloaded) = app.preloaded_player {
        assert!(!preloaded.state.is_muted);
    }

    // Trigger frame tick to attach preloaded player if ready -> must spawn unmuted
    let _ = app.update(Message::Tick);
    for p in &app.players {
        assert!(!p.state.is_muted);
    }

    // Hit 'm' key again -> toggles back to muted (true)
    let _ = app.update(Message::ToggleMuteFocused);
    assert!(app.settings.scroll_mode_muted);
    assert!(app.scroll_engine.scroll_mode_muted);
    for p in &app.players {
        assert!(p.state.is_muted);
    }
    if let Some(ref preloaded) = app.preloaded_player {
        assert!(preloaded.state.is_muted);
    }
}

#[test]
fn test_seek_relative_debounced_and_accumulated() {
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("wazoo_test_seek_debounce");
    let f1 = temp_dir.path().join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![VideoRecord::new(1, "V1", f1.to_string_lossy())];
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let p_id = app.players[0].id;
    app.focused_idx = 0;

    // Initially, no seek is debouncing
    assert!(app.players[0].seek_debounce.is_none());

    // 1. First seek forward: creates seek debounce state
    let _ = app.update(Message::SeekRelativeFocused(5.0));
    assert!(app.players[0].seek_debounce.is_some());
    let deb = app.players[0].seek_debounce.unwrap();
    assert_eq!(deb.ticks_remaining, wazoo_app::app::SEEK_DEBOUNCE_TICKS);
    assert_eq!(deb.accumulated_secs(), 5.0);
    assert_eq!(deb.target_secs, 5.0);
    assert_eq!(app.players[0].position().as_secs_f64(), 5.0);

    // Toast message is formatted with +5
    let toast = app.overlay.toast_message.as_ref().unwrap();
    assert!(toast.contains("+5"));

    // 2. Repeated keystrokes accumulate delta and target position
    let _ = app.update(Message::SeekRelativeFocused(5.0));
    let deb = app.players[0].seek_debounce.unwrap();
    assert_eq!(deb.ticks_remaining, wazoo_app::app::SEEK_DEBOUNCE_TICKS);
    assert_eq!(deb.accumulated_secs(), 10.0);
    assert_eq!(deb.target_secs, 10.0);
    assert_eq!(app.players[0].position().as_secs_f64(), 10.0);

    let toast = app.overlay.toast_message.as_ref().unwrap();
    assert!(toast.contains("+10"));

    // A third keystroke
    let _ = app.update(Message::SeekRelativeFocused(5.0));
    let deb = app.players[0].seek_debounce.unwrap();
    assert_eq!(deb.accumulated_secs(), 15.0);
    assert_eq!(deb.target_secs, 15.0);
    assert_eq!(app.players[0].position().as_secs_f64(), 15.0);

    // Opposing keystroke (Seek backward) reduces accumulated and target
    let _ = app.update(Message::SeekRelativeFocused(-5.0));
    let deb = app.players[0].seek_debounce.unwrap();
    assert_eq!(deb.accumulated_secs(), 10.0);
    assert_eq!(deb.target_secs, 10.0);

    // 3. Advancing Tick decrements debounce countdown
    let _ = app.update(Message::Tick);
    let deb = app.players[0].seek_debounce.unwrap();
    assert_eq!(deb.ticks_remaining, wazoo_app::app::SEEK_DEBOUNCE_TICKS - 1);

    // 4. Advancing all remaining ticks flushes the seek debounce
    while app.players[0].seek_debounce.is_some() {
        let _ = app.update(Message::Tick);
    }
    assert!(app.players[0].seek_debounce.is_none());

    // 5. Explicit seek cancels any active debounce
    let _ = app.update(Message::SeekRelativeFocused(5.0));
    assert!(app.players[0].seek_debounce.is_some());
    let _ = app.update(Message::Seek(p_id, std::time::Duration::from_secs(30)));
    assert!(app.players[0].seek_debounce.is_none());

    // 6. TogglePlay flushes any active debounce
    let _ = app.update(Message::SeekRelativeFocused(5.0));
    assert!(app.players[0].seek_debounce.is_some());
    let _ = app.update(Message::TogglePlay(p_id));
    assert!(app.players[0].seek_debounce.is_none());
}

#[test]
fn test_tv_static_crops_to_previous_video_aspect_ratio() {
    // 1. Verify crop_static_frame calculations for 16:9
    let raw_w = 500u32;
    let raw_h = 375u32;
    let raw_pixels = vec![255u8; (raw_w * raw_h * 4) as usize];
    let raw = wazoo_app::assets::RawStaticFrame {
        width: raw_w,
        height: raw_h,
        pixels: std::sync::Arc::from(raw_pixels),
    };

    let mut guard_16_9 = wazoo_media::pipeline::FrameData {
        width: 0,
        height: 0,
        pixels: Vec::new(),
        new_frame: false,
        frame_seq: 0,
    };
    WazooApp::crop_static_frame(&raw, 16.0 / 9.0, &mut guard_16_9);
    assert_eq!(guard_16_9.width, 500);
    assert_eq!(guard_16_9.height, 281);
    assert_eq!(guard_16_9.pixels.len(), (500 * 281 * 4) as usize);
    let aspect_16_9 = guard_16_9.width as f32 / guard_16_9.height as f32;
    assert!((aspect_16_9 - (16.0 / 9.0)).abs() < 0.01);

    // 2. Verify crop_static_frame calculations for 4:3 (native)
    let mut guard_4_3 = wazoo_media::pipeline::FrameData {
        width: 0,
        height: 0,
        pixels: Vec::new(),
        new_frame: false,
        frame_seq: 0,
    };
    WazooApp::crop_static_frame(&raw, 4.0 / 3.0, &mut guard_4_3);
    assert_eq!(guard_4_3.width, 500);
    assert_eq!(guard_4_3.height, 375);
    assert_eq!(guard_4_3.pixels.len(), (500 * 375 * 4) as usize);

    // 3. Verify crop_static_frame calculations for 9:16 vertical video
    let mut guard_9_16 = wazoo_media::pipeline::FrameData {
        width: 0,
        height: 0,
        pixels: Vec::new(),
        new_frame: false,
        frame_seq: 0,
    };
    WazooApp::crop_static_frame(&raw, 9.0 / 16.0, &mut guard_9_16);
    assert_eq!(guard_9_16.height, 375);
    assert_eq!(guard_9_16.width, 211);
    let aspect_9_16 = guard_9_16.width as f32 / guard_9_16.height as f32;
    assert!((aspect_9_16 - (9.0 / 16.0)).abs() < 0.01);

    // 4. Verify player aspect ratio retention across loading state
    let (mut app, _) = new_test_app();
    let temp_dir = TempTestDir::new("wazoo_test_static");
    let f1 = temp_dir.path().join("test.mp4");
    let _ = std::fs::File::create(&f1);
    let p1 = f1.to_string_lossy().to_string();
    let v1 = VideoRecord::new(1, "test", &p1);
    app.available_videos = vec![v1];

    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);

    // Simulate 16:9 video aspect ratio on player 0
    app.players[0].last_aspect_ratio = Some(16.0 / 9.0);
    app.players[0].start_loading();
    assert!(app.players[0].is_loading);
    assert_eq!(app.players[0].aspect_ratio(), Some(16.0 / 9.0));

    // Update static frames and verify player 0's static frame is cropped to 16:9
    app.settings.loading_indicator = wazoo_core::LoadingIndicator::TvStatic;
    app.update_static_frame();

    let p_frame = app.players[0].static_frame.lock().unwrap();
    assert_eq!(p_frame.width, 500);
    assert_eq!(p_frame.height, 281);
    let static_ar = p_frame.width as f32 / p_frame.height as f32;
    assert!((static_ar - (16.0 / 9.0)).abs() < 0.01);
    drop(p_frame);

    // 5. Verify multi-player independence (Player 0 is 16:9, Player 1 is 4:3)
    let f2 = temp_dir.path().join("test2.mp4");
    let _ = std::fs::File::create(&f2);
    let p2 = f2.to_string_lossy().to_string();
    let v2 = VideoRecord::new(2, "test2", &p2);
    app.available_videos.push(v2);

    let _ = app.update(Message::SetPlayerCount(2));
    assert_eq!(app.players.len(), 2);

    app.players[0].last_aspect_ratio = Some(16.0 / 9.0);
    app.players[0].start_loading();

    app.players[1].last_aspect_ratio = Some(4.0 / 3.0);
    app.players[1].start_loading();

    app.update_static_frame();

    let frame0 = app.players[0].static_frame.lock().unwrap();
    assert_eq!(frame0.width, 500);
    assert_eq!(frame0.height, 281);

    let frame1 = app.players[1].static_frame.lock().unwrap();
    assert_eq!(frame1.width, 500);
    assert_eq!(frame1.height, 375);
}

#[test]
fn test_async_four_players_animate_tv_static_while_buffering_network_streams() {
    let (mut app, _) = new_test_app();
    app.settings.loading_indicator = wazoo_core::LoadingIndicator::TvStatic;
    app.settings.use_standby_player = false;
    app.available_videos = vec![
        VideoRecord::new(1, "Stream 1", "http://127.0.0.1:9/v1.mp4"),
        VideoRecord::new(2, "Stream 2", "http://127.0.0.1:9/v2.mp4"),
        VideoRecord::new(3, "Stream 3", "http://127.0.0.1:9/v3.mp4"),
        VideoRecord::new(4, "Stream 4", "http://127.0.0.1:9/v4.mp4"),
    ];

    // Pressing 4 immediately spawns 4 loading players with placeholder handles (no UI blocking)
    let _ = app.update(Message::SetPlayerCount(4));
    assert_eq!(app.players.len(), 4);
    for p in &app.players {
        assert!(p.is_loading);
        assert!(p.is_placeholder());
        assert!(p.is_network_stream());
    }

    // Record initial static frame sequence & pixels for all 4 players
    let initial_seqs: Vec<u64> = app
        .players
        .iter()
        .map(|p| p.static_frame.lock().unwrap().frame_seq)
        .collect();
    let initial_pixels: Vec<Vec<u8>> = app
        .players
        .iter()
        .map(|p| p.static_frame.lock().unwrap().pixels.clone())
        .collect();

    // Advance several 60Hz ticks while streams are still buffering
    for _ in 0..8 {
        let _ = app.update(Message::Tick);
    }

    // All 4 players must still be loading, and every player's TV static frame must have animated
    for (idx, p) in app.players.iter().enumerate() {
        assert!(p.is_loading);
        let guard = p.static_frame.lock().unwrap();
        assert!(
            guard.frame_seq > initial_seqs[idx],
            "Player {idx} static frame_seq did not advance ({} -> {})",
            initial_seqs[idx],
            guard.frame_seq
        );
        assert_ne!(
            guard.pixels, initial_pixels[idx],
            "Player {idx} static pixel buffer did not change across ticks"
        );
    }

    // Watchdog should tolerate slow WiFi buffering past 10s (up to 30s) without skipping
    for _ in 0..12 {
        let _ = app.update(Message::WatchdogTick);
    }
    for p in &app.players {
        assert!(p.is_loading);
        assert_eq!(p.loading_ticks, 12);
    }
}

#[test]
fn test_player_handle_ready_preserves_state_and_rejects_stale_generation() {
    let (mut app, _) = new_test_app();
    app.settings.use_standby_player = false;
    app.available_videos = vec![
        VideoRecord::new(1, "Stream 1", "http://127.0.0.1:9/v1.mp4"),
        VideoRecord::new(2, "Stream 2", "http://127.0.0.1:9/v2.mp4"),
    ];

    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let pid = app.players[0].id;
    let initial_path = app.players[0].state.path.clone();
    let generation = app.players[0].load_generation;

    // User lowers volume, mutes, and pauses while loading is still in flight
    let _ = app.update(Message::SetVolume(pid, 0.35));
    let _ = app.update(Message::TogglePlayerMute(pid));
    let _ = app.update(Message::TogglePlay(pid));
    assert!(app.players[0].state.is_muted);
    assert!((app.players[0].state.volume - 0.35).abs() < 0.001);
    assert!(!app.players[0].state.is_playing);

    // Stale generation result is ignored
    let stale_handle = wazoo_media::VideoHandle::new_placeholder(
        pid,
        "http://127.0.0.1:9/stale.mp4",
        "Stale",
        wazoo_media::BufferConfig::default(),
        wazoo_media::StartTime::Beginning,
    );
    let _ = app.update(Message::PlayerHandleReady {
        player_id: pid,
        generation: generation.wrapping_add(99),
        result: std::sync::Arc::new(std::sync::Mutex::new(Some(Ok(stale_handle)))),
    });
    assert_eq!(app.players[0].state.path, initial_path);

    // Matching generation result attaches and inherits user state
    let ready_handle = wazoo_media::VideoHandle::new_idle(wazoo_media::BufferConfig::default())
        .expect("idle libmpv handle");
    let _ = app.update(Message::PlayerHandleReady {
        player_id: pid,
        generation,
        result: std::sync::Arc::new(std::sync::Mutex::new(Some(Ok(ready_handle)))),
    });
    assert!(!app.players[0].is_placeholder());
    assert!(app.players[0].state.is_muted);
    assert!((app.players[0].state.volume - 0.35).abs() < 0.001);
    assert!(!app.players[0].state.is_playing);
}

#[test]
fn test_next_video_loads_in_playing_state() {
    let (mut app, _) = new_test_app();
    app.settings.use_standby_player = false;
    app.default_shuffle_mode = false;
    app.available_videos = vec![
        VideoRecord::new(1, "Stream 1", "http://127.0.0.1:9/v1.mp4"),
        VideoRecord::new(2, "Stream 2", "http://127.0.0.1:9/v2.mp4"),
    ];

    let _ = app.update(Message::SetPlayerCount(1));
    let pid = app.players[0].id;

    // Pressing Up (NextVideo) stops old playback before starting async load of next video;
    // the newly loaded video must start playing (not paused).
    let _ = app.update(Message::NextVideo(pid));
    assert!(app.players[0].is_loading);
    assert!(app.players[0].state.is_playing);

    let generation = app.players[0].load_generation;
    let ready_handle = wazoo_media::VideoHandle::new_idle(wazoo_media::BufferConfig::default())
        .expect("idle libmpv handle");
    let _ = app.update(Message::PlayerHandleReady {
        player_id: pid,
        generation,
        result: std::sync::Arc::new(std::sync::Mutex::new(Some(Ok(ready_handle)))),
    });
    assert!(app.players[0].state.is_playing);
}
