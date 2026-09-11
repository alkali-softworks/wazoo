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
}
