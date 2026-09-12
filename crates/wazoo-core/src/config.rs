/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Configuration & Persistence Manager
 * 
 * Resolves standard platform directories for config and database storage, loading and saving
 * JSON settings files and managing database paths.
 */

use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::models::WazooSettings;

pub struct ConfigManager {
    config_dir: PathBuf,
    data_dir: PathBuf,
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

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("wazoo.db")
    }

    pub fn load_settings(&self) -> WazooSettings {
        let path = self.config_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<WazooSettings>(&content) {
                    return settings;
                }
            }
        }
        WazooSettings::default()
    }

    pub fn save_settings(&self, settings: &WazooSettings) -> Result<(), std::io::Error> {
        let path = self.config_file_path();
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(path, json)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_paths() {
        let mgr = ConfigManager::new();
        assert!(mgr.config_file_path().to_string_lossy().contains("wazoo-rs"));
        assert!(mgr.database_path().to_string_lossy().contains("wazoo-rs"));
        assert!(mgr.config_dir.exists());
        assert!(mgr.data_dir.exists());
    }

    #[test]
    fn test_session_videos_serialization() {
        let mut settings = WazooSettings::default();
        settings.session_videos.push(crate::models::VideoSession {
            path: "/path/to/video1.mp4".to_string(),
            position_secs: 42.5,
            is_muted: false,
            volume: 0.8,
        });

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: WazooSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_videos.len(), 1);
        assert_eq!(deserialized.session_videos[0].path, "/path/to/video1.mp4");
        assert_eq!(deserialized.session_videos[0].position_secs, 42.5);
        assert_eq!(deserialized.session_videos[0].is_muted, false);
        assert_eq!(deserialized.session_videos[0].volume, 0.8);

        // Verify backwards compatibility when session_videos is omitted from JSON
        let json_legacy = r#"{"window_bounds":{"x":0,"y":0,"width":1280,"height":720},"window_opacity":1.0,"media_folders":[],"player_count":1,"layout":"grid","playback_mode":"normal","scroll_speed":1.0,"is_global_muted":true,"last_query":"","last_folder":"All"}"#;
        let legacy_settings: WazooSettings = serde_json::from_str(json_legacy).unwrap();
        assert!(legacy_settings.session_videos.is_empty());
        assert!(legacy_settings.bookmarks.is_empty());
    }

    #[test]
    fn test_bookmarks_serialization() {
        let mut settings = WazooSettings::default();
        settings.bookmarks.push(crate::models::Bookmark {
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
        assert_eq!(deserialized.bookmarks[0].is_shuffle, false);

        // Verify legacy bookmark JSON without is_shuffle defaults to true
        let legacy_bookmark_json = r#"{"name":"Legacy","query":"test","path":"/path/test.mp4","position_secs":10.0}"#;
        let legacy_bookmark: crate::models::Bookmark = serde_json::from_str(legacy_bookmark_json).unwrap();
        assert_eq!(legacy_bookmark.is_shuffle, true);
    }
}
