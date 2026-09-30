use std::fs;
use wazoo_core::ConfigManager;
use wazoo_core::models::WazooSettings;

#[test]
fn test_config_manager_paths() {
    let mgr = ConfigManager::new();
    assert!(mgr.config_file_path().to_string_lossy().contains("wazoo"));
    assert!(mgr.database_path().to_string_lossy().contains("wazoo"));
    assert_eq!(
        mgr.database_path().parent(),
        mgr.config_file_path().parent()
    );
    assert_eq!(mgr.database_path(), mgr.config_dir.join("wazoo.db"));
    assert!(mgr.config_dir.exists());
    assert!(mgr.data_dir.exists());
}

#[test]
fn test_default_player_setting_serialization() {
    let mut settings = WazooSettings::default();
    assert!(!settings.is_default_player);

    settings.is_default_player = true;
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert!(deserialized.is_default_player);
}

#[test]
fn test_session_videos_serialization() {
    let mut settings = WazooSettings::default();
    settings
        .session_videos
        .push(wazoo_core::models::VideoSession {
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

    // Loading settings provides default keybinds
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
fn test_playback_mode_does_not_persist_scroll_or_flip() {
    use wazoo_core::models::PlaybackMode;

    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_mode_persist_test_{}",
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

    // 2. If save_settings is called while in Scroll mode, it should save as Normal mode
    let scroll_settings = WazooSettings {
        playback_mode: PlaybackMode::Scroll,
        ..Default::default()
    };
    mgr.save_settings(&scroll_settings).unwrap();
    let content_scroll = fs::read_to_string(mgr.config_file_path()).unwrap();
    assert!(!content_scroll.contains(r#""playback_mode": "scroll""#));
    assert!(content_scroll.contains(r#""playback_mode": "normal""#));

    let loaded_scroll = mgr.load_settings();
    assert_eq!(loaded_scroll.playback_mode, PlaybackMode::Normal);

    // 3. Pre-existing settings file with "flip" is coerced to Normal on load (boot)
    fs::write(mgr.config_file_path(), r#"{"playback_mode": "flip"}"#).unwrap();
    let loaded_flip = mgr.load_settings();
    assert_eq!(loaded_flip.playback_mode, PlaybackMode::Normal);

    // 4. Pre-existing settings file with "scroll" is coerced to Normal on load (boot)
    fs::write(mgr.config_file_path(), r#"{"playback_mode": "scroll"}"#).unwrap();
    let loaded_scroll_setting = mgr.load_settings();
    assert_eq!(loaded_scroll_setting.playback_mode, PlaybackMode::Normal);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_playback_settings_serialization_and_clamping() {
    let mut settings = WazooSettings::default();
    assert_eq!(settings.gamma, 0.0);
    assert_eq!(settings.contrast, 0.0);
    assert_eq!(settings.brightness, 0.0);
    assert_eq!(settings.saturation, 0.0);
    assert_eq!(settings.playback_speed, 1.0);

    settings.gamma = 15.0;
    settings.contrast = -20.0;
    settings.brightness = 5.0;
    settings.saturation = 30.0;
    settings.playback_speed = 1.25;

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.gamma, 15.0);
    assert_eq!(deserialized.contrast, -20.0);
    assert_eq!(deserialized.brightness, 5.0);
    assert_eq!(deserialized.saturation, 30.0);
    assert_eq!(deserialized.playback_speed, 1.25);

    // Verify clamping via ConfigManager
    let temp_dir = std::env::temp_dir().join(format!(
        "wazoo_playback_clamp_test_{}",
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

    let extreme_settings = WazooSettings {
        gamma: 250.0,
        contrast: -150.0,
        brightness: 999.0,
        saturation: -500.0,
        playback_speed: 10.0,
        ..Default::default()
    };
    mgr.save_settings(&extreme_settings).unwrap();

    let loaded = mgr.load_settings();
    assert_eq!(loaded.gamma, 100.0);
    assert_eq!(loaded.contrast, -100.0);
    assert_eq!(loaded.brightness, 100.0);
    assert_eq!(loaded.saturation, -100.0);
    assert_eq!(loaded.playback_speed, 4.0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_scroll_mode_muted_default() {
    let settings = WazooSettings::default();
    assert!(settings.scroll_mode_muted);

    // Verify explicit false
    let json_unmuted = r#"{"scroll_mode_muted": false}"#;
    let unmuted: WazooSettings = serde_json::from_str(json_unmuted).unwrap();
    assert!(!unmuted.scroll_mode_muted);
}

#[test]
fn test_crt_enabled_serialization() {
    let default_settings = WazooSettings::default();
    assert!(!default_settings.crt_enabled);

    let mut settings = WazooSettings::default();
    settings.crt_enabled = true;
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
    assert!(deserialized.crt_enabled);

    // Verify default fallback when missing in json
    let missing_json = r#"{}"#;
    let loaded: WazooSettings = serde_json::from_str(missing_json).unwrap();
    assert!(!loaded.crt_enabled);
}

