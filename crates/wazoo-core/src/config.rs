/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Configuration & Persistence Manager
 *
 * Resolves standard platform directories for config and database storage, loading and saving
 * JSON settings files and managing database paths.
 */

use crate::models::WazooSettings;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("com", "alkalisoftworks", "wazoo-rs")
            .expect("Unable to determine project directories");

        let config_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        if !data_dir.exists() {
            let _ = fs::create_dir_all(&data_dir);
        }

        Self {
            config_dir,
            data_dir,
        }
    }

    pub fn with_dirs(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        if !data_dir.exists() {
            let _ = fs::create_dir_all(&data_dir);
        }
        Self {
            config_dir,
            data_dir,
        }
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn database_path(&self) -> PathBuf {
        self.config_dir.join("wazoo.db")
    }

    pub fn load_settings(&self) -> WazooSettings {
        let path = self.config_file_path();
        let mut settings = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut settings) = serde_json::from_str::<WazooSettings>(&content) {
                    settings.buffer_size_mb = settings.buffer_size_mb.clamp(16, 4096);
                    settings.buffer_duration_secs = settings.buffer_duration_secs.clamp(2, 300);
                    settings.window_bounds.width = settings.window_bounds.width.clamp(200, 7680);
                    settings.window_bounds.height = settings.window_bounds.height.clamp(150, 4320);
                    settings.keybinds.reconcile_with_defaults();
                    settings
                } else {
                    WazooSettings::default()
                }
            } else {
                WazooSettings::default()
            }
        } else {
            WazooSettings::default()
        };

        if let Ok(val) = std::env::var("WAZOO_FLIP_INTERVAL") {
            if let Ok(secs) = val.trim().parse::<u64>() {
                settings.flip_interval_secs = secs;
            }
        }
        settings.flip_interval_secs = settings.flip_interval_secs.clamp(1, 3600);

        if settings.playback_mode == crate::models::PlaybackMode::Flip {
            settings.playback_mode = crate::models::PlaybackMode::Normal;
        }

        settings
    }

    pub fn has_keybinds_in_settings(&self) -> bool {
        self.has_complete_keybinds_in_settings()
    }

    pub fn has_complete_keybinds_in_settings(&self) -> bool {
        let path = self.config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    return crate::keybinds::KeybindSettings::is_complete_json(&val);
                }
            }
        }
        false
    }

    pub fn save_settings(&self, settings: &WazooSettings) -> Result<(), std::io::Error> {
        let path = self.config_file_path();
        let mut to_save = settings.clone();
        if to_save.playback_mode == crate::models::PlaybackMode::Flip {
            to_save.playback_mode = crate::models::PlaybackMode::Normal;
        }
        let json = serde_json::to_string_pretty(&to_save)?;
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, json)?;
        fs::rename(temp_path, path)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}


