use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use wazoo_core::ConfigManager;
use wazoo_core::models::{PlaybackMode, WazooSettings};

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

struct TempConfigDir(PathBuf);

impl TempConfigDir {
    fn new(prefix: &str) -> (Self, ConfigManager) {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}_{seq}", std::process::id(), nanos));
        let _ = fs::create_dir_all(&dir);
        let mgr = ConfigManager::with_dirs(dir.clone(), dir.clone());
        (Self(dir), mgr)
    }
}

impl Drop for TempConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

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
fn test_buffer_settings_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_clamp_test");
    let hostile_json = r#"{"buffer_size_mb": 999999999, "buffer_duration_secs": 99999}"#;
    fs::write(mgr.config_file_path(), hostile_json).unwrap();

    let loaded = mgr.load_settings();
    assert_eq!(loaded.buffer_size_mb, 4096);
    assert_eq!(loaded.buffer_duration_secs, 300);
}

#[test]
fn test_atomic_save_settings() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_atomic_test");
    let settings = WazooSettings {
        last_query: "secure_query".to_string(),
        ..Default::default()
    };
    mgr.save_settings(&settings).unwrap();

    assert!(mgr.config_file_path().exists());
    assert!(!mgr.config_file_path().with_extension("tmp").exists());

    let loaded = mgr.load_settings();
    assert_eq!(loaded.last_query, "secure_query");
}

#[test]
fn test_effective_subtitle_language_defaults_to_i18n_language() {
    let mut settings = WazooSettings::default();
    assert_eq!(settings.preferred_subtitle_language, None);
    assert_eq!(settings.language, "en");
    assert_eq!(settings.effective_subtitle_language(), "en");

    settings.language = "es".to_string();
    assert_eq!(settings.effective_subtitle_language(), "es");

    settings.preferred_subtitle_language = Some("Japanese".to_string());
    assert_eq!(settings.effective_subtitle_language(), "Japanese");

    settings.preferred_subtitle_language = None;
    assert_eq!(settings.effective_subtitle_language(), "es");

    settings.preferred_subtitle_language = Some("  ".to_string());
    assert_eq!(settings.effective_subtitle_language(), "es");
}

#[test]
fn test_keybinds_reconciliation_and_has_keybinds() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_keybinds_test");

    assert!(!mgr.has_keybinds_in_settings());

    let mut loaded = mgr.load_settings();
    assert_eq!(loaded.keybinds.add_player, "n");

    loaded.keybinds.add_player = "p".to_string();
    mgr.save_settings(&loaded).unwrap();
    assert!(mgr.has_keybinds_in_settings());

    let reloaded = mgr.load_settings();
    assert_eq!(reloaded.keybinds.add_player, "p");

    let incomplete_json = r#"{"keybinds":{"add_player":"p"}}"#;
    fs::write(mgr.config_file_path(), incomplete_json).unwrap();
    assert!(!mgr.has_complete_keybinds_in_settings());

    let reconciled = mgr.load_settings();
    assert_eq!(reconciled.keybinds.add_player, "p");
    assert_eq!(reconciled.keybinds.close_app, "Alt+X");
}

#[test]
fn test_window_bounds_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_bounds_test");
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
}

#[test]
fn test_flip_interval_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_flip_test");

    fs::write(mgr.config_file_path(), r#"{"flip_interval_secs": 0}"#).unwrap();
    assert_eq!(mgr.load_settings().flip_interval_secs, 1);

    fs::write(mgr.config_file_path(), r#"{"flip_interval_secs": 99999}"#).unwrap();
    assert_eq!(mgr.load_settings().flip_interval_secs, 3600);
}

#[test]
fn test_playback_mode_does_not_persist_scroll_or_flip() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_mode_persist_test");

    for mode in [PlaybackMode::Flip, PlaybackMode::Scroll] {
        let settings = WazooSettings {
            playback_mode: mode,
            ..Default::default()
        };
        mgr.save_settings(&settings).unwrap();

        let content = fs::read_to_string(mgr.config_file_path()).unwrap();
        assert!(content.contains(r#""playback_mode": "normal""#));
        assert_eq!(mgr.load_settings().playback_mode, PlaybackMode::Normal);
    }

    for persisted in [r#"{"playback_mode": "flip"}"#, r#"{"playback_mode": "scroll"}"#] {
        fs::write(mgr.config_file_path(), persisted).unwrap();
        assert_eq!(mgr.load_settings().playback_mode, PlaybackMode::Normal);
    }
}

#[test]
fn test_playback_settings_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_playback_clamp_test");
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
}

#[test]
fn test_cube_settings_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_cube_clamp_test");
    let extreme_settings = WazooSettings {
        cube_speed: 99.0,
        cube_size: 0.01,
        ..Default::default()
    };
    mgr.save_settings(&extreme_settings).unwrap();

    let loaded = mgr.load_settings();
    assert_eq!(loaded.cube_speed, 4.0);
    assert_eq!(loaded.cube_size, 0.4);
}

#[test]
fn test_crt_settings_clamping() {
    let (_tmp, mgr) = TempConfigDir::new("wazoo_crt_clamp_test");
    let mut extreme_crt = wazoo_core::CrtSettings::default();
    extreme_crt.scanline_darkness = 99.0;
    extreme_crt.gamma = -10.0;
    extreme_crt.glitch_intensity = 5.0;
    extreme_crt.glitch_frequency = -2.0;
    let extreme_settings = WazooSettings {
        crt: extreme_crt,
        ..Default::default()
    };
    mgr.save_settings(&extreme_settings).unwrap();

    let loaded = mgr.load_settings();
    assert_eq!(loaded.crt.scanline_darkness, 1.0);
    assert_eq!(loaded.crt.gamma, 0.2);
    assert_eq!(loaded.crt.glitch_intensity, 1.0);
    assert_eq!(loaded.crt.glitch_frequency, 0.0);
}

#[test]
fn test_theme_settings_hex_parsing_clamping_and_presets() {
    use wazoo_core::{
        DEFAULT_THEME_ACCENT_COLOR, DEFAULT_THEME_SURFACE_COLOR, DEFAULT_THEME_TITLEBAR_COLOR,
        ThemePreset, ThemeSettings, format_hex_rgb, parse_hex_rgb, sanitize_hex_or_default,
    };

    assert_eq!(parse_hex_rgb("#42b883"), Some((0x42, 0xb8, 0x83)));
    assert_eq!(parse_hex_rgb("1F092F"), Some((0x1f, 0x09, 0x2f)));
    assert_eq!(parse_hex_rgb("#fff"), Some((255, 255, 255)));
    assert_eq!(parse_hex_rgb("#abc"), Some((0xaa, 0xbb, 0xcc)));
    assert_eq!(parse_hex_rgb("#zzzzzz"), None);
    assert_eq!(parse_hex_rgb("#12"), None);
    assert_eq!(format_hex_rgb(0x42, 0xb8, 0x83), "#42b883");
    assert_eq!(sanitize_hex_or_default("not-a-color", "#1f1f1f"), "#1f1f1f");

    let def = ThemeSettings::default();
    assert_eq!(def.accent_color, DEFAULT_THEME_ACCENT_COLOR);
    assert_eq!(def.surface_color, DEFAULT_THEME_SURFACE_COLOR);
    assert_eq!(def.titlebar_color, DEFAULT_THEME_TITLEBAR_COLOR);
    assert!(def.matches_preset(ThemePreset::Default));

    for preset in [
        ThemePreset::Mocha,
        ThemePreset::Amber,
        ThemePreset::Strawberry,
        ThemePreset::Monokai,
        ThemePreset::Turtle,
    ] {
        let mut themed = ThemeSettings::default();
        themed.apply_preset(preset);
        assert!(themed.matches_preset(preset));
        assert!(!themed.matches_preset(ThemePreset::Default));
    }

    let mut bad = ThemeSettings {
        accent_color: "bad_hex".to_string(),
        surface_color: " #333 ".to_string(),
        titlebar_color: "invalid".to_string(),
        titlebar_badge_color: "000".to_string(),
        hud_color: "oops".to_string(),
        text_color: "FAFAFA".to_string(),
        hud_opacity: 5.0,
        corner_radius: -2.0,
        focus_ring_width: 99.0,
    };
    bad.clamp_all();
    assert_eq!(bad.accent_color, DEFAULT_THEME_ACCENT_COLOR);
    assert_eq!(bad.surface_color, "#333333");
    assert_eq!(bad.titlebar_color, DEFAULT_THEME_TITLEBAR_COLOR);
    assert_eq!(bad.titlebar_badge_color, "#000000");
    assert_eq!(bad.text_color, "#fafafa");
    assert_eq!(bad.hud_opacity, 1.0);
    assert_eq!(bad.corner_radius, 0.0);
    assert_eq!(bad.focus_ring_width, 8.0);
}
