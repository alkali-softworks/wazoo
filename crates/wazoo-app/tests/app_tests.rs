use iced::Point;
use wazoo_app::app::{SettingsTab, WazooApp, new_test_app};
use wazoo_app::message::Message;
use wazoo_core::{ConfigManager, Database, VideoRecord};

#[test]
fn test_test_app_isolation() {
    let (mut app, _) = new_test_app();
    // Ensure test config path does not point to live user directory
    let path = app.config_mgr.config_file_path();
    assert!(!path.to_string_lossy().contains(".config/wazoo-rs"));
    assert!(!path.to_string_lossy().contains(".config/wazoo"));
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
    app.default_shuffle_mode = false;
    let next1 = app.get_next_video_rec(Some("/media/v1.mp4")).unwrap();
    assert_eq!(next1.path, "/media/v2.mp4");
    let next2 = app.get_next_video_rec(Some("/media/v2.mp4")).unwrap();
    assert_eq!(next2.path, "/media/v3.mp4");
    let next3 = app.get_next_video_rec(Some("/media/v3.mp4")).unwrap();
    assert_eq!(next3.path, "/media/v1.mp4"); // Wraps around

    // 2. Random/shuffle mode
    app.default_shuffle_mode = true;
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
fn test_navigation_secondary_filter_from_file_picker_search() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Cowboy Bebop - 01", "/anime/Cowboy Bebop/01.mkv"),
        VideoRecord::new(2, "Cowboy Bebop - 02", "/anime/Cowboy Bebop/02.mkv"),
        VideoRecord::new(3, "Space Dandy - 01", "/anime/Space Dandy/01.mkv"),
        VideoRecord::new(4, "Space Dandy - 02", "/anime/Space Dandy/02.mkv"),
        VideoRecord::new(5, "Trigun - 01", "/anime/Trigun/01.mkv"),
    ];

    // 1. When file picker search is empty or whitespace, sequential navigation walks all available videos
    app.drawers.file_picker_search = "".to_string();
    let next = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next.path, "/anime/Cowboy Bebop/02.mkv");

    app.drawers.file_picker_search = "   ".to_string();
    let next_space_blank = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next_space_blank.path, "/anime/Cowboy Bebop/02.mkv");

    // 2. When file picker search has a secondary filter "space", candidates are filtered
    app.drawers.file_picker_search = "space".to_string();
    // Starting from Bebop (which is outside the filter), next should enter Space Dandy
    let next1 = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(next1.path, "/anime/Space Dandy/01.mkv");

    // Advancing from Space Dandy 01 gives Space Dandy 02
    let next2 = app
        .get_next_video_rec_for_navigation(Some("/anime/Space Dandy/01.mkv"), false)
        .unwrap();
    assert_eq!(next2.path, "/anime/Space Dandy/02.mkv");

    // Advancing from Space Dandy 02 wraps around to Space Dandy 01 within the filtered pool
    let next3 = app
        .get_next_video_rec_for_navigation(Some("/anime/Space Dandy/02.mkv"), false)
        .unwrap();
    assert_eq!(next3.path, "/anime/Space Dandy/01.mkv");

    // Going backwards from Space Dandy 01 wraps to Space Dandy 02 within filtered pool
    let prev1 = app
        .get_prev_video_rec_for_navigation(Some("/anime/Space Dandy/01.mkv"), false)
        .unwrap();
    assert_eq!(prev1.path, "/anime/Space Dandy/02.mkv");

    // 3. Fallback when filter matches no videos: falls back to available_videos
    app.drawers.file_picker_search = "nonexistent_query_xyz".to_string();
    let fallback = app
        .get_next_video_rec_for_navigation(Some("/anime/Cowboy Bebop/01.mkv"), false)
        .unwrap();
    assert_eq!(fallback.path, "/anime/Cowboy Bebop/02.mkv");
}

#[test]
fn test_typing_in_file_picker_search_does_not_swap_playing_video() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!("test_typing_filter_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("Cowboy Bebop - 01.mp4");
    let f2 = temp_dir.join("Space Dandy - 01.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    let path1 = f1.to_string_lossy().to_string();
    let path2 = f2.to_string_lossy().to_string();

    app.available_videos = vec![
        VideoRecord::new(1, "Cowboy Bebop - 01", &path1),
        VideoRecord::new(2, "Space Dandy - 01", &path2),
    ];
    app.default_shuffle_mode = false;

    // Initialize 1 player; it plays first video (Cowboy Bebop)
    let _ = app.update(Message::SetPlayerCount(1));
    assert_eq!(app.players.len(), 1);
    let p0_id = app.players[0].id;
    app.players[0].shuffle = false;
    assert_eq!(app.players[0].state.path, path1);

    // User types in file picker drawer search box
    let _ = app.update(Message::FilePickerSearchChanged("space".to_string()));
    assert_eq!(app.drawers.file_picker_search, "space");
    // Playing video MUST NOT be swapped while typing!
    assert_eq!(app.players[0].state.path, path1);

    // Debounce timer arrives to apply drawer search UI filtering
    let _ = app.update(Message::ApplyFilePickerSearch);
    // Playing video STILL MUST NOT be swapped!
    assert_eq!(app.players[0].state.path, path1);

    // Only when user advances to next video does the secondary filter take effect
    let _ = app.update(Message::NextVideo(p0_id));
    assert_eq!(app.players[0].state.path, path2);

    let _ = std::fs::remove_dir_all(&temp_dir);
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
    assert_eq!(app.search.active_folder, "Todo");
    assert_eq!(app.search.selected_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.search.active_folder));

    // 2. File picker in Spanish does not show badge for Todo
    {
        let _view_all = app.view_file_picker();
    }

    // 3. Search confined to anime
    let _ = app.update(Message::SelectSearchFolder("/media/anime".to_string()));
    assert_eq!(app.search.active_folder, "Todo"); // Still Todo before search
    let _ = app.update(Message::PerformSearch);
    assert_eq!(app.search.active_folder, "/media/anime");
    assert_eq!(app.settings.last_folders, vec!["/media/anime".to_string()]);
    assert!(!app.is_all_folder(&app.search.active_folder));

    // 4. Reset search folder returns to Spanish All ("Todo")
    let _ = app.update(Message::ResetSearchFolder);
    assert_eq!(app.search.active_folder, "Todo");
    assert_eq!(app.search.selected_folder, "Todo");
    assert!(app.settings.last_folders.is_empty());
    assert!(app.is_all_folder(&app.search.active_folder));
}

#[test]
fn test_file_picker_multiple_confined_folder_chips_wrap() {
    let (mut app, _) = new_test_app();
    app.settings.media_folders = vec![
        "/media/anime".to_string(),
        "/media/tv".to_string(),
        "/media/Sat_Morning_Shows".to_string(),
    ];
    app.available_videos = vec![
        VideoRecord::new(1, "Anime 1", "/media/anime/a1.mp4"),
        VideoRecord::new(2, "TV 1", "/media/tv/t1.mp4"),
        VideoRecord::new(3, "Sat Show 1", "/media/Sat_Morning_Shows/s1.mp4"),
    ];

    // Select multiple folders
    app.search.active_folders = vec![
        "/media/anime".to_string(),
        "/media/tv".to_string(),
        "/media/Sat_Morning_Shows".to_string(),
    ];

    // Open file picker drawer
    let _ = app.update(Message::ToggleFilePicker);
    assert!(app.drawers.show_file_picker);

    // Verify view renders with wrapped chips
    {
        let _view = app.view_file_picker();
    }

    // Remove one chip
    let _ = app.update(Message::RemoveActiveSearchFolder("/media/tv".to_string()));
    assert_eq!(app.search.active_folders.len(), 2);
    assert!(!app.search.active_folders.contains(&"/media/tv".to_string()));

    {
        let _view_after_remove = app.view_file_picker();
    }
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
    assert_eq!(app.settings.keybinds.open_help, "F1");
    assert_eq!(app.settings.keybinds.open_settings, "F2");
    assert_eq!(app.settings.keybinds.toggle_filters, "7");
    assert_eq!(app.settings.keybinds.fit_window, "w");

    // Every key in ALL_KEYS must be present in settings.json
    for key in wazoo_core::KeybindSettings::ALL_KEYS {
        assert!(
            content.contains(&format!("\"{}\"", key)),
            "settings.json must contain key {}",
            key
        );
    }

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
    assert_eq!(app.settings.keybinds.open_help, "F1");
    assert_eq!(app.settings.keybinds.open_settings, "F2");
    assert_eq!(app.settings.keybinds.toggle_filters, "7");
    assert_eq!(app.settings.keybinds.fit_window, "w");

    // Boot should have written the complete list to settings.json
    assert!(app.config_mgr.has_complete_keybinds_in_settings());
    let updated_file_content = std::fs::read_to_string(app.config_mgr.config_file_path()).unwrap();
    assert!(updated_file_content.contains("\"toggle_layout\": \"o\""));
    assert!(updated_file_content.contains("\"close_app\": \"Alt+X\""));
    assert!(updated_file_content.contains("\"add_player\": \"n\""));
    assert!(updated_file_content.contains("\"open_help\": \"F1\""));
    assert!(updated_file_content.contains("\"open_settings\": \"F2\""));
    assert!(updated_file_content.contains("\"toggle_filters\": \"7\""));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

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
fn test_view_titlebar() {
    let (app, _) = new_test_app();
    let _elem = app.view_titlebar();
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
    let _ = app
        .db
        .insert_misc_video("existing_video", &f1.to_string_lossy());
    assert_eq!(app.db.get_video_count().unwrap(), 1);

    // Perform a search for a term that does not match
    app.search.input = "nonexistent_query_xyz".to_string();
    let _ = app.update(Message::PerformSearch);

    assert_eq!(app.search.active_query, "nonexistent_query_xyz");
    assert!(app.available_videos.is_empty());

    // Calling view() exercises the view_no_matches code path
    {
        let _main_view = app.view();
    }

    // Now clear search
    let _ = app.update(Message::ClearSearch);
    assert!(app.search.active_query.is_empty());
    assert_eq!(app.available_videos.len(), 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_settings_toggle_default_player() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.is_default_player);

    let _ = app.update(Message::OpenSettingsModal);
    assert!(app.modals.settings);

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
fn test_settings_modal_tabs_and_playback_options() {
    let (mut app, _) = new_test_app();
    assert_eq!(app.modals.settings_tab, SettingsTab::General);

    // Open settings modal
    let _ = app.update(Message::OpenSettingsModal);
    assert!(app.modals.settings);
    assert_eq!(app.modals.settings_tab, SettingsTab::General);

    // Render General tab
    {
        let _view_general = app.view_settings_modal();
    }

    // Switch to Playback tab
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Playback));
    assert_eq!(app.modals.settings_tab, SettingsTab::Playback);

    // Render Playback tab
    {
        let _view_playback = app.view_settings_modal();
    }

    // Adjust playback options
    let _ = app.update(Message::SetGamma(25.0));
    assert_eq!(app.settings.gamma, 25.0);

    let _ = app.update(Message::SetContrast(-15.0));
    assert_eq!(app.settings.contrast, -15.0);

    let _ = app.update(Message::SetBrightness(10.0));
    assert_eq!(app.settings.brightness, 10.0);

    let _ = app.update(Message::SetSaturation(40.0));
    assert_eq!(app.settings.saturation, 40.0);

    let _ = app.update(Message::SetPlaybackSpeed(1.5));
    assert_eq!(app.settings.playback_speed, 1.5);

    let _ = app.update(Message::SetBufferDuration(15));
    assert_eq!(app.settings.buffer_duration_secs, 15);

    let _ = app.update(Message::SetBufferSize(64));
    assert_eq!(app.settings.buffer_size_mb, 64);

    // Re-render Playback tab with adjusted values
    {
        let _view_playback_adjusted = app.view_settings_modal();
    }

    // Test ResetPlaybackOptions
    let _ = app.update(Message::ResetPlaybackOptions);
    assert_eq!(app.settings.gamma, 0.0);
    assert_eq!(app.settings.contrast, 0.0);
    assert_eq!(app.settings.brightness, 0.0);
    assert_eq!(app.settings.saturation, 0.0);
    // Test individual slider reset (the reset button next to each value badge)
    let _ = app.update(Message::SetGamma(40.0));
    let _ = app.update(Message::SetContrast(100.0));
    assert_eq!(app.settings.gamma, 40.0);
    assert_eq!(app.settings.contrast, 100.0);
    {
        let _view_with_reset_buttons = app.view_settings_modal();
    }
    // Reset only gamma via individual reset button
    let _ = app.update(Message::SetGamma(0.0));
    assert_eq!(app.settings.gamma, 0.0);
    assert_eq!(app.settings.contrast, 100.0);
    // Reset only contrast via individual reset button
    let _ = app.update(Message::SetContrast(0.0));
    assert_eq!(app.settings.contrast, 0.0);

    // Switch to Filters tab (Shader Filters)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Filters));
    assert_eq!(app.modals.settings_tab, SettingsTab::Filters);
    {
        let _view_filters = app.view_settings_modal();
    }
    // Master filter toggle enables all checked filters (CRT is checked by default)
    let _ = app.update(Message::ToggleFilters);
    assert!(app.settings.filters_enabled);
    assert!(app.settings.crt_enabled);
    {
        let _view_filters_active = app.view_settings_modal();
    }
    // Individual filter toggles work
    let _ = app.update(Message::ToggleWavyFilter);
    assert!(app.settings.filter_wavy);
    let _ = app.update(Message::ToggleFogFilter);
    assert!(app.settings.filter_fog);

    let _ = app.update(Message::ToggleFilters);
    assert!(!app.settings.filters_enabled);
    assert!(!app.settings.crt_enabled);

    // Switch to System tab (Window Opacity & Default Player)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::System));
    assert_eq!(app.modals.settings_tab, SettingsTab::System);
    {
        let _view_system = app.view_settings_modal();
    }

    // Adjust window opacity on System tab
    let _ = app.update(Message::SetWindowOpacity(0.75));
    assert_eq!(app.settings.window_opacity, 0.75);

    // Toggle default player on System tab
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(app.settings.is_default_player);
    {
        let _view_system_active = app.view_settings_modal();
    }
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(!app.settings.is_default_player);

    // Toggle standby player on System tab
    assert!(!app.settings.use_standby_player);
    let _ = app.update(Message::ToggleStandbyPlayer);
    assert!(app.settings.use_standby_player);
    {
        let _view_system_standby_on = app.view_settings_modal();
    }
    let _ = app.update(Message::ToggleStandbyPlayer);
    assert!(!app.settings.use_standby_player);

    // Adjust flip interval on System tab
    assert_eq!(app.settings.flip_interval_secs, 45);
    let _ = app.update(Message::SetFlipInterval(90));
    assert_eq!(app.settings.flip_interval_secs, 90);
    {
        let _view_system_flip_active = app.view_settings_modal();
    }
    // Reset flip interval via reset button
    let _ = app.update(Message::SetFlipInterval(
        wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS,
    ));
    assert_eq!(app.settings.flip_interval_secs, 45);

    // Test flip interval clamping
    let _ = app.update(Message::SetFlipInterval(0));
    assert_eq!(app.settings.flip_interval_secs, 1);
    let _ = app.update(Message::SetFlipInterval(99999));
    assert_eq!(app.settings.flip_interval_secs, 3600);
    let _ = app.update(Message::SetFlipInterval(45));

    // Switch to Cube tab (3D Cube Screensaver settings)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Cube));
    assert_eq!(app.modals.settings_tab, SettingsTab::Cube);
    {
        let _view_cube = app.view_settings_modal();
    }

    // Toggle cube screensaver, adjust speed & size, spawn additional cube
    let _ = app.update(Message::ToggleCubeScreensaver);
    assert!(app.cube.enabled);
    assert_eq!(app.cube.cubes.len(), 1);
    {
        let _view_cube_active = app.view_settings_modal();
    }
    let _ = app.update(Message::SetCubeSpeed(1.5));
    assert_eq!(app.cube.speed_multiplier, 1.5);
    let _ = app.update(Message::SetCubeSize(1.8));
    assert_eq!(app.cube.size_multiplier, 1.8);
    let _ = app.update(Message::SpawnCube);
    assert_eq!(app.cube.cubes.len(), 2);
    let _ = app.update(Message::ClearCubes);
    assert_eq!(app.cube.cubes.len(), 0);

    // Switch back to General tab
    let _ = app.update(Message::SetSettingsTab(SettingsTab::General));
    assert_eq!(app.modals.settings_tab, SettingsTab::General);
    {
        let _view_general_back = app.view_settings_modal();
    }

    // Close settings modal
    let _ = app.update(Message::CloseSettingsModal);
    assert!(!app.modals.settings);
}

#[test]
fn test_menu_modal_open_close_and_render() {
    let (mut app, _) = new_test_app();
    assert!(!app.modals.menu);

    let _ = app.update(Message::OpenMenuModal);
    assert!(app.modals.menu);
    {
        let _menu_view = app.view_menu_modal();
    }

    let _ = app.update(Message::CloseMenuModal);
    assert!(!app.modals.menu);
}

#[test]
fn test_search_and_bookmarks_modal_render() {
    let (mut app, _) = new_test_app();
    let _ = app.update(Message::OpenSearchModal);
    assert!(app.modals.search);
    {
        let _search_view = app.view_search_modal();
    }
    let _ = app.update(Message::CloseSearchModal);
    assert!(!app.modals.search);

    let _ = app.update(Message::ToggleBookmarksModal);
    assert!(app.modals.bookmarks);
    {
        let _bookmarks_view = app.view_bookmarks_modal();
    }
    let _ = app.update(Message::CloseBookmarksModal);
    assert!(!app.modals.bookmarks);
}

#[test]
fn test_top_menu_slide_down_animation() {
    use wazoo_app::app::{TITLEBAR_FADE_TICKS, TITLEBAR_SHOW_DELAY_TICKS, TITLEBAR_SLIDE_TICKS};

    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: top menu is hidden
    assert!(!app.titlebar.show);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Move cursor into top menu trigger zone (y < 35.0)
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(300.0, 10.0)));

    // Tick through hover delay (8 ticks)
    for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
        let _ = app.update(Message::Tick);
    }

    // After hover delay passes, titlebar begins showing and starts sliding down from 0.0
    assert!(app.titlebar.show);
    assert_eq!(app.titlebar.slide_ticks, 0);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Initial view rendering at start of slide down
    {
        let _view_start = app.view_titlebar();
    }

    // Animate ticks and verify slide down progress increases monotonically to 1.0
    let mut prev_progress = app.titlebar_slide_progress();
    for _ in 1..=TITLEBAR_SLIDE_TICKS {
        let _ = app.update(Message::Tick);
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
        let _ = app.update(Message::Tick);
    }

    // Dismissal complete
    assert!(!app.titlebar.show);
    assert_eq!(app.titlebar_slide_progress(), 0.0);
}

#[test]
fn test_dropdown_menu_slide_down_animation() {
    use wazoo_app::app::DROPDOWN_MENU_SLIDE_TICKS;

    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: dropdown menu is closed
    assert!(!app.titlebar.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // Toggle dropdown menu open
    let _ = app.update(Message::ToggleDropdownMenu);
    assert!(app.titlebar.show_dropdown_menu);
    assert!(app.titlebar.show);
    assert_eq!(app.titlebar.dropdown_menu_slide_ticks, 0);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // View rendered at initial opening state
    {
        let _view_open_start = app.view_titlebar();
    }

    // Animate frames through DROPDOWN_MENU_SLIDE_TICKS
    let mut prev_progress = app.dropdown_menu_slide_progress();
    for _ in 1..=DROPDOWN_MENU_SLIDE_TICKS {
        let _ = app.update(Message::Tick);
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
    assert!(!app.titlebar.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);
}

#[test]
fn test_player_osd_fade_in_and_fade_out_animation() {
    use wazoo_app::app::PLAYER_OVERLAY_FADE_TICKS;

    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: player OSD is completely hidden
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);

    // Hover over a player to trigger OSD entrance
    let _ = app.update(Message::PlayerHovered(1));
    assert_eq!(app.hovered_player_id, Some(1));

    // On initial trigger, fade-in starts at alpha 0.0 (smooth entrance instead of popping to 1.0)
    assert_eq!(app.overlay.fade_in_ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);

    // Over PLAYER_OVERLAY_FADE_TICKS (12 ticks), alpha monotonically fades in to 1.0
    let mut prev_alpha = app.player_overlay_alpha();
    for _ in 1..=PLAYER_OVERLAY_FADE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_alpha = app.player_overlay_alpha();
        assert!(
            cur_alpha >= prev_alpha,
            "Fade-in alpha must increase monotonically: prev={}, cur={}",
            prev_alpha,
            cur_alpha
        );
        prev_alpha = cur_alpha;
    }

    // Fully faded in
    assert_eq!(app.player_overlay_alpha(), 1.0);

    // Now unhover the player to trigger fade out
    let _ = app.update(Message::PlayerUnhovered(1));

    // Over PLAYER_OVERLAY_FADE_TICKS frames, alpha monotonically fades out from 1.0 to 0.0
    let mut prev_alpha = app.player_overlay_alpha();
    for _ in 1..=PLAYER_OVERLAY_FADE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_alpha = app.player_overlay_alpha();
        assert!(
            cur_alpha <= prev_alpha,
            "Fade-out alpha must decrease monotonically: prev={}, cur={}",
            prev_alpha,
            cur_alpha
        );
        prev_alpha = cur_alpha;
    }

    // Overlay completely dismissed
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);
    assert_eq!(app.hovered_player_id, None);

    // Test interrupted entrance: unhovering while still fading in does not jump
    let _ = app.update(Message::PlayerHovered(1));
    for _ in 0..4 {
        let _ = app.update(Message::Tick);
    }
    let mid_alpha = app.player_overlay_alpha();
    assert!((mid_alpha - (4.0 / PLAYER_OVERLAY_FADE_TICKS as f32)).abs() < 1e-4);

    // Unhover while mid-entrance: fade-out smoothly reverses from current alpha (4/12)
    let _ = app.update(Message::PlayerUnhovered(1));
    assert!(app.overlay.ticks <= 4);

    for _ in 0..4 {
        let _ = app.update(Message::Tick);
    }
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);
}

#[test]
fn test_prev_next_frame_keybind_dispatch() {
    let (mut app, _) = new_test_app();

    // Next frame ('.')
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character(".".into()),
        iced::event::Status::Ignored,
    ));
    assert!(app.overlay.ticks > 0);

    // Reset ticks to verify comma independently
    app.overlay.ticks = 0;

    // Prev frame (',')
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character(",".into()),
        iced::event::Status::Ignored,
    ));
    assert!(app.overlay.ticks > 0);
}

#[test]
fn test_pin_keybind_dispatch() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.is_always_on_top);

    // Press 'p'
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
    ));
    assert!(app.settings.is_always_on_top);

    // Press 'p' again to toggle off
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character("p".into()),
        iced::event::Status::Ignored,
    ));
    assert!(!app.settings.is_always_on_top);
}

#[test]
fn test_player_by_id_helpers() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!("test_player_helpers_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("v1.mp4");
    let _ = std::fs::File::create(&f1);

    app.available_videos = vec![wazoo_core::VideoRecord::new(1, "V1", f1.to_string_lossy())];
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

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_mute_default_when_no_preference() {
    let (mut app, _) = new_test_app();
    assert_eq!(app.settings.playback_mode, wazoo_core::PlaybackMode::Normal);

    let temp_dir = std::env::temp_dir().join(format!("test_mute_pref_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("V1.mp4");
    let f2 = temp_dir.join("V2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        wazoo_core::VideoRecord::new(1, "V1", f1.to_string_lossy()),
        wazoo_core::VideoRecord::new(2, "V2", f2.to_string_lossy()),
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

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_mute_preference_preserved_when_user_explicitly_mutes() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!("test_mute_explicit_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("V1.mp4");
    let f2 = temp_dir.join("V2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        wazoo_core::VideoRecord::new(1, "V1", f1.to_string_lossy()),
        wazoo_core::VideoRecord::new(2, "V2", f2.to_string_lossy()),
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

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_scroll_mode_m_key_toggles_global_mute_and_spawns_with_preference() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!("test_scroll_mute_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("S1.mp4");
    let f2 = temp_dir.join("S2.mp4");
    let _ = std::fs::File::create(&f1);
    let _ = std::fs::File::create(&f2);

    app.available_videos = vec![
        wazoo_core::VideoRecord::new(1, "S1", f1.to_string_lossy()),
        wazoo_core::VideoRecord::new(2, "S2", f2.to_string_lossy()),
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

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_boot_playback_mode_is_always_normal_even_if_scroll_was_used() {
    let (mut app, _) = new_test_app();
    app.settings.playback_mode = wazoo_core::PlaybackMode::Scroll;
    app.config_mgr.save_settings(&app.settings).unwrap();

    // Verify settings file does not save "scroll"
    let content = std::fs::read_to_string(app.config_mgr.config_file_path()).unwrap();
    assert!(!content.contains(r#""playback_mode": "scroll""#));
    assert!(content.contains(r#""playback_mode": "normal""#));

    // Verify load_settings always returns Normal
    let loaded = app.config_mgr.load_settings();
    assert_eq!(loaded.playback_mode, wazoo_core::PlaybackMode::Normal);

    // Verify booted app with backend always starts in Normal mode
    let db = wazoo_core::Database::open_in_memory().unwrap();
    let (booted_app, _) = WazooApp::new_with_backend(None, app.config_mgr, db);
    assert_eq!(
        booted_app.settings.playback_mode,
        wazoo_core::PlaybackMode::Normal
    );
}

#[test]
fn test_set_active_query_updates_search_tags_and_reconciles() {
    let (mut app, _) = new_test_app();
    app.db
        .insert_or_update_video(&wazoo_core::VideoRecord::new(
            1,
            "ShowA - S01E01",
            "/media/Shows/ShowA - S01E01.mp4",
        ))
        .unwrap();
    app.db
        .insert_or_update_video(&wazoo_core::VideoRecord::new(
            2,
            "ShowA - S01E02",
            "/media/Shows/ShowA - S01E02.mp4",
        ))
        .unwrap();
    app.db
        .insert_or_update_video(&wazoo_core::VideoRecord::new(
            3,
            "ShowB - S01E01",
            "/media/Shows/ShowB - S01E01.mp4",
        ))
        .unwrap();

    let _ = app.update(Message::SetActiveQuery("ShowA".to_string()));
    assert_eq!(app.search.active_query, "ShowA");
    assert_eq!(app.search.tags, vec!["ShowA"]);
    assert_eq!(app.available_videos.len(), 2);
    assert!(
        app.available_videos
            .iter()
            .all(|v| v.path.contains("ShowA"))
    );
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
    let _ = app.update(Message::WindowResized(win_id, iced::Size::new(960.0, 540.0)));
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
    let _ = app.update(Message::WindowResized(win_id, iced::Size::new(1280.0, 720.0)));
    assert!(app.window.is_resizing());

    // Cursor movement during resize is ignored to prevent titlebar/hover thrashing
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(10.0, 10.0)));
    assert_eq!(app.titlebar.hover_ticks, 0);

    // Player hover messages during resize are suppressed
    let _ = app.update(Message::PlayerHovered(1));
    assert_eq!(app.hovered_player_id, None);

    // Player unhover messages during resize are suppressed
    let _ = app.update(Message::PlayerUnhovered(1));
    assert_eq!(app.hovered_player_id, None);

    // WindowMoved also keeps resize debounce alive
    let _ = app.update(Message::WindowMoved(win_id, iced::Point::new(100.0, 100.0)));
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
fn test_file_picker_search_clear_button() {
    let (mut app, _) = new_test_app();
    app.available_videos = vec![
        VideoRecord::new(1, "Space Odyssey", "/media/movies/space_odyssey.mp4"),
        VideoRecord::new(2, "Anime 1", "/media/anime/a1.mp4"),
    ];

    // Open file picker drawer
    let _ = app.update(Message::ToggleFilePicker);
    assert!(app.drawers.show_file_picker);

    // Initial empty search: view file picker renders without clear button
    assert!(app.drawers.file_picker_search.is_empty());
    {
        let _view_empty = app.view_file_picker();
    }

    // User types in search: search input is not empty
    let _ = app.update(Message::FilePickerSearchChanged("space".to_string()));
    assert_eq!(app.drawers.file_picker_search, "space");
    assert!(app.drawers.file_picker_debounce_ticks > 0);

    // View renders with clear button layered on search bar
    {
        let _view_with_query = app.view_file_picker();
    }

    // Navigating with "space" active yields Space Odyssey even starting from Anime 1
    let next_filtered = app
        .get_next_video_rec_for_navigation(Some("/media/anime/a1.mp4"), false)
        .unwrap();
    assert_eq!(next_filtered.name, "Space Odyssey");

    // Pressing the clear button:
    let _ = app.update(Message::ClearFilePickerSearch);
    assert_eq!(app.drawers.file_picker_search, "");
    assert_eq!(app.drawers.file_picker_debounce_ticks, 0);

    // Filter cleared: sequential navigation from Space Odyssey advances to Anime 1
    let next_all = app
        .get_next_video_rec_for_navigation(Some("/media/movies/space_odyssey.mp4"), false)
        .unwrap();
    assert_eq!(next_all.name, "Anime 1");

    // View renders cleanly with empty search input (no clear button)
    {
        let _view_cleared = app.view_file_picker();
    }
}

#[test]
fn test_search_modal_long_tags_list_wrapping_and_scrolling() {
    let (mut app, _) = new_test_app();

    // 1. Open search modal with no tags
    let _ = app.update(Message::OpenSearchModal);
    assert!(app.modals.search);
    assert!(app.search.tags.is_empty());
    {
        let _view_empty = app.view_search_modal();
    }

    // 2. Add multiple tags matching the user's scenario
    let _ = app.update(Message::SearchInputChanged(
        "not dragon ball, twilight, boku, hotel, otome wa boku desu,".to_string(),
    ));
    assert_eq!(app.search.tags.len(), 5);
    assert_eq!(app.search.tags[0], "not dragon ball");
    assert_eq!(app.search.tags[4], "otome wa boku desu");
    {
        let _view_wrapped = app.view_search_modal();
    }

    // 3. Add more tags to trigger the scrollable view (> 3 rows)
    let _ = app.update(Message::SearchInputChanged(
        "tag6, tag7, tag8, tag9, tag10, tag11, tag12, tag13,".to_string(),
    ));
    assert_eq!(app.search.tags.len(), 13);
    {
        let _view_scrollable = app.view_search_modal();
    }

    // 4. Remove a tag and verify view renders cleanly
    let _ = app.update(Message::RemoveSearchTag(0));
    assert_eq!(app.search.tags.len(), 12);
    {
        let _view_after_remove = app.view_search_modal();
    }
}

#[test]
fn test_playback_settings_sliders_debounced() {
    let (mut app, _) = new_test_app();

    // Open settings modal
    let _ = app.update(Message::OpenSettingsModal);
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Playback));
    assert!(app.modals.settings);
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);

    // 1. Moving sliders updates app.settings immediately for responsive UI
    let _ = app.update(Message::SetGamma(35.0));
    assert_eq!(app.settings.gamma, 35.0);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        wazoo_app::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS
    );

    let _ = app.update(Message::SetSaturation(-20.0));
    assert_eq!(app.settings.saturation, -20.0);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        wazoo_app::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS
    );

    // 2. Debounce ticks count down with Tick
    let _ = app.update(Message::Tick);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        wazoo_app::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS - 1
    );

    // 3. Advancing all remaining ticks flushes the debounce
    while app.modals.playback_settings_debounce_ticks > 0 {
        let _ = app.update(Message::Tick);
    }
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);

    // 4. Closing the settings modal immediately flushes any pending debounced settings
    let _ = app.update(Message::SetBrightness(15.0));
    assert!(app.modals.playback_settings_debounce_ticks > 0);
    let _ = app.update(Message::CloseSettingsModal);
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);
    assert_eq!(app.settings.brightness, 15.0);
}

#[test]
fn test_seek_relative_debounced_and_accumulated() {
    let (mut app, _) = new_test_app();
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_test_seek_debounce_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let f1 = temp_dir.join("v1.mp4");
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
    assert!(app.overlay.toast_message.as_ref().unwrap().contains("Fullscreen Enabled"));
    assert!(app.overlay.toast_message.as_ref().unwrap().contains("[F11]"));

    // 2. Escape exits fullscreen when no modals/drawers are open
    let _ = app.update(Message::EscapePressed);
    assert!(!app.window.is_fullscreen);
    assert!(app.overlay.toast_message.as_ref().unwrap().contains("Fullscreen Disabled"));

    // 3. Rebind toggle_fullscreen and verify toast reflects the new key
    app.settings.keybinds.toggle_fullscreen = "f12".to_string();
    let _ = app.update(Message::ToggleFullscreen);
    assert!(app.window.is_fullscreen);
    assert!(app.overlay.toast_message.as_ref().unwrap().contains("[F12]"));
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
    assert!(app.window.is_fullscreen, "Closing drawer should not exit fullscreen");

    // 6. Subsequent Escape exits fullscreen
    let _ = app.update(Message::EscapePressed);
    assert!(!app.window.is_fullscreen);
}

#[test]
fn test_fit_window_keybind_and_resizing() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_fit_window_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_mgr = ConfigManager::with_dirs(temp_dir.clone(), temp_dir.clone());
    let db = Database::open_in_memory().expect("in-memory db");
    let (mut app, _) = WazooApp::new_with_backend(None, config_mgr, db);

    let f1 = temp_dir.join("test_video.mp4");
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
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character("w".into()),
        iced::event::Status::Ignored,
    ));

    // Window width shrinks to 1920 to eliminate horizontal black bars, height remains 1080
    assert_eq!(app.settings.window_bounds.width, 1920);
    assert_eq!(app.settings.window_bounds.height, 1080);

    // Case 2: Letterbox (window is too tall, e.g. 1920x1500)
    app.settings.window_bounds.width = 1920;
    app.settings.window_bounds.height = 1500;

    // Press 'W' key (uppercase)
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character("W".into()),
        iced::event::Status::Ignored,
    ));

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

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_fit_window_multi_player_layouts() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_fit_multi_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_mgr = ConfigManager::with_dirs(temp_dir.clone(), temp_dir.clone());
    let db = Database::open_in_memory().expect("in-memory db");
    let (mut app, _) = WazooApp::new_with_backend(None, config_mgr, db);

    let f1 = temp_dir.join("v1.mp4");
    let f2 = temp_dir.join("v2.mp4");
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

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_calculate_alt_resize_direction_all_zones() {
    let (mut app, _) = new_test_app();
    app.settings.window_bounds.width = 900;
    app.settings.window_bounds.height = 600;

    // Top row (row = 0)
    app.window.cursor_position = iced::Point::new(100.0, 100.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::NorthWest));

    app.window.cursor_position = iced::Point::new(450.0, 100.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::North));

    app.window.cursor_position = iced::Point::new(800.0, 100.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::NorthEast));

    // Middle row (row = 1) - Outer edges
    app.window.cursor_position = iced::Point::new(100.0, 300.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::West));

    app.window.cursor_position = iced::Point::new(800.0, 300.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::East));

    // Bottom row (row = 2)
    app.window.cursor_position = iced::Point::new(100.0, 500.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::SouthWest));

    app.window.cursor_position = iced::Point::new(450.0, 500.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::South));

    app.window.cursor_position = iced::Point::new(800.0, 500.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::SouthEast));

    // Center zone (row = 1, col = 1) delegates to nearest quadrant
    // Top-left of center: x < 450, y < 300 -> NorthWest
    app.window.cursor_position = iced::Point::new(350.0, 250.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::NorthWest));

    // Top-right of center: x >= 450, y < 300 -> NorthEast
    app.window.cursor_position = iced::Point::new(550.0, 250.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::NorthEast));

    // Bottom-left of center: x < 450, y >= 300 -> SouthWest
    app.window.cursor_position = iced::Point::new(350.0, 350.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::SouthWest));

    // Bottom-right of center: x >= 450, y >= 300 -> SouthEast
    app.window.cursor_position = iced::Point::new(550.0, 350.0);
    assert!(matches!(app.calculate_alt_resize_direction(), iced::window::Direction::SouthEast));
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
    app.window.cursor_position = iced::Point::new(800.0, 500.0); // SouthEast corner

    let _ = app.update(Message::RightClickPressed(win_id));
    assert!(!app.modals.menu, "Context menu modal should NOT be opened when Alt is pressed");
    assert!(app.window.is_resizing(), "Alt + Right-click should mark window as resizing");
    assert!(app.window.last_resize_time.is_some());

    // 3. RightClickReleased resets resize state immediately
    let _ = app.update(Message::RightClickReleased);
    assert!(!app.window.is_resizing(), "RightClickReleased should reset is_resizing immediately");
    assert!(app.window.last_resize_time.is_none());

    // 4. AltDragResize in fullscreen mode is suppressed
    app.window.is_fullscreen = true;
    let _ = app.update(Message::AltDragResize);
    assert!(!app.window.is_resizing(), "Resize should not trigger when in fullscreen mode");
}







