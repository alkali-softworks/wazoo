use super::common::TempTestDir;
use wazoo_app::app::{WazooApp, new_test_app};
use wazoo_app::message::Message;
use wazoo_core::{ConfigManager, Database};

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
fn test_boot_persists_default_keybinds() {
    let temp_dir = TempTestDir::new("wazoo_boot_kb");
    let config_mgr = ConfigManager::with_dirs(
        temp_dir.path().to_path_buf(),
        temp_dir.path().to_path_buf(),
    );
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
}

#[test]
fn test_boot_reconciles_and_persists_incomplete_keybinds() {
    let temp_dir = TempTestDir::new("wazoo_incomplete_kb");
    let config_mgr = ConfigManager::with_dirs(
        temp_dir.path().to_path_buf(),
        temp_dir.path().to_path_buf(),
    );
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
    let db = Database::open_in_memory().unwrap();
    let (booted_app, _) = WazooApp::new_with_backend(None, app.config_mgr, db);
    assert_eq!(
        booted_app.settings.playback_mode,
        wazoo_core::PlaybackMode::Normal
    );
}

#[test]
fn test_boot_restored_session_shows_loading_static_immediately() {
    let temp_dir = TempTestDir::new("wazoo_boot_static");
    let config_mgr = ConfigManager::with_dirs(
        temp_dir.path().to_path_buf(),
        temp_dir.path().to_path_buf(),
    );
    let db = Database::open_in_memory().expect("in-memory db");

    let stream_url = "http://127.0.0.1:9/boot_stream.mp4".to_string();
    db.insert_or_update_video(&wazoo_core::VideoRecord::new(1, "Boot Stream", &stream_url))
        .unwrap();

    let mut settings = config_mgr.load_settings();
    settings.loading_indicator = wazoo_core::LoadingIndicator::TvStatic;
    settings.player_count = 1;
    settings.session_videos = vec![wazoo_core::VideoSession {
        path: stream_url.clone(),
        position_secs: 42.0,
        is_muted: true,
        volume: 0.6,
        is_shuffle: false,
    }];
    config_mgr.save_settings(&settings).unwrap();

    let (mut app, _) = WazooApp::new_with_backend(None, config_mgr, db);
    assert_eq!(app.players.len(), 1);
    assert!(
        app.players[0].is_loading,
        "Booted session player must immediately enter loading state instead of showing black"
    );
    assert!(app.players[0].is_placeholder());
    assert_eq!(app.players[0].state.path, stream_url);
    assert!(app.players[0].state.is_muted);
    assert!((app.players[0].state.volume - 0.6).abs() < 0.001);

    let static_guard = app.players[0].static_frame.lock().unwrap();
    assert!(
        static_guard.width > 0 && static_guard.height > 0 && !static_guard.pixels.is_empty(),
        "First-frame static buffer must be populated immediately on boot"
    );
    let seq0 = static_guard.frame_seq;
    drop(static_guard);

    for _ in 0..4 {
        let _ = app.update(Message::Tick);
    }
    let seq1 = app.players[0].static_frame.lock().unwrap().frame_seq;
    assert!(seq1 > seq0, "Boot static must animate across ticks");
}
