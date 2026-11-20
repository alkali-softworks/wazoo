use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use wazoo_app::app::WazooApp;
use wazoo_app::message::Message;
use wazoo_core::{ConfigManager, Database};

static TEMP_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

pub struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    pub fn new(prefix: &str) -> Self {
        let seq = TEMP_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "{prefix}_{}_{}_{}",
            std::process::id(),
            nanos,
            seq
        ));
        let _ = std::fs::create_dir_all(&path);
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub fn new_backend_app(prefix: &str) -> (WazooApp, TempTestDir) {
    let temp = TempTestDir::new(prefix);
    let config_mgr =
        ConfigManager::with_dirs(temp.path().to_path_buf(), temp.path().to_path_buf());
    let db = Database::open_in_memory().expect("in-memory db");
    let (app, _) = WazooApp::new_with_backend(None, config_mgr, db);
    (app, temp)
}

pub fn press_char(app: &mut WazooApp, ch: &str) {
    let _ = app.update(Message::KeyPressed(
        iced::keyboard::Key::Character(ch.into()),
        iced::event::Status::Ignored,
    ));
}
