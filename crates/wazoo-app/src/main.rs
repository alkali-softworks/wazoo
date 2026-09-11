use std::time::Duration;
use iced::{
    keyboard::{key::Named, Key},
    widget::{button, column, container, mouse_area, row, scrollable, slider, text, text_input, Space, Stack},
    window, Alignment, Element, Length, Subscription, Task, Theme,
};
use wazoo_core::{ConfigManager, Database, LayoutMode, PlaybackMode, VideoRecord, WazooSettings};
use wazoo_media::{BufferConfig, PlayerId, ScrollEngine, VideoHandle};
use wazoo_scanner::{Scanner, ScanProgress};

pub struct WazooApp {
    settings: WazooSettings,
    config_mgr: ConfigManager,
    db: Database,
    players: Vec<VideoHandle>,
    scroll_engine: ScrollEngine,
    available_videos: Vec<VideoRecord>,
    active_search_query: String,
    search_input: String,
    folder_input: String,
    show_search_modal: bool,
    show_settings_modal: bool,
    show_help_modal: bool,
    toast_message: Option<String>,
    toast_time_remaining: usize,
    next_player_id: PlayerId,
    is_scanning: bool,
    scan_progress: Option<ScanProgress>,
    focused_player_idx: usize,
    is_shuffle_mode: bool,
    show_wazoo_controls: bool,
    show_title_overlay: bool,
    hovered_player_id: Option<PlayerId>,
    subtitles_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Keyboard Event
    KeyPressed(Key),

    // Playback controls
    TogglePlay(PlayerId),
    TogglePlayFocused,
    NextVideo(PlayerId),
    NextVideoFocused,
    PrevVideoFocused,
    Seek(PlayerId, Duration),
    SeekRatio(PlayerId, f32),
    SeekRelativeFocused(f64),
    PlayerHovered(PlayerId),
    PlayerUnhovered(PlayerId),
    SetVolume(PlayerId, f64),
    AdjustVolumeFocused(f64),
    TogglePlayerMute(PlayerId),
    ToggleMuteFocused,
    ToggleGlobalMute,
    GlobalUnmute,
    ToggleShuffleMode,

    // Layout & Modes
    CycleLayout,
    SetPlayerCount(usize),
    ToggleScrollMode,
    ToggleFlipMode,
    SetScrollSpeed(f32),
    AdjustScrollSpeed(f32),
    AddNewPlayer,
    RemoveFocusedPlayer,
    CycleFocusedPlayer,
    SetFocusedPlayer(usize),

    // Modals & UI
    OpenSearchModal,
    CloseSearchModal,
    SearchInputChanged(String),
    PerformSearch,
    OpenSettingsModal,
    CloseSettingsModal,
    SetBufferDuration(u32),
    OpenHelpModal,
    CloseHelpModal,
    FolderInputChanged(String),
    PickFolders,
    FoldersSelected(Vec<String>),
    AddMediaFolder,
    RemoveMediaFolder(String),
    StartScan,
    ScanProgressUpdate(ScanProgress),
    ScanFinished(Result<usize, String>),
    ToggleControlsHUD,
    ToggleTitleOverlay,
    ToggleSubtitles,
    EscapePressed,

    // Window management
    ToggleDecorations,
    WindowResized(u32, u32),

    // Timers & Ticks
    AnimationTick,
    WatchdogTick,
    FlipModeTick,
    DismissToast,
}

impl WazooApp {
    pub fn new() -> (Self, Task<Message>) {
        let config_mgr = ConfigManager::new();
        let settings = config_mgr.load_settings();
        let db = Database::open(config_mgr.database_path())
            .expect("Failed to initialize SQLite database");

        let videos = db.get_all_videos().unwrap_or_default();
        let scroll_engine = ScrollEngine::new(settings.window_bounds.height as f32);

        let mut app = Self {
            settings: settings.clone(),
            config_mgr,
            db,
            players: Vec::new(),
            scroll_engine,
            available_videos: videos,
            active_search_query: settings.last_query.clone(),
            search_input: settings.last_query.clone(),
            folder_input: String::new(),
            show_search_modal: false,
            show_settings_modal: false,
            show_help_modal: false,
            toast_message: Some("Welcome to Wazoo (Native Rust)".to_string()),
            toast_time_remaining: 3,
            next_player_id: 1,
            is_scanning: false,
            scan_progress: None,
            focused_player_idx: 0,
            is_shuffle_mode: true,
            show_wazoo_controls: true,
            show_title_overlay: false,
            hovered_player_id: None,
            subtitles_enabled: true,
        };

        // Initialize players based on settings
        let count = settings.player_count.clamp(1, 12);
        for _ in 0..count {
            app.add_player_internal();
        }

        (app, Task::none())
    }

    fn focused_player_id(&self) -> Option<PlayerId> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_player_idx % self.players.len();
            Some(self.players[idx].id)
        }
    }

    fn get_next_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
        if self.available_videos.is_empty() {
            return None;
        }
        if self.is_shuffle_mode {
            let idx = rand::random::<usize>() % self.available_videos.len();
            Some(self.available_videos[idx].clone())
        } else {
            if let Some(curr) = current_path {
                if let Some(pos) = self.available_videos.iter().position(|v| v.path == curr) {
                    let next_pos = (pos + 1) % self.available_videos.len();
                    return Some(self.available_videos[next_pos].clone());
                }
            }
            Some(self.available_videos[0].clone())
        }
    }

    fn get_prev_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
        if self.available_videos.is_empty() {
            return None;
        }
        if let Some(curr) = current_path {
            if let Some(pos) = self.available_videos.iter().position(|v| v.path == curr) {
                let prev_pos = if pos == 0 { self.available_videos.len() - 1 } else { pos - 1 };
                return Some(self.available_videos[prev_pos].clone());
            }
        }
        Some(self.available_videos[0].clone())
    }

    fn create_video_handle(&self, id: PlayerId, path: &str, name: &str) -> Result<VideoHandle, String> {
        let buffer_config = BufferConfig {
            duration_secs: self.settings.buffer_duration_secs,
            size_mb: self.settings.buffer_size_mb,
            read_chunk_kb: 512,
        };
        VideoHandle::with_buffering(id, path, name, buffer_config)
    }

    fn add_player_internal(&mut self) -> Option<PlayerId> {
        let video_rec = self.get_next_video_rec(None)?;

        let id = self.next_player_id;
        self.next_player_id += 1;

        match self.create_video_handle(id, &video_rec.path, &video_rec.name) {
            Ok(mut handle) => {
                handle.set_muted(self.settings.is_global_muted);
                self.players.push(handle);
                Some(id)
            }
            Err(err) => {
                log::error!("Failed to create VideoHandle for {}: {}", video_rec.path, err);
                None
            }
        }
    }

    pub fn title(&self) -> String {
        format!("Wazoo - Ambient Media Engine ({} videos)", self.available_videos.len())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(key) => {
                if self.show_search_modal || self.show_settings_modal || self.show_help_modal {
                    if key == Key::Named(Named::Escape) {
                        self.show_search_modal = false;
                        self.show_settings_modal = false;
                        self.show_help_modal = false;
                    }
                    return Task::none();
                }

                match key {
                    Key::Named(Named::Space) => return self.update(Message::TogglePlayFocused),
                    Key::Named(Named::ArrowUp) => return self.update(Message::NextVideoFocused),
                    Key::Named(Named::ArrowDown) => return self.update(Message::PrevVideoFocused),
                    Key::Named(Named::ArrowLeft) => return self.update(Message::SeekRelativeFocused(-5.0)),
                    Key::Named(Named::ArrowRight) => return self.update(Message::SeekRelativeFocused(5.0)),
                    Key::Named(Named::Tab) => return self.update(Message::CycleFocusedPlayer),
                    Key::Named(Named::Escape) => return self.update(Message::EscapePressed),
                    Key::Character(s) => match s.as_str() {
                        "f" | "F" | "j" | "J" | "/" => return self.update(Message::OpenSearchModal),
                        "s" | "S" => return self.update(Message::ToggleShuffleMode),
                        "1" => return self.update(Message::SetPlayerCount(1)),
                        "2" => return self.update(Message::SetPlayerCount(2)),
                        "3" => return self.update(Message::SetPlayerCount(3)),
                        "4" => return self.update(Message::SetPlayerCount(4)),
                        "5" => return self.update(Message::ToggleScrollMode),
                        "6" => return self.update(Message::ToggleFlipMode),
                        "l" | "L" => return self.update(Message::CycleLayout),
                        "n" | "N" => return self.update(Message::AddNewPlayer),
                        "x" | "X" => return self.update(Message::RemoveFocusedPlayer),
                        "m" | "M" => return self.update(Message::ToggleMuteFocused),
                        "[" => return self.update(Message::AdjustVolumeFocused(-0.1)),
                        "]" => return self.update(Message::AdjustVolumeFocused(0.1)),
                        "-" => return self.update(Message::AdjustScrollSpeed(-0.1)),
                        "+" | "=" => return self.update(Message::AdjustScrollSpeed(0.1)),
                        "c" | "C" => return self.update(Message::ToggleSubtitles),
                        "t" | "T" => return self.update(Message::ToggleTitleOverlay),
                        "h" | "H" => return self.update(Message::ToggleControlsHUD),
                        "?" => return self.update(Message::OpenHelpModal),
                        "," => return self.update(Message::SeekRelativeFocused(-0.04)),
                        "." => return self.update(Message::SeekRelativeFocused(0.04)),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Message::TogglePlay(id) => {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.toggle_play();
                }
            }
            Message::TogglePlayFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlay(id));
                }
            }
            Message::NextVideo(id) => {
                let curr_path = self.players.iter().find(|p| p.id == id).map(|p| p.state.path.clone());
                if let Some(video_rec) = self.get_next_video_rec(curr_path.as_deref()) {
                    if let Ok(mut new_handle) = self.create_video_handle(id, &video_rec.path, &video_rec.name) {
                        new_handle.set_muted(self.settings.is_global_muted);
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                            *p = new_handle;
                        }
                    }
                }
            }
            Message::NextVideoFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::NextVideo(id));
                }
            }
            Message::PrevVideoFocused => {
                if let Some(id) = self.focused_player_id() {
                    let curr_path = self.players.iter().find(|p| p.id == id).map(|p| p.state.path.clone());
                    if let Some(prev_rec) = self.get_prev_video_rec(curr_path.as_deref()) {
                        if let Ok(mut new_handle) = self.create_video_handle(id, &prev_rec.path, &prev_rec.name) {
                            new_handle.set_muted(self.settings.is_global_muted);
                            if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                                *p = new_handle;
                            }
                        }
                    }
                }
            }
            Message::Seek(id, pos) => {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.seek(pos);
                }
            }
            Message::SeekRatio(id, ratio) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    self.focused_player_idx = pos;
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    let dur = p.duration();
                    if dur > Duration::ZERO {
                        let target_secs = dur.as_secs_f64() * (ratio.clamp(0.0, 1.0) as f64);
                        let target = Duration::from_secs_f64(target_secs);
                        p.seek(target);

                        let pos_s = target.as_secs();
                        let dur_s = dur.as_secs();
                        self.toast_message = Some(format!(
                            "Seek [{:02}:{:02} / {:02}:{:02}]",
                            pos_s / 60,
                            pos_s % 60,
                            dur_s / 60,
                            dur_s % 60
                        ));
                        self.toast_time_remaining = 2;
                    }
                }
            }
            Message::PlayerHovered(id) => {
                self.hovered_player_id = Some(id);
            }
            Message::PlayerUnhovered(id) => {
                if self.hovered_player_id == Some(id) {
                    self.hovered_player_id = None;
                }
            }
            Message::SeekRelativeFocused(secs) => {
                if let Some(id) = self.focused_player_id() {
                    self.hovered_player_id = Some(id);
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                        p.seek_relative(secs);
                        let pos = p.position();
                        let dur = p.duration();
                        let pos_s = pos.as_secs();
                        let dur_s = dur.as_secs();
                        let sign = if secs > 0.0 { "+" } else { "" };
                        self.toast_message = Some(format!(
                            "Seek {sign}{:.0}s  [{:02}:{:02} / {:02}:{:02}]",
                            secs,
                            pos_s / 60,
                            pos_s % 60,
                            dur_s / 60,
                            dur_s % 60
                        ));
                        self.toast_time_remaining = 2;
                    }
                }
            }
            Message::SetVolume(id, vol) => {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.set_volume(vol);
                }
            }
            Message::AdjustVolumeFocused(delta) => {
                if self.settings.playback_mode == PlaybackMode::Scroll && delta > 0.0 {
                    return self.update(Message::GlobalUnmute);
                }
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                        p.adjust_volume(delta);
                        self.toast_message = Some(format!("Volume: {:.0}%", p.state.volume * 100.0));
                        self.toast_time_remaining = 1;
                    }
                }
            }
            Message::TogglePlayerMute(id) => {
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    let muted = !p.state.is_muted;
                    p.set_muted(muted);
                }
            }
            Message::ToggleMuteFocused => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    return self.update(Message::ToggleGlobalMute);
                }
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlayerMute(id));
                }
            }
            Message::ToggleGlobalMute => {
                self.settings.is_global_muted = !self.settings.is_global_muted;
                for p in &mut self.players {
                    p.set_muted(self.settings.is_global_muted);
                }
                self.toast_message = Some(if self.settings.is_global_muted {
                    "Global Mute: ON".to_string()
                } else {
                    "Global Mute: OFF".to_string()
                });
                self.toast_time_remaining = 2;
            }
            Message::GlobalUnmute => {
                self.settings.is_global_muted = false;
                for p in &mut self.players {
                    p.set_muted(false);
                }
                self.toast_message = Some("Global Mute: OFF".to_string());
                self.toast_time_remaining = 2;
            }
            Message::ToggleShuffleMode => {
                self.is_shuffle_mode = !self.is_shuffle_mode;
                self.toast_message = Some(if self.is_shuffle_mode {
                    "Switched to shuffle mode".to_string()
                } else {
                    "Switched to sequential mode".to_string()
                });
                self.toast_time_remaining = 2;
            }
            Message::CycleLayout => {
                self.settings.layout = match self.settings.layout {
                    LayoutMode::Grid => LayoutMode::Row,
                    LayoutMode::Row => LayoutMode::Column,
                    LayoutMode::Column => LayoutMode::Grid,
                };
                self.toast_message = Some(format!("Layout: {:?}", self.settings.layout));
                self.toast_time_remaining = 2;
            }
            Message::SetPlayerCount(count) => {
                let target = count.clamp(1, 12);
                self.settings.player_count = target;
                while self.players.len() > target {
                    self.players.pop();
                }
                while self.players.len() < target {
                    self.add_player_internal();
                }
                self.toast_message = Some(format!("Players: {target}"));
                self.toast_time_remaining = 2;
            }
            Message::ToggleScrollMode => {
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Scroll => PlaybackMode::Normal,
                    _ => PlaybackMode::Scroll,
                };
                self.toast_message = Some(format!("Scroll Mode: {:?}", self.settings.playback_mode));
                self.toast_time_remaining = 2;
            }
            Message::ToggleFlipMode => {
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Flip => PlaybackMode::Normal,
                    _ => PlaybackMode::Flip,
                };
                self.toast_message = Some(format!("Flip Mode: {:?}", self.settings.playback_mode));
                self.toast_time_remaining = 2;
            }
            Message::SetScrollSpeed(speed) => {
                self.settings.scroll_speed = speed.clamp(0.1, 10.0);
                self.scroll_engine.set_speed(self.settings.scroll_speed);
                self.toast_message = Some(format!("Scroll Speed: {:.1}", self.settings.scroll_speed));
                self.toast_time_remaining = 1;
            }
            Message::AdjustScrollSpeed(delta) => {
                let new_speed = (self.settings.scroll_speed + delta).clamp(0.1, 10.0);
                return self.update(Message::SetScrollSpeed(new_speed));
            }
            Message::AddNewPlayer => {
                let new_count = (self.players.len() + 1).min(12);
                return self.update(Message::SetPlayerCount(new_count));
            }
            Message::RemoveFocusedPlayer => {
                if let Some(id) = self.focused_player_id() {
                    self.players.retain(|p| p.id != id);
                    self.settings.player_count = self.players.len();
                    if self.focused_player_idx >= self.players.len() && !self.players.is_empty() {
                        self.focused_player_idx = self.players.len() - 1;
                    }
                    self.toast_message = Some(format!("Players: {}", self.players.len()));
                    self.toast_time_remaining = 2;
                }
            }
            Message::CycleFocusedPlayer => {
                if !self.players.is_empty() {
                    self.focused_player_idx = (self.focused_player_idx + 1) % self.players.len();
                    self.toast_message = Some(format!("Focused Player: {}", self.focused_player_idx + 1));
                    self.toast_time_remaining = 1;
                }
            }
            Message::SetFocusedPlayer(idx) => {
                if idx < self.players.len() {
                    self.focused_player_idx = idx;
                }
            }
            Message::OpenSearchModal => {
                self.show_search_modal = true;
            }
            Message::CloseSearchModal => {
                self.show_search_modal = false;
            }
            Message::SearchInputChanged(val) => {
                self.search_input = val;
            }
            Message::PerformSearch => {
                self.active_search_query = self.search_input.clone();
                let folders = self.settings.media_folders.clone();
                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    self.toast_message = Some(format!("Found {} videos", results.len()));
                    self.toast_time_remaining = 2;
                    self.available_videos = results;
                }
                self.show_search_modal = false;
            }
            Message::OpenSettingsModal => {
                self.show_settings_modal = true;
            }
            Message::CloseSettingsModal => {
                self.show_settings_modal = false;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::SetBufferDuration(secs) => {
                self.settings.buffer_duration_secs = secs;
                let _ = self.config_mgr.save_settings(&self.settings);
                self.toast_message = Some(format!("Buffer set to {secs}s (active on next video load)"));
                self.toast_time_remaining = 3;
            }
            Message::OpenHelpModal => {
                self.show_help_modal = true;
            }
            Message::CloseHelpModal => {
                self.show_help_modal = false;
            }
            Message::PickFolders => {
                let starting_dir = self.settings.media_folders.first().cloned();
                return Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title("Select Media Folder(s)");
                        if let Some(ref dir) = starting_dir {
                            dialog = dialog.set_directory(dir);
                        }
                        if let Some(handles) = dialog.pick_folders().await {
                            handles
                                .into_iter()
                                .map(|h| h.path().to_string_lossy().to_string())
                                .collect()
                        } else {
                            Vec::new()
                        }
                    },
                    Message::FoldersSelected,
                );
            }
            Message::FoldersSelected(folders) => {
                if folders.is_empty() {
                    return Task::none();
                }
                let mut added_any = false;
                for folder in folders {
                    let trimmed = folder.trim().to_string();
                    if !trimmed.is_empty() && !self.settings.media_folders.contains(&trimmed) {
                        self.settings.media_folders.push(trimmed);
                        added_any = true;
                    }
                }
                if added_any {
                    let _ = self.config_mgr.save_settings(&self.settings);
                    self.toast_message = Some("Added folder(s). Starting library scan...".to_string());
                    self.toast_time_remaining = 3;

                    if !self.is_scanning && !self.settings.media_folders.is_empty() {
                        self.is_scanning = true;
                        let folders = self.settings.media_folders.clone();
                        let db_path = self.config_mgr.database_path();
                        return Task::perform(
                            async move {
                                let scanner = Scanner::new(None);
                                scanner.scan_and_index(&folders, db_path, None).await
                            },
                            Message::ScanFinished,
                        );
                    }
                }
            }
            Message::FolderInputChanged(val) => {
                self.folder_input = val;
            }
            Message::AddMediaFolder => {
                let trimmed = self.folder_input.trim().to_string();
                if !trimmed.is_empty() {
                    if !self.settings.media_folders.contains(&trimmed) {
                        self.settings.media_folders.push(trimmed.clone());
                        self.folder_input.clear();
                        let _ = self.config_mgr.save_settings(&self.settings);
                        self.toast_message = Some(format!("Added folder: {trimmed}"));
                        self.toast_time_remaining = 3;

                        if !self.is_scanning {
                            self.is_scanning = true;
                            let folders = self.settings.media_folders.clone();
                            let db_path = self.config_mgr.database_path();
                            return Task::perform(
                                async move {
                                    let scanner = Scanner::new(None);
                                    scanner.scan_and_index(&folders, db_path, None).await
                                },
                                Message::ScanFinished,
                            );
                        }
                    } else {
                        self.folder_input.clear();
                    }
                } else {
                    return Task::done(Message::PickFolders);
                }
            }
            Message::RemoveMediaFolder(folder) => {
                self.settings.media_folders.retain(|f| f != &folder);
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::StartScan => {
                if !self.is_scanning && !self.settings.media_folders.is_empty() {
                    self.is_scanning = true;
                    let folders = self.settings.media_folders.clone();
                    let db_path = self.config_mgr.database_path();

                    return Task::perform(
                        async move {
                            let scanner = Scanner::new(None);
                            scanner.scan_and_index(&folders, db_path, None).await
                        },
                        Message::ScanFinished,
                    );
                }
            }
            Message::ScanProgressUpdate(progress) => {
                self.scan_progress = Some(progress);
            }
            Message::ScanFinished(res) => {
                self.is_scanning = false;
                self.scan_progress = None;
                match res {
                    Ok(count) => {
                        self.toast_message = Some(format!("Indexed {count} videos!"));
                        self.toast_time_remaining = 3;
                        if let Ok(videos) = self.db.get_all_videos() {
                            self.available_videos = videos;
                            if self.players.is_empty() && !self.available_videos.is_empty() {
                                let count = self.settings.player_count.clamp(1, 12);
                                for _ in 0..count {
                                    self.add_player_internal();
                                }
                            }
                        }
                    }
                    Err(err) => {
                        self.toast_message = Some(format!("Scan error: {err}"));
                        self.toast_time_remaining = 3;
                    }
                }
            }
            Message::ToggleControlsHUD => {
                self.show_wazoo_controls = !self.show_wazoo_controls;
            }
            Message::ToggleTitleOverlay => {
                self.show_title_overlay = !self.show_title_overlay;
            }
            Message::ToggleSubtitles => {
                self.subtitles_enabled = !self.subtitles_enabled;
                self.toast_message = Some(if self.subtitles_enabled {
                    "Subtitles: ON".to_string()
                } else {
                    "Subtitles: OFF".to_string()
                });
                self.toast_time_remaining = 2;
            }
            Message::EscapePressed => {
                self.show_help_modal = !self.show_help_modal;
            }
            Message::ToggleDecorations => {}
            Message::WindowResized(w, h) => {
                self.settings.window_bounds.width = w;
                self.settings.window_bounds.height = h;
                self.scroll_engine.set_window_height(h as f32);
            }
            Message::AnimationTick => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    let offscreen = self.scroll_engine.tick();
                    for id in offscreen {
                        self.players.retain(|p| p.id != id);
                    }

                    if let Some(spawn_y) = self.scroll_engine.needs_new_player() {
                        if let Some(id) = self.add_player_internal() {
                            let height = 360.0;
                            self.scroll_engine.add_item(id, spawn_y, height);
                        }
                    }

                    for p in &mut self.players {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }
                }
            }
            Message::WatchdogTick => {
                // Check if any video finished or is stuck
                let finished_or_stuck: Vec<PlayerId> = self
                    .players
                    .iter_mut()
                    .filter_map(|p| {
                        if p.is_finished() || p.check_stuck() {
                            Some(p.id)
                        } else {
                            None
                        }
                    })
                    .collect();

                for id in finished_or_stuck {
                    let _ = self.update(Message::NextVideo(id));
                }

                if self.toast_message.is_some() {
                    if self.toast_time_remaining > 0 {
                        self.toast_time_remaining -= 1;
                    } else {
                        self.toast_message = None;
                    }
                }
            }
            Message::FlipModeTick => {
                if self.settings.playback_mode == PlaybackMode::Flip && !self.players.is_empty() {
                    let rand_id = self.players[rand::random::<usize>() % self.players.len()].id;
                    let _ = self.update(Message::NextVideo(rand_id));
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == rand_id) {
                        p.seek_random();
                    }
                }
            }
            Message::DismissToast => {
                self.toast_message = None;
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            iced::time::every(Duration::from_secs(1)).map(|_| Message::WatchdogTick),
            iced::event::listen_with(|event, _status, _window| {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
                    Some(Message::KeyPressed(key))
                } else {
                    None
                }
            }),
        ];

        if self.settings.playback_mode == PlaybackMode::Scroll {
            subs.push(iced::time::every(Duration::from_millis(16)).map(|_| Message::AnimationTick));
        }

        if self.settings.playback_mode == PlaybackMode::Flip {
            subs.push(iced::time::every(Duration::from_secs(15)).map(|_| Message::FlipModeTick));
        }

        Subscription::batch(subs)
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.available_videos.is_empty() {
            return self.view_welcome();
        }

        let players_view = self.view_players();

        let mut content = column![players_view];

        // Bottom control bar overlay
        if self.show_wazoo_controls {
            let controls = row![
                button(text("🔍 Find (F)")).on_press(Message::OpenSearchModal),
                button(text(format!("Layout: {:?} (L)", self.settings.layout))).on_press(Message::CycleLayout),
                button(text(if self.is_shuffle_mode { "🔀 Shuffle (S)" } else { "🔁 Sequential (S)" }))
                    .on_press(Message::ToggleShuffleMode),
                button(text(if self.settings.playback_mode == PlaybackMode::Scroll { "🌊 Scroll: ON (5)" } else { "🌊 Scroll: OFF (5)" }))
                    .on_press(Message::ToggleScrollMode),
                button(text(if self.settings.playback_mode == PlaybackMode::Flip { "⚡ Flip: ON (6)" } else { "⚡ Flip: OFF (6)" }))
                    .on_press(Message::ToggleFlipMode),
                button(text(if self.settings.is_global_muted { "🔇 Unmute (M)" } else { "🔊 Mute (M)" }))
                    .on_press(Message::ToggleGlobalMute),
                button(text("⚙ Settings")).on_press(Message::OpenSettingsModal),
                button(text("❓ Help (?)")).on_press(Message::OpenHelpModal),
            ]
            .spacing(8)
            .padding(8);

            content = content.push(controls);
        }

        let base_layer = container(content)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut root_stack_children = vec![Element::from(base_layer)];

        // Floating Toast / Notice Overlay (Zero reflow - floats over top center)
        if let Some(ref toast) = self.toast_message {
            let toast_widget = container(
                row![
                    text(toast).size(15).color(iced::Color::WHITE),
                    button(text("✕").size(12)).on_press(Message::DismissToast).padding(2),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([8, 18])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.85))),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            });

            let toast_layer = container(
                column![
                    Space::new().height(Length::Fixed(24.0)),
                    toast_widget,
                ]
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill);

            root_stack_children.push(Element::from(toast_layer));
        }

        // Floating Modal Overlays (Zero reflow - video continues in background)
        if self.show_search_modal {
            root_stack_children.push(self.view_search_modal());
        } else if self.show_settings_modal {
            root_stack_children.push(self.view_settings_modal());
        } else if self.show_help_modal {
            root_stack_children.push(self.view_help_modal());
        }

        Stack::with_children(root_stack_children)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_players(&self) -> Element<'_, Message> {
        if self.players.is_empty() {
            return text("No active players").into();
        }

        match self.settings.layout {
            LayoutMode::Row => {
                let mut r = row![].spacing(6).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    r = r.push(self.view_single_player(p));
                }
                r.into()
            }
            LayoutMode::Column => {
                let mut c = column![].spacing(6).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    c = c.push(self.view_single_player(p));
                }
                c.into()
            }
            LayoutMode::Grid => {
                let count = self.players.len();
                let cols = if count <= 1 { 1 } else if count <= 4 { 2 } else if count <= 9 { 3 } else { 4 };

                let mut rows = column![].spacing(6).width(Length::Fill).height(Length::Fill);
                for chunk in self.players.chunks(cols) {
                    let mut r = row![].spacing(6).width(Length::Fill).height(Length::Fill);
                    for p in chunk {
                        r = r.push(self.view_single_player(p));
                    }
                    rows = rows.push(r);
                }
                rows.into()
            }
        }
    }

    fn view_single_player<'a>(&self, p: &'a VideoHandle) -> Element<'a, Message> {
        let player_id = p.id;
        let is_focused = self.focused_player_id() == Some(player_id);
        let is_hovered = self.hovered_player_id == Some(player_id);

        let pos = p.position();
        let dur = p.duration();
        let pos_secs = pos.as_secs();
        let dur_secs = dur.as_secs();
        let time_str = format!(
            "{:02}:{:02} / {:02}:{:02}",
            pos_secs / 60,
            pos_secs % 60,
            dur_secs / 60,
            dur_secs % 60
        );

        let progress_ratio = if dur.as_secs_f64() > 0.0 {
            (pos.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0) as f32
        } else {
            0.0f32
        };

        let video_widget = iced_video_player::VideoPlayer::new(&p.video)
            .width(Length::Fill)
            .height(Length::Fill);

        let show_overlay = is_hovered || !p.state.is_playing || self.show_title_overlay;

        let mut stack_children: Vec<Element<'a, Message>> = vec![Element::from(video_widget)];

        if show_overlay {
            let focus_indicator = if is_focused { "▶ " } else { "" };
            let title_text = format!("{focus_indicator}{}", p.state.name);

            let top_bar = container(
                row![
                    text(title_text).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                ]
                .align_y(Alignment::Center),
            )
            .padding(8)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.65))),
                ..Default::default()
            });

            let seek_bar = slider(
                0.0..=1.0,
                progress_ratio,
                move |ratio| Message::SeekRatio(player_id, ratio),
            )
            .step(0.001)
            .width(Length::Fill);

            let bottom_controls = container(
                column![
                    seek_bar,
                    row![
                        button(text(if p.state.is_playing { "⏸" } else { "▶" }))
                            .on_press(Message::TogglePlay(player_id)),
                        button(text("⏮"))
                            .on_press(Message::SeekRelativeFocused(-5.0)),
                        button(text("⏭"))
                            .on_press(Message::NextVideo(player_id)),
                        button(text(if p.state.is_muted { "🔇" } else { "🔊" }))
                            .on_press(Message::TogglePlayerMute(player_id)),
                        button(text("CC"))
                            .on_press(Message::ToggleSubtitles),
                        Space::new().width(Length::Fill),
                        text(time_str).size(12).color(iced::Color::WHITE),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(4),
            )
            .padding(8)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
                ..Default::default()
            });

            let overlay_layer = column![
                top_bar,
                Space::new().height(Length::Fill),
                bottom_controls,
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            stack_children.push(Element::from(overlay_layer));
        }

        let player_stack = Stack::with_children(stack_children)
            .width(Length::Fill)
            .height(Length::Fill);

        mouse_area(player_stack)
            .on_enter(Message::PlayerHovered(player_id))
            .on_exit(Message::PlayerUnhovered(player_id))
            .into()
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        let mut col = column![
            text("Welcome to Wazoo").size(28),
            text("Ambient media engine for non-stop viewing").size(16),
            text("No videos indexed yet. Add your media folder(s) to start:").size(14),
            row![
                button(text("📁 Add Folder")).on_press(Message::PickFolders).padding(10),
                button(text(if self.is_scanning { "Scanning..." } else { "Start Scan" }))
                    .on_press(Message::StartScan)
                    .padding(10),
            ]
            .spacing(12),
            row![
                text_input("Or enter folder path manually...", &self.folder_input)
                    .on_input(Message::FolderInputChanged)
                    .on_submit(Message::AddMediaFolder)
                    .padding(8)
                    .width(Length::Fixed(350.0)),
                button(text("Add Path")).on_press(Message::AddMediaFolder).padding(8),
            ]
            .spacing(10),
        ]
        .spacing(16)
        .align_x(Alignment::Center);

        if !self.settings.media_folders.is_empty() {
            let mut folders_col = column![text("Configured Folders:").size(14)].spacing(6);
            for f in &self.settings.media_folders {
                let f_clone = f.clone();
                folders_col = folders_col.push(
                    row![
                        text(format!("• {f}")).size(13),
                        button(text("✕")).on_press(Message::RemoveMediaFolder(f_clone)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
            col = col.push(folders_col);
        }

        if self.is_scanning {
            col = col.push(text("Scanning media folders and indexing videos...").size(13));
        }

        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_search_modal(&self) -> Element<'_, Message> {
        let card = container(
            column![
                text("Instant Search (Find)").size(22),
                row![
                    text_input("Search videos...", &self.search_input)
                        .on_input(Message::SearchInputChanged)
                        .on_submit(Message::PerformSearch)
                        .padding(10)
                        .width(Length::Fixed(350.0)),
                    button(text("Search")).on_press(Message::PerformSearch).padding(8),
                    button(text("Cancel (Esc)")).on_press(Message::CloseSearchModal).padding(8),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(format!("Current results: {} videos", self.available_videos.len())).size(12),
            ]
            .spacing(14),
        )
        .padding(24)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.12, 0.12, 0.14, 0.96))),
            border: iced::Border {
                radius: 12.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        });

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
                ..Default::default()
            })
            .into()
    }

    fn view_settings_modal(&self) -> Element<'_, Message> {
        let mut folders_col = column![text("Media Folders:").size(16)].spacing(6);
        for folder in &self.settings.media_folders {
            let f = folder.clone();
            folders_col = folders_col.push(
                row![
                    text(folder).size(13),
                    Space::new().width(Length::Fill),
                    button(text("Remove")).on_press(Message::RemoveMediaFolder(f)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            );
        }

        let content = column![
            text("Wazoo Settings").size(22),
            folders_col,
            row![
                button(text("📁 Add Folder")).on_press(Message::PickFolders).padding(8),
                button(text(if self.is_scanning { "Scanning..." } else { "Re-scan Library" }))
                    .on_press(Message::StartScan)
                    .padding(8),
            ]
            .spacing(10),
            row![
                text_input("Or enter folder path manually...", &self.folder_input)
                    .on_input(Message::FolderInputChanged)
                    .on_submit(Message::AddMediaFolder)
                    .width(Length::Fixed(280.0))
                    .padding(6),
                button(text("Add Path")).on_press(Message::AddMediaFolder).padding(6),
            ]
            .spacing(8),
            row![
                text("Players count:"),
                button(text("1")).on_press(Message::SetPlayerCount(1)),
                button(text("2")).on_press(Message::SetPlayerCount(2)),
                button(text("3")).on_press(Message::SetPlayerCount(3)),
                button(text("4")).on_press(Message::SetPlayerCount(4)),
                button(text("6")).on_press(Message::SetPlayerCount(6)),
                button(text("8")).on_press(Message::SetPlayerCount(8)),
            ]
            .spacing(8),
            row![
                text("WiFi / Samba Buffer:"),
                button(text(if self.settings.buffer_duration_secs == 5 { "5s (Low) ★" } else { "5s" }))
                    .on_press(Message::SetBufferDuration(5)),
                button(text(if self.settings.buffer_duration_secs == 10 { "10s (Recommended) ★" } else { "10s" }))
                    .on_press(Message::SetBufferDuration(10)),
                button(text(if self.settings.buffer_duration_secs == 20 { "20s (High) ★" } else { "20s" }))
                    .on_press(Message::SetBufferDuration(20)),
                button(text(if self.settings.buffer_duration_secs == 30 { "30s (Max) ★" } else { "30s" }))
                    .on_press(Message::SetBufferDuration(30)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            button(text("Done")).on_press(Message::CloseSettingsModal).padding(8),
        ]
        .spacing(16);

        let card = container(scrollable(content))
            .padding(24)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.12, 0.12, 0.14, 0.96))),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            });

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
                ..Default::default()
            })
            .into()
    }

    fn view_help_modal(&self) -> Element<'_, Message> {
        let content = column![
            text("Wazoo Help & Keyboard Shortcuts").size(22),
            text("• F / j / /: Find / Search modal").size(14),
            text("• S: Toggle Shuffle / Sequential playback mode").size(14),
            text("• 1, 2, 3, 4: Set player count to 1, 2, 3, or 4").size(14),
            text("• 5: Toggle Scroll Mode (The Infinity Stream)").size(14),
            text("• 6: Toggle Flip Mode (staggered auto-shuffle)").size(14),
            text("• Space: Play / Pause focused video").size(14),
            text("• ArrowUp: Play next video on focused player").size(14),
            text("• ArrowDown: Play previous video on focused player").size(14),
            text("• ArrowLeft / Right: Seek -5s / +5s").size(14),
            text("• , / .: Frame backward / forward").size(14),
            text("• L: Cycle Layout (Grid ➔ Row ➔ Column)").size(14),
            text("• N: Add new player (up to 12)").size(14),
            text("• X: Remove focused player").size(14),
            text("• Tab: Cycle focused player").size(14),
            text("• M: Toggle Mute (Global in scroll mode)").size(14),
            text("• [ / ]: Volume Down / Up").size(14),
            text("• C: Toggle Subtitles").size(14),
            text("• T: Toggle Title Overlay").size(14),
            text("• H: Toggle Bottom Controls HUD").size(14),
            text("• - / +: Adjust scroll speed").size(14),
            text("• Esc: Close modal / cancel").size(14),
            button(text("Close (Esc)")).on_press(Message::CloseHelpModal).padding(8),
        ]
        .spacing(12);

        let card = container(scrollable(content))
            .padding(24)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.12, 0.12, 0.14, 0.96))),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            });

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.75))),
                ..Default::default()
            })
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(WazooApp::new, WazooApp::update, WazooApp::view)
        .title(WazooApp::title)
        .subscription(WazooApp::subscription)
        .theme(WazooApp::theme)
        .window(window::Settings {
            size: iced::Size::new(1280.0, 720.0),
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .run()
}
