use std::fs;
use wazoo_core::models::WazooSettings;
use wazoo_core::ConfigManager;

#[test]
fn test_config_manager_paths() {
    let mgr = ConfigManager::new();
    assert!(
        mgr.config_file_path()
            .to_string_lossy()
            .contains("wazoo-rs")
    );
    assert!(mgr.database_path().to_string_lossy().contains("wazoo-rs"));
    assert_eq!(mgr.database_path().parent(), mgr.config_file_path().parent());
    assert_eq!(
        mgr.database_path(),
        mgr.config_dir.join("wazoo.db")
    );
    assert!(mgr.config_dir.exists());
    assert!(mgr.data_dir.exists());
}

#[test]
fn test_legacy_database_migration() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_mig_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let config_dir = temp_dir.join("config");
    let data_dir = temp_dir.join("data");
    let _ = fs::create_dir_all(&config_dir);
    let _ = fs::create_dir_all(&data_dir);

    // Write a dummy legacy db into data_dir
    let legacy_db = data_dir.join("wazoo.db");
    fs::write(&legacy_db, b"sqlite_legacy_test_data").unwrap();

    // Write a 0-byte placeholder into config_dir (simulating empty file)
    let config_db = config_dir.join("wazoo.db");
    fs::write(&config_db, b"").unwrap();

    let mgr = ConfigManager::with_dirs(config_dir.clone(), data_dir.clone());
    assert_eq!(mgr.database_path(), config_dir.join("wazoo.db"));
    assert_eq!(mgr.database_path().parent(), mgr.config_file_path().parent());
    assert_eq!(fs::read(&config_db).unwrap(), b"sqlite_legacy_test_data");

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_session_videos_serialization() {
    let mut settings = WazooSettings::default();
    settings.session_videos.push(wazoo_core::models::VideoSession {
        path: "/path/to/video1.mp4".to_string(),
        position_secs: 42.5,
        is_muted: false,
        volume: 0.8,
        is_shuffle: false,
    });

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.session_videos.len(), 1);
    assert_eq!(deserialized.session_videos[0].path, "/path/to/video1.mp4");
    assert_eq!(deserialized.session_videos[0].position_secs, 42.5);
    assert!(!deserialized.session_videos[0].is_muted);
    assert_eq!(deserialized.session_videos[0].volume, 0.8);
    assert!(!deserialized.session_videos[0].is_shuffle);

    // Verify legacy video session JSON without is_shuffle defaults to true
    let legacy_session_json = r#"{"path":"/path/legacy.mp4","position_secs":12.0}"#;
    let legacy_session: wazoo_core::models::VideoSession =
        serde_json::from_str(legacy_session_json).unwrap();
    assert!(legacy_session.is_shuffle);

    // Verify backwards compatibility when session_videos is omitted from JSON
    let json_legacy = r#"{"window_bounds":{"x":0,"y":0,"width":1280,"height":720},"window_opacity":1.0,"media_folders":[],"player_count":1,"layout":"grid","playback_mode":"normal","scroll_speed":1.0,"is_global_muted":true,"last_query":"","last_folder":"All"}"#;
    let legacy_settings: WazooSettings = serde_json::from_str(json_legacy).unwrap();
    assert!(legacy_settings.session_videos.is_empty());
    assert!(legacy_settings.bookmarks.is_empty());
}

#[test]
fn test_bookmarks_serialization() {
    let mut settings = WazooSettings::default();
    settings.bookmarks.push(wazoo_core::models::Bookmark {
        name: "Sci-Fi / Episode 01".to_string(),
        query: "scifi".to_string(),
        path: "/media/scifi/ep01.mp4".to_string(),
        position_secs: 125.4,
        is_shuffle: false,
    });

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.bookmarks.len(), 1);
    assert_eq!(deserialized.bookmarks[0].name, "Sci-Fi / Episode 01");
    assert_eq!(deserialized.bookmarks[0].query, "scifi");
    assert_eq!(deserialized.bookmarks[0].path, "/media/scifi/ep01.mp4");
    assert_eq!(deserialized.bookmarks[0].position_secs, 125.4);
    assert!(!deserialized.bookmarks[0].is_shuffle);

    // Verify legacy bookmark JSON without is_shuffle defaults to true
    let legacy_bookmark_json =
        r#"{"name":"Legacy","query":"test","path":"/path/test.mp4","position_secs":10.0}"#;
    let legacy_bookmark: wazoo_core::models::Bookmark =
        serde_json::from_str(legacy_bookmark_json).unwrap();
    assert!(legacy_bookmark.is_shuffle);
}

#[test]
fn test_buffer_settings_clamping() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_clamp_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);
    let config_file = temp_dir.join("settings.json");

    let hostile_json = r#"{"buffer_size_mb": 999999999, "buffer_duration_secs": 99999}"#;
    fs::write(&config_file, hostile_json).unwrap();

    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    let loaded = mgr.load_settings();
    assert_eq!(loaded.buffer_size_mb, 4096);
    assert_eq!(loaded.buffer_duration_secs, 300);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_atomic_save_settings() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_atomic_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);

    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    let settings = WazooSettings {
        last_query: "secure_query".to_string(),
        ..Default::default()
    };
    mgr.save_settings(&settings).unwrap();

    assert!(mgr.config_file_path().exists());
    assert!(!mgr.config_file_path().with_extension("tmp").exists());

    let loaded = mgr.load_settings();
    assert_eq!(loaded.last_query, "secure_query");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_preferred_audio_language_serialization() {
    let mut settings = WazooSettings::default();
    assert_eq!(settings.preferred_audio_language, None);

    settings.preferred_audio_language = Some("Japanese".to_string());
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.preferred_audio_language.as_deref(),
        Some("Japanese")
    );

    // Backwards compatibility when omitted
    let legacy_json = r#"{"window_opacity":1.0}"#;
    let legacy: WazooSettings = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(legacy.preferred_audio_language, None);
}

#[test]
fn test_keybinds_serialization_and_has_keybinds() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_keybinds_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);

    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    // Initially no settings file exists
    assert!(!mgr.has_keybinds_in_settings());

    // Save legacy settings without keybinds field
    let legacy_json = r#"{"window_opacity":0.95}"#;
    fs::write(mgr.config_file_path(), legacy_json).unwrap();
    assert!(!mgr.has_keybinds_in_settings());

    // Loading legacy settings fills in default keybinds
    let mut loaded = mgr.load_settings();
    assert_eq!(loaded.keybinds.add_player, "n");

    // Customizing and saving persists keybinds
    loaded.keybinds.add_player = "p".to_string();
    mgr.save_settings(&loaded).unwrap();
    assert!(mgr.has_keybinds_in_settings());

    // Reading back preserves customized keybind
    let reloaded = mgr.load_settings();
    assert_eq!(reloaded.keybinds.add_player, "p");

    // Saving an incomplete list of keybinds
    let incomplete_json = r#"{"keybinds":{"add_player":"p"}}"#;
    fs::write(mgr.config_file_path(), incomplete_json).unwrap();
    assert!(!mgr.has_complete_keybinds_in_settings());

    // Loading reconciles missing keys with defaults while preserving existing custom key
    let reconciled = mgr.load_settings();
    assert_eq!(reconciled.keybinds.add_player, "p");
    assert_eq!(reconciled.keybinds.close_app, "Alt+X");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_window_bounds_serialization_and_clamping() {
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_bounds_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);

    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    let custom_json = r#"{
        "window_bounds": {
            "x": 100,
            "y": 150,
            "width": 399,
            "height": 680
        }
    }"#;
    fs::write(mgr.config_file_path(), custom_json).unwrap();

    let loaded = mgr.load_settings();
    assert_eq!(loaded.window_bounds.x, 100);
    assert_eq!(loaded.window_bounds.y, 150);
    assert_eq!(loaded.window_bounds.width, 399);
    assert_eq!(loaded.window_bounds.height, 680);

    // Clamping invalid bounds
    let tiny_json = r#"{
        "window_bounds": {
            "x": -50,
            "y": 20,
            "width": 50,
            "height": 20
        }
    }"#;
    fs::write(mgr.config_file_path(), tiny_json).unwrap();

    let clamped = mgr.load_settings();
    assert_eq!(clamped.window_bounds.x, -50);
    assert_eq!(clamped.window_bounds.y, 20);
    assert_eq!(clamped.window_bounds.width, 200);
    assert_eq!(clamped.window_bounds.height, 150);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_flip_interval_serialization_and_clamping() {
    let mut settings = WazooSettings::default();
    assert_eq!(settings.flip_interval_secs, 45);

    // Verify serialization and deserialization
    settings.flip_interval_secs = 60;
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.flip_interval_secs, 60);

    // Verify legacy settings without flip_interval_secs defaults to 45
    let legacy_json = r#"{"window_opacity":1.0}"#;
    let legacy: WazooSettings = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(legacy.flip_interval_secs, 45);

    // Verify clamping in load_settings
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_flip_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);
    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    fs::write(mgr.config_file_path(), r#"{"flip_interval_secs": 0}"#).unwrap();
    let loaded = mgr.load_settings();
    assert_eq!(loaded.flip_interval_secs, 1);

    fs::write(mgr.config_file_path(), r#"{"flip_interval_secs": 99999}"#).unwrap();
    let loaded_max = mgr.load_settings();
    assert_eq!(loaded_max.flip_interval_secs, 3600);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_flip_mode_does_not_persist() {
    use wazoo_core::models::PlaybackMode;

    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_flip_persist_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::create_dir_all(&temp_dir);

    let mgr = ConfigManager {
        config_dir: temp_dir.clone(),
        data_dir: temp_dir.clone(),
    };

    // 1. If save_settings is called while in Flip mode, it should save as Normal mode
    let settings = WazooSettings {
        playback_mode: PlaybackMode::Flip,
        ..Default::default()
    };
    mgr.save_settings(&settings).unwrap();

    let content = fs::read_to_string(mgr.config_file_path()).unwrap();
    assert!(!content.contains(r#""playback_mode": "flip""#));
    assert!(content.contains(r#""playback_mode": "normal""#));

    let loaded = mgr.load_settings();
    assert_eq!(loaded.playback_mode, PlaybackMode::Normal);

    // 2. Scroll mode does persist
    let scroll_settings = WazooSettings {
        playback_mode: PlaybackMode::Scroll,
        ..Default::default()
    };
    mgr.save_settings(&scroll_settings).unwrap();
    let loaded_scroll = mgr.load_settings();
    assert_eq!(loaded_scroll.playback_mode, PlaybackMode::Scroll);

    // 3. Pre-existing settings file with "flip" is coerced to Normal on load
    fs::write(mgr.config_file_path(), r#"{"playback_mode": "flip"}"#).unwrap();
    let loaded_legacy = mgr.load_settings();
    assert_eq!(loaded_legacy.playback_mode, PlaybackMode::Normal);

    let _ = fs::remove_dir_all(&temp_dir);
}
