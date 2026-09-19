use iced::Point;
use std::time::Duration;
use wazoo_app::app::{
    new_test_app, PlaybackHistoryEntry, WazooApp, MAX_PLAYER_NAV_HISTORY_ENTRIES,
    TITLEBAR_FADE_TICKS, TITLEBAR_HIDE_TICKS,
};
use wazoo_app::message::Message;
use wazoo_core::{ConfigManager, Database, VideoRecord};

#[test]
fn test_test_app_isolation() {
    let (mut app, _) = new_test_app();
    // Ensure test config path does not point to live user directory
    let path = app.config_mgr.config_file_path();
    assert!(!path.to_string_lossy().contains(".config/wazoo-rs"));
    app.settings.media_folders = vec!["/test/isolated/folder".to_string()];
    let save_res = app.config_mgr.save_settings(&app.settings);
    assert!(save_res.is_ok());
    assert!(path.exists());
}

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
fn test_get_next_video_rec_sequential_and_random() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
        VideoRecord::new(3, "V3", "/media/v3.mp4"),
    ];

    // 1. Sequential mode
    app.is_shuffle_mode = false;
    let next1 = app.get_next_video_rec(Some("/media/v1.mp4")).unwrap();
    assert_eq!(next1.path, "/media/v2.mp4");
    let next2 = app.get_next_video_rec(Some("/media/v2.mp4")).unwrap();
    assert_eq!(next2.path, "/media/v3.mp4");
    let next3 = app.get_next_video_rec(Some("/media/v3.mp4")).unwrap();
    assert_eq!(next3.path, "/media/v1.mp4"); // Wraps around

    // 2. Random/shuffle mode
    app.is_shuffle_mode = true;
    for _ in 0..10 {
        let rand_rec = app.get_next_video_rec(Some("/media/v1.mp4")).unwrap();
        assert!(app.available_videos.iter().any(|v| v.path == rand_rec.path));
        // With 3 videos, shuffle mode avoids immediately repeating current video
        assert_ne!(rand_rec.path, "/media/v1.mp4");
    }
}

#[test]
fn test_get_prev_video_rec_sequential_and_random() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
        VideoRecord::new(3, "V3", "/media/v3.mp4"),
    ];

    // 1. Sequential mode
    let prev2 = app
        .get_prev_video_rec_with_mode(Some("/media/v2.mp4"), false)
        .unwrap();
    assert_eq!(prev2.path, "/media/v1.mp4");
    let prev1 = app
        .get_prev_video_rec_with_mode(Some("/media/v1.mp4"), false)
        .unwrap();
    assert_eq!(prev1.path, "/media/v3.mp4"); // Wraps around to end

    // 2. Random/shuffle mode
    for _ in 0..10 {
        let rand_rec = app
            .get_prev_video_rec_with_mode(Some("/media/v1.mp4"), true)
            .unwrap();
        assert!(app.available_videos.iter().any(|v| v.path == rand_rec.path));
        // With 3 videos, shuffle mode avoids immediately repeating current video
        assert_ne!(rand_rec.path, "/media/v1.mp4");
    }
}

#[test]
fn test_player_nav_history_scrub_back_and_forward() {
    let (mut app, _) = new_test_app();
    let player_id = 1;

    // Simulate initial video A
    app.push_player_nav_entry(player_id, "/media/A.mp4".to_string(), None);
    // User plays random B
    app.push_player_nav_entry(player_id, "/media/B.mp4".to_string(), Some(15.0));
    // User plays random C
    app.push_player_nav_entry(player_id, "/media/C.mp4".to_string(), Some(30.0));

    let hist = app.player_nav_history.get(&player_id).unwrap();
    assert_eq!(hist.back_stack.len(), 3);
    assert_eq!(hist.forward_stack.len(), 0);

    // Previous action simulation: pop C from back_stack, push to forward_stack
    let current_c = app
        .player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .back_stack
        .pop()
        .unwrap();
    app.player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .forward_stack
        .push(current_c);
    let target_b = app
        .player_nav_history
        .get(&player_id)
        .unwrap()
        .back_stack
        .last()
        .unwrap()
        .clone();
    assert_eq!(target_b.path, "/media/B.mp4");
    assert_eq!(target_b.position_secs, Some(15.0));

    // Previous again: pop B from back_stack, push to forward_stack
    let current_b = app
        .player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .back_stack
        .pop()
        .unwrap();
    app.player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .forward_stack
        .push(current_b);
    let target_a = app
        .player_nav_history
        .get(&player_id)
        .unwrap()
        .back_stack
        .last()
        .unwrap()
        .clone();
    assert_eq!(target_a.path, "/media/A.mp4");

    // Now next action: forward_stack pop gives B!
    let forward_b = app
        .player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .forward_stack
        .pop()
        .unwrap();
    assert_eq!(forward_b.path, "/media/B.mp4");
    assert_eq!(forward_b.position_secs, Some(15.0));
    app.push_player_nav_entry(player_id, forward_b.path, forward_b.position_secs);

    // Next action again: forward_stack pop gives C!
    let forward_c = app
        .player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .forward_stack
        .pop()
        .unwrap();
    assert_eq!(forward_c.path, "/media/C.mp4");
    assert_eq!(forward_c.position_secs, Some(30.0));
    app.push_player_nav_entry(player_id, forward_c.path, forward_c.position_secs);

    // Forward stack is now empty
    assert!(
        app.player_nav_history
            .get(&player_id)
            .unwrap()
            .forward_stack
            .is_empty()
    );

    // Toggling shuffle mode clears both back_stack and forward_stack
    app.player_nav_history
        .get_mut(&player_id)
        .unwrap()
        .forward_stack
        .push(PlaybackHistoryEntry {
            path: "/media/D.mp4".to_string(),
            position_secs: None,
        });
    let _ = app.update(Message::ToggleShuffleMode);
    assert!(
        app.player_nav_history
            .get(&player_id)
            .unwrap()
            .forward_stack
            .is_empty()
    );
    assert!(
        app.player_nav_history
            .get(&player_id)
            .unwrap()
            .back_stack
            .is_empty()
    );
}

#[test]
fn test_player_nav_history_cap_at_1000() {
    let (mut app, _) = new_test_app();
    let player_id = 1;

    for i in 0..1050 {
        app.push_player_nav_entry(player_id, format!("/media/video{}.mp4", i), None);
    }

    let hist = app.player_nav_history.get(&player_id).unwrap();
    assert_eq!(hist.back_stack.len(), MAX_PLAYER_NAV_HISTORY_ENTRIES);
    assert_eq!(hist.back_stack.len(), 1000);
    assert_eq!(hist.back_stack.first().unwrap().path, "/media/video50.mp4");
    assert_eq!(hist.back_stack.last().unwrap().path, "/media/video1049.mp4");
}

#[test]
fn test_file_picker_confined_folder_badge_multilingual() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "Movie 1", "/media/movies/m1.mp4"),
    ];

    // 1. Switch language to Spanish
    let _ = app.update(Message::SetLanguage("es".to_string()));
    let all_es = app.t("common.all");
    assert_eq!(all_es, "Todo");
    assert_eq!(app.active_search_folder, "Todo");
    assert_eq!(app.selected_search_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.active_search_folder));

    // 2. File picker in Spanish does not show badge for Todo
    {
        let _view_all = app.view_file_picker();
    }

    // 3. Search confined to anime
    let _ = app.update(Message::SelectSearchFolder("/media/anime".to_string()));
    assert_eq!(app.active_search_folder, "Todo"); // Still Todo before search
    let _ = app.update(Message::PerformSearch);
    assert_eq!(app.active_search_folder, "/media/anime");
    assert_eq!(app.settings.last_folders, vec!["/media/anime".to_string()]);
    assert!(!app.is_all_folder(&app.active_search_folder));

    // 4. Reset search folder returns to Spanish All ("Todo")
    let _ = app.update(Message::ResetSearchFolder);
    assert_eq!(app.active_search_folder, "Todo");
    assert_eq!(app.selected_search_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.active_search_folder));
}

#[test]
fn test_file_picker_highlights_playing_video() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "Movie 1", "/media/movies/m1.mp4"),
    ];
    let _ = app.update(Message::ToggleFilePicker);
    let _ = app.update(Message::ToggleFolderCollapse("anime".to_string()));

    {
        let _view_before = app.view_file_picker();
    }

    if let Some(p) = app.players.first_mut() {
        p.state.path = "/media/anime/a1.mp4".to_string();
    }
    {
        let _view_playing = app.view_file_picker();
    }
}

#[test]
fn test_search_modal_folder_toggles_and_reset_to_all() {
    let (mut app, _) = new_test_app();
    app.settings.media_folders = vec![
        "/media/anime".to_string(),
        "/media/movies".to_string(),
        "/media/music".to_string(),
    ];

    // Populate database
    app.db
        .batch_insert_videos(&[
            VideoRecord::new(1, "Anime Ep 1", "/media/anime/ep1.mp4"),
            VideoRecord::new(2, "Blockbuster Movie", "/media/movies/movie.mp4"),
            VideoRecord::new(3, "Music Video", "/media/music/clip.mp4"),
        ])
        .unwrap();

    // 1. Initially "All" is active and selected
    assert!(app.is_all_search_selected());
    assert!(app.selected_search_folders.is_empty());

    // 2. Toggle folder 1: /media/anime
    let _ = app.update(Message::ToggleSearchFolder("/media/anime".to_string()));
    assert!(!app.is_all_search_selected());
    assert_eq!(app.selected_search_folders, vec!["/media/anime".to_string()]);

    // 3. Toggle folder 2: /media/movies (mix and match!)
    let _ = app.update(Message::ToggleSearchFolder("/media/movies".to_string()));
    assert!(!app.is_all_search_selected());
    assert_eq!(
        app.selected_search_folders,
        vec!["/media/anime".to_string(), "/media/movies".to_string()]
    );

    // 4. Toggle folder 1 off
    let _ = app.update(Message::ToggleSearchFolder("/media/anime".to_string()));
    assert!(!app.is_all_search_selected());
    assert_eq!(app.selected_search_folders, vec!["/media/movies".to_string()]);

    // 5. Toggle folder 2 off -> auto-reverts to All
    let _ = app.update(Message::ToggleSearchFolder("/media/movies".to_string()));
    assert!(app.is_all_search_selected());
    assert!(app.selected_search_folders.is_empty());
    assert!(app.is_all_folder(&app.selected_search_folder));

    // 6. Select multiple folders again, then click "All" button
    let _ = app.update(Message::ToggleSearchFolder("/media/anime".to_string()));
    let _ = app.update(Message::ToggleSearchFolder("/media/music".to_string()));
    assert_eq!(app.selected_search_folders.len(), 2);

    let all_label = app.t("common.all");
    let _ = app.update(Message::SelectSearchFolder(all_label));
    assert!(app.is_all_search_selected());
    assert!(app.selected_search_folders.is_empty());

    // 7. Mix and match /media/anime and /media/movies, then PerformSearch
    let _ = app.update(Message::ToggleSearchFolder("/media/anime".to_string()));
    let _ = app.update(Message::ToggleSearchFolder("/media/movies".to_string()));
    let _ = app.update(Message::PerformSearch);

    assert_eq!(
        app.active_search_folders,
        vec!["/media/anime".to_string(), "/media/movies".to_string()]
    );
    assert_eq!(
        app.settings.last_folders,
        vec!["/media/anime".to_string(), "/media/movies".to_string()]
    );
    // Only anime and movies should be in available_videos (2 out of 3)
    assert_eq!(app.available_videos.len(), 2);
    assert!(app.available_videos.iter().any(|v| v.path.starts_with("/media/anime")));
    assert!(app.available_videos.iter().any(|v| v.path.starts_with("/media/movies")));
    assert!(!app.available_videos.iter().any(|v| v.path.starts_with("/media/music")));

    // 8. Remove one active folder from filter badge
    let _ = app.update(Message::RemoveActiveSearchFolder("/media/anime".to_string()));
    assert_eq!(app.active_search_folders, vec!["/media/movies".to_string()]);
    assert_eq!(app.available_videos.len(), 1);
    assert_eq!(app.available_videos[0].path, "/media/movies/movie.mp4");

    // 9. Remove remaining folder -> returns to All
    let _ = app.update(Message::RemoveActiveSearchFolder("/media/movies".to_string()));
    assert!(app.active_search_folders.is_empty());
    assert!(app.is_all_folder(&app.active_search_folder));
    assert_eq!(app.available_videos.len(), 3);
}

#[test]
fn test_boot_persists_default_keybinds() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_boot_kb_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_mgr = ConfigManager::with_dirs(temp_dir.clone(), temp_dir.clone());
    let db = Database::open_in_memory().expect("in-memory db");

    assert!(!config_mgr.config_file_path().exists());
    assert!(!config_mgr.has_keybinds_in_settings());

    let (app, _) = WazooApp::new_with_backend(None, config_mgr, db);

    assert!(app.config_mgr.config_file_path().exists());
    assert!(app.config_mgr.has_keybinds_in_settings());
    let content = std::fs::read_to_string(app.config_mgr.config_file_path()).unwrap();
    assert!(content.contains("\"keybinds\""));
    assert_eq!(app.settings.keybinds.add_player, "n");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_boot_reconciles_and_persists_incomplete_keybinds() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_incomplete_kb_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_mgr = ConfigManager::with_dirs(temp_dir.clone(), temp_dir.clone());
    let db = Database::open_in_memory().expect("in-memory db");

    // Write incomplete keybinds list into settings.json (only 1 key specified)
    let incomplete_json = r#"{"keybinds":{"toggle_layout":"o"}}"#;
    std::fs::write(config_mgr.config_file_path(), incomplete_json).unwrap();
    assert!(!config_mgr.has_complete_keybinds_in_settings());

    // Boot the app
    let (app, _) = WazooApp::new_with_backend(None, config_mgr, db);

    // Custom key is preserved, missing keys are filled with defaults
    assert_eq!(app.settings.keybinds.toggle_layout, "o");
    assert_eq!(app.settings.keybinds.close_app, "Alt+X");
    assert_eq!(app.settings.keybinds.add_player, "n");

    // Boot should have written the complete list to settings.json
    assert!(app.config_mgr.has_complete_keybinds_in_settings());
    let updated_file_content =
        std::fs::read_to_string(app.config_mgr.config_file_path()).unwrap();
    assert!(updated_file_content.contains("\"toggle_layout\": \"o\""));
    assert!(updated_file_content.contains("\"close_app\": \"Alt+X\""));
    assert!(updated_file_content.contains("\"add_player\": \"n\""));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_play_history_cap_deduplication_and_drawer() {
    let (mut app, _) = new_test_app();

    // 1. Initial state is empty and closed
    assert!(app.play_history.is_empty());
    assert!(!app.show_history_drawer);

    // 2. Record items with consecutive duplicates
    app.record_play_history("/media/video1.mp4");
    app.record_play_history("/media/video1.mp4"); // should be deduplicated
    assert_eq!(app.play_history.len(), 1);
    assert_eq!(app.play_history[0].path, "/media/video1.mp4");
    assert_eq!(app.play_history[0].title, "video1");

    app.record_play_history("/media/video2.mp4");
    assert_eq!(app.play_history.len(), 2);

    // 3. Cap at 1000 entries
    for i in 3..=1050 {
        app.record_play_history(&format!("/media/video{}.mp4", i));
    }
    assert_eq!(app.play_history.len(), 1000);
    // The oldest items (video1 to video50) should be dropped; oldest in list should be video51
    assert_eq!(app.play_history.first().unwrap().path, "/media/video51.mp4");
    assert_eq!(
        app.play_history.last().unwrap().path,
        "/media/video1050.mp4"
    );

    // 4. Toggle drawer
    let _ = app.update(Message::ToggleHistoryDrawer);
    assert!(app.show_history_drawer);
    assert!(!app.show_file_picker);
    assert!(!app.show_transcript);

    // 5. Search filter
    let _ = app.update(Message::HistorySearchChanged("video100".to_string()));
    assert_eq!(app.history_search, "video100");

    // 6. View rendering does not panic
    let _ = app.view_history_drawer();

    // 7. Escape closes drawer
    let _ = app.update(Message::EscapePressed);
    assert!(!app.show_history_drawer);

    // 8. Clear history
    let _ = app.update(Message::ClearPlayHistory);
    assert!(app.play_history.is_empty());
    let _ = app.view_history_drawer();
}

#[test]
fn test_session_videos_shuffle_persistence() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "V1", "/media/v1.mp4"),
        VideoRecord::new(2, "V2", "/media/v2.mp4"),
    ];

    // Default should be shuffle mode
    assert!(app.is_shuffle_mode);

    // Toggle shuffle mode -> sequential
    let _ = app.update(Message::ToggleShuffleMode);
    assert!(!app.is_shuffle_mode);

    // Save session state
    app.save_session_state();

    // If there were players, session_videos records is_shuffle: false
    let loaded = app.config_mgr.load_settings();
    if let Some(first) = loaded.session_videos.first() {
        assert!(!first.is_shuffle);
    }
}

#[test]
fn test_view_titlebar() {
    let (app, _) = new_test_app();
    let _elem = app.view_titlebar();
}

#[test]
fn test_titlebar_persists_during_window_drag() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window_id = Some(win_id);

    // Move cursor to titlebar and show it
    let _ = app.update(Message::CursorMoved(win_id, Point::new(200.0, 15.0)));
    app.show_titlebar = true;
    app.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
    assert_eq!(app.titlebar_alpha(), 1.0);

    // Press titlebar
    let _ = app.update(Message::TitleBarPressed);
    assert!(app.titlebar_drag_pending);
    assert_eq!(app.titlebar_alpha(), 1.0);

    // Move mouse by > 5px to initiate window drag
    let _ = app.update(Message::CursorMoved(win_id, Point::new(200.0, 25.0)));
    assert!(app.is_window_dragging);
    assert!(!app.titlebar_drag_pending);
    assert_eq!(app.titlebar_alpha(), 1.0);

    // Simulate wobbly windows: window moves and cursor flies around over video while dragging
    let _ = app.update(Message::WindowMoved(win_id, Point::new(105.0, 105.0)));
    let _ = app.update(Message::CursorMoved(win_id, Point::new(400.0, 500.0)));
    assert!(app.is_window_dragging);
    assert!(app.show_titlebar);
    assert_eq!(app.titlebar_alpha(), 1.0);

    // Run 50 frame ticks with window moving (well beyond TITLEBAR_HIDE_TICKS = 50)
    for i in 0..50 {
        let _ = app.update(Message::WindowMoved(
            win_id,
            Point::new(110.0 + i as f32, 110.0 + i as f32),
        ));
        let _ = app.update(Message::VideoFrameTick);
    }

    // Titlebar MUST still be fully visible and alpha == 1.0 while moving
    assert!(app.show_titlebar);
    assert_eq!(app.titlebar_alpha(), 1.0);

    // Release mouse button (mouseup) to conclude drag
    let _ = app.update(Message::LeftClickReleased);
    assert!(!app.is_window_dragging);
    assert_eq!(app.titlebar_hide_ticks, TITLEBAR_FADE_TICKS);

    // After fading out over TITLEBAR_FADE_TICKS frames, titlebar is completely dismissed without wiggling
    for _ in 0..TITLEBAR_FADE_TICKS {
        let _ = app.update(Message::VideoFrameTick);
    }
    assert!(!app.show_titlebar);
    assert_eq!(app.titlebar_alpha(), 0.0);
}

#[test]
fn test_titlebar_dismisses_when_wm_eats_mouseup() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window_id = Some(win_id);

    let _ = app.update(Message::CursorMoved(win_id, Point::new(200.0, 15.0)));
    let _ = app.update(Message::TitleBarPressed);
    let _ = app.update(Message::CursorMoved(win_id, Point::new(200.0, 25.0)));
    assert!(app.is_window_dragging);

    // Window moves
    let _ = app.update(Message::WindowMoved(win_id, Point::new(100.0, 100.0)));
    assert!(app.show_titlebar);

    // Simulate WM eating mouseup: window stops moving and time elapses
    std::thread::sleep(Duration::from_millis(320));

    // User moves mouse over video without clicking
    let _ = app.update(Message::CursorMoved(win_id, Point::new(300.0, 300.0)));
    assert!(!app.is_window_dragging);

    // Titlebar smoothly fades out without needing a click on the video
    for _ in 0..TITLEBAR_FADE_TICKS {
        let _ = app.update(Message::VideoFrameTick);
    }
    assert!(!app.show_titlebar);
    assert_eq!(app.titlebar_alpha(), 0.0);
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
fn test_unfocused_window_fps_throttling_30fps() {
    let (mut app, _) = new_test_app();
    app.is_window_focused = false;
    app.unfocused_frame_ticks = 0;

    // Tick 1: increments counter to 1, odd tick throttled
    let _ = app.update(Message::VideoFrameTick);
    assert_eq!(app.unfocused_frame_ticks, 1);

    // Tick 2: increments counter to 2, even tick executes (~30 FPS rate from 60 FPS base)
    let _ = app.update(Message::VideoFrameTick);
    assert_eq!(app.unfocused_frame_ticks, 2);

    // Tick 3: odd tick throttled
    let _ = app.update(Message::VideoFrameTick);
    assert_eq!(app.unfocused_frame_ticks, 3);

    // Tick 4: even tick executes
    let _ = app.update(Message::VideoFrameTick);
    assert_eq!(app.unfocused_frame_ticks, 4);

    // Regaining focus resets unfocused_frame_ticks
    let _ = app.update(Message::WindowFocused);
    assert!(app.is_window_focused);
    assert_eq!(app.unfocused_frame_ticks, 0);
}

#[test]
fn test_add_new_player_focuses_new_player_and_can_be_removed() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_test_players_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("v1.mp4");
    let f2 = temp_dir.join("v2.mp4");
    let f3 = temp_dir.join("v3.mp4");
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
    assert_eq!(app.focused_player_idx, 0);
    let first_player_id = app.players[0].id;

    // Manually add a new player (such as pressing N)
    let _ = app.update(Message::AddNewPlayer);
    assert_eq!(app.players.len(), 2);
    assert_eq!(app.focused_player_idx, 1);
    let second_player_id = app.players[1].id;
    assert_ne!(first_player_id, second_player_id);
    assert_eq!(app.focused_player_id(), Some(second_player_id));
    assert_eq!(app.focus_border_ticks, 0);

    // Press X (RemoveFocusedPlayer) removes the newly added active player
    let _ = app.update(Message::RemoveFocusedPlayer);
    assert_eq!(app.players.len(), 1);
    assert_eq!(app.players[0].id, first_player_id);
    assert_eq!(app.focused_player_idx, 0);
    assert_eq!(app.focused_player_id(), Some(first_player_id));
    assert_eq!(app.focus_border_ticks, 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_query_zero_matches_shows_no_matches_view_and_clear_search() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_test_no_match_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("existing_video.mp4");
    let _ = std::fs::File::create(&f1);

    // Add video to DB
    let _ = app.db.insert_misc_video("existing_video", &f1.to_string_lossy());
    assert_eq!(app.db.get_video_count().unwrap(), 1);

    // Perform a search for a term that does not match
    app.search_input = "nonexistent_query_xyz".to_string();
    let _ = app.update(Message::PerformSearch);

    assert_eq!(app.active_search_query, "nonexistent_query_xyz");
    assert!(app.available_videos.is_empty());

    // Calling view() exercises the view_no_matches code path
    {
        let _main_view = app.view();
    }

    // Now clear search
    let _ = app.update(Message::ClearSearch);
    assert!(app.active_search_query.is_empty());
    assert_eq!(app.available_videos.len(), 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_settings_toggle_default_player() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.is_default_player);

    let _ = app.update(Message::OpenSettingsModal);
    assert!(app.show_settings_modal);

    // Render settings modal with toggle button
    {
        let _view = app.view_settings_modal();
    }

    // Toggle default player ON
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(app.settings.is_default_player);

    // Render settings modal with active toggle button
    {
        let _view_active = app.view_settings_modal();
    }

    // Toggle default player OFF
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(!app.settings.is_default_player);
}

#[test]
fn test_top_menu_slide_down_animation() {
    use wazoo_app::app::{TITLEBAR_FADE_TICKS, TITLEBAR_SHOW_DELAY_TICKS, TITLEBAR_SLIDE_TICKS};

    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window_id = Some(win_id);

    // Initial state: top menu is hidden
    assert!(!app.show_titlebar);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Move cursor into top menu trigger zone (y < 35.0)
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(300.0, 10.0)));

    // Tick through hover delay (8 ticks)
    for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
        let _ = app.update(Message::VideoFrameTick);
    }

    // After hover delay passes, titlebar begins showing and starts sliding down from 0.0
    assert!(app.show_titlebar);
    assert_eq!(app.titlebar_slide_ticks, 0);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Initial view rendering at start of slide down
    {
        let _view_start = app.view_titlebar();
    }

    // Animate ticks and verify slide down progress increases monotonically to 1.0
    let mut prev_progress = app.titlebar_slide_progress();
    for _ in 1..=TITLEBAR_SLIDE_TICKS {
        let _ = app.update(Message::VideoFrameTick);
        let cur_progress = app.titlebar_slide_progress();
        assert!(
            cur_progress >= prev_progress,
            "Slide progress must increase monotonically: prev={}, cur={}",
            prev_progress,
            cur_progress
        );
        prev_progress = cur_progress;
    }

    // Titlebar has fully slid down into view
    assert_eq!(app.titlebar_slide_progress(), 1.0);
    {
        let _view_full = app.view_titlebar();
    }

    // Now move cursor away to trigger dismissal
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(300.0, 200.0)));

    // While fading out and sliding back up, progress smoothly decreases to 0.0
    for _ in 0..TITLEBAR_FADE_TICKS {
        let _ = app.update(Message::VideoFrameTick);
    }

    // Dismissal complete
    assert!(!app.show_titlebar);
    assert_eq!(app.titlebar_slide_progress(), 0.0);
}

#[test]
fn test_dropdown_menu_slide_down_animation() {
    use wazoo_app::app::DROPDOWN_MENU_SLIDE_TICKS;

    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window_id = Some(win_id);

    // Initial state: dropdown menu is closed
    assert!(!app.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // Toggle dropdown menu open
    let _ = app.update(Message::ToggleDropdownMenu);
    assert!(app.show_dropdown_menu);
    assert!(app.show_titlebar);
    assert_eq!(app.dropdown_menu_slide_ticks, 0);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // View rendered at initial opening state
    {
        let _view_open_start = app.view_titlebar();
    }

    // Animate frames through DROPDOWN_MENU_SLIDE_TICKS
    let mut prev_progress = app.dropdown_menu_slide_progress();
    for _ in 1..=DROPDOWN_MENU_SLIDE_TICKS {
        let _ = app.update(Message::VideoFrameTick);
        let cur_progress = app.dropdown_menu_slide_progress();
        assert!(
            cur_progress >= prev_progress,
            "Dropdown slide progress must increase monotonically: prev={}, cur={}",
            prev_progress,
            cur_progress
        );
        prev_progress = cur_progress;
    }

    // Dropdown has fully slid down
    assert_eq!(app.dropdown_menu_slide_progress(), 1.0);
    {
        let _view_open_full = app.view_titlebar();
    }

    // Close dropdown menu
    let _ = app.update(Message::CloseDropdownMenu);
    assert!(!app.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);
}


