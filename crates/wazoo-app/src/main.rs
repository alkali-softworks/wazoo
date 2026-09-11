mod format;
mod theme;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;
use iced::futures::SinkExt;
use iced::{
    keyboard::{key::Named, Key},
    widget::{button, column, container, mouse_area, row, scrollable, slider, svg, text, text_input, Space, Stack},
    Alignment, Color, Element, Length, Point, Subscription, Task, Theme,
};
use wazoo_core::{ConfigManager, Database, LayoutMode, PlaybackMode, VideoRecord, WazooSettings};
use wazoo_media::{BufferConfig, PlayerId, ScrollEngine, VideoHandle};
use wazoo_scanner::{ScanProgress, ScanStage, Scanner};

static APP_ICON_BYTES: &[u8] = include_bytes!("../resources/icon.png");

// Crisp vector SVGs for window controls matching modern desktop apps
static SVG_WINDOW_MINIMIZE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><line x1="0" y1="5" x2="10" y2="5" stroke="#cccccc" stroke-width="1.2"/></svg>"##;
static SVG_WINDOW_MAXIMIZE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" fill="none"><rect x="1.5" y="1.5" width="7" height="7" stroke="#cccccc" stroke-width="1.2"/></svg>"##;
static SVG_WINDOW_CLOSE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="#cccccc" stroke-width="1.2"/><line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="#cccccc" stroke-width="1.2"/></svg>"##;

// Lucide-style 24x24 vector playback SVGs for bottom player controls
static SVG_PLAYER_PLAY: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><polygon points="6 3 20 12 6 21 6 3"/></svg>"##;
static SVG_PLAYER_PAUSE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><rect x="5" y="3" width="4" height="18" rx="1"/><rect x="15" y="3" width="4" height="18" rx="1"/></svg>"##;
static SVG_PLAYER_NEXT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><polygon points="5 4 15 12 5 20 5 4"/><rect x="17" y="4" width="3" height="16" rx="1"/></svg>"##;
static SVG_PLAYER_VOLUME: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" fill="#ffffff"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>"##;
static SVG_PLAYER_MUTE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" fill="#ffffff"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>"##;

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
    selected_search_folder: String,
    show_search_modal: bool,
    show_settings_modal: bool,
    show_help_modal: bool,
    show_menu_modal: bool,
    show_file_picker: bool,
    file_picker_search: String,
    show_titlebar: bool,
    titlebar_hide_ticks: usize,
    show_dropdown_menu: bool,
    is_alt_pressed: bool,
    player_overlay_ticks: usize,
    window_id: Option<iced::window::Id>,
    app_icon_handle: iced::widget::image::Handle,
    toast_message: Option<String>,
    toast_time_remaining: usize,
    next_player_id: PlayerId,
    is_scanning: bool,
    scan_progress: Option<ScanProgress>,
    focused_player_idx: usize,
    focus_border_ticks: usize,
    is_shuffle_mode: bool,
    hovered_player_id: Option<PlayerId>,
    subtitles_enabled: bool,
    loading_player_ids: HashSet<PlayerId>,
    loading_player_ticks: HashMap<PlayerId, usize>,
    spinner_ticks: u32,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Window management & Events
    WindowIdReceived(iced::window::Id),
    CursorMoved(iced::window::Id, Point),
    RightClickPressed(iced::window::Id),
    KeyPressed(Key),
    KeyReleased(Key),
    MinimizeWindow,
    MaximizeWindow,
    CloseApp,
    DragWindow,
    ToggleDropdownMenu,
    CloseDropdownMenu,
    OpenMenuModal,
    CloseMenuModal,
    ToggleFilePicker,
    FilePickerSearchChanged(String),
    PlayFileInFocused(String),
    SelectSearchFolder(String),
    SetWindowOpacity(f32),

    // Playback controls
    PlayerClicked(PlayerId),
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
    ToggleSubtitles,
    EscapePressed,

    // Timers & Ticks
    AnimationTick,
    VideoFrameTick,
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

        let videos: Vec<VideoRecord> = db.get_all_videos().unwrap_or_default();
        let scroll_engine = ScrollEngine::new(settings.window_bounds.height as f32);

        let icon_handle = iced::widget::image::Handle::from_bytes(APP_ICON_BYTES);

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
            selected_search_folder: "All".to_string(),
            show_search_modal: false,
            show_settings_modal: false,
            show_help_modal: false,
            show_menu_modal: false,
            show_file_picker: false,
            file_picker_search: String::new(),
            show_titlebar: false,
            titlebar_hide_ticks: 0,
            show_dropdown_menu: false,
            is_alt_pressed: false,
            player_overlay_ticks: 0,
            window_id: None,
            app_icon_handle: icon_handle,
            toast_message: Some("Welcome to Wazoo".to_string()),
            toast_time_remaining: 3,
            next_player_id: 1,
            is_scanning: false,
            scan_progress: None,
            focused_player_idx: 0,
            focus_border_ticks: 0,
            is_shuffle_mode: true,
            hovered_player_id: None,
            subtitles_enabled: true,
            loading_player_ids: HashSet::new(),
            loading_player_ticks: HashMap::new(),
            spinner_ticks: 0,
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
                let prev_pos = if pos == 0 {
                    self.available_videos.len() - 1
                } else {
                    pos - 1
                };
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

    pub fn current_opacity(&self) -> f32 {
        if self.is_alt_pressed {
            0.85
        } else {
            self.settings.window_opacity.clamp(0.1, 1.0)
        }
    }

    fn add_player_internal(&mut self) -> Option<PlayerId> {
        let id = self.next_player_id;
        self.next_player_id += 1;

        for _ in 0..3 {
            if let Some(video_rec) = self.get_next_video_rec(None) {
                match self.create_video_handle(id, &video_rec.path, &video_rec.name) {
                    Ok(mut handle) => {
                        handle.set_muted(self.settings.is_global_muted);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.players.push(handle);
                        self.loading_player_ids.insert(id);
                        self.loading_player_ticks.insert(id, 0);
                        return Some(id);
                    }
                    Err(err) => {
                        log::error!("Failed to create VideoHandle for {}: {}", video_rec.path, err);
                    }
                }
            }
        }
        None
    }

    pub fn title(&self) -> String {
        format!("Wazoo - Ambient Media Engine ({} videos)", self.available_videos.len())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowIdReceived(id) => {
                self.window_id = Some(id);
            }
            Message::CursorMoved(win_id, pos) => {
                self.window_id = Some(win_id);
                self.player_overlay_ticks = 120; // 2 seconds delay before hiding controls

                if pos.y < 35.0 || self.show_dropdown_menu {
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = 25;
                } else if !self.show_dropdown_menu {
                    if self.titlebar_hide_ticks > 0 {
                        self.titlebar_hide_ticks -= 1;
                    } else {
                        self.show_titlebar = false;
                    }
                }
            }
            Message::RightClickPressed(win_id) => {
                self.window_id = Some(win_id);
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
            }
            Message::KeyPressed(key) => {
                // If Alt key pressed
                if key == Key::Named(Named::Alt) || key == Key::Named(Named::AltGraph) {
                    self.is_alt_pressed = true;
                    return Task::none();
                }

                // If Alt is held and X is pressed: quit app
                if self.is_alt_pressed {
                    if let Key::Character(ref s) = key {
                        if s.eq_ignore_ascii_case("x") {
                            return self.update(Message::CloseApp);
                        }
                    }
                }

                if key == Key::Named(Named::Escape) {
                    return self.update(Message::EscapePressed);
                }

                match key {
                    Key::Named(Named::Space) => return self.update(Message::TogglePlayFocused),
                    Key::Named(Named::ArrowUp) => return self.update(Message::NextVideoFocused),
                    Key::Named(Named::ArrowDown) => return self.update(Message::PrevVideoFocused),
                    Key::Named(Named::ArrowLeft) => return self.update(Message::SeekRelativeFocused(-5.0)),
                    Key::Named(Named::ArrowRight) => return self.update(Message::SeekRelativeFocused(5.0)),
                    Key::Named(Named::Tab) => return self.update(Message::CycleFocusedPlayer),
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
                        "h" | "H" => return self.update(Message::ToggleFilePicker),
                        "?" => return self.update(Message::OpenHelpModal),
                        "," => return self.update(Message::SeekRelativeFocused(-0.04)),
                        "." => return self.update(Message::SeekRelativeFocused(0.04)),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Message::KeyReleased(key) => {
                if key == Key::Named(Named::Alt) || key == Key::Named(Named::AltGraph) {
                    self.is_alt_pressed = false;
                }
            }
            Message::MinimizeWindow => {
                if let Some(id) = self.window_id {
                    return iced::window::minimize(id, true);
                }
            }
            Message::MaximizeWindow => {
                if let Some(id) = self.window_id {
                    return iced::window::toggle_maximize(id);
                }
            }
            Message::CloseApp => {
                if let Some(id) = self.window_id {
                    return iced::window::close(id);
                } else {
                    std::process::exit(0);
                }
            }
            Message::DragWindow => {
                if let Some(id) = self.window_id {
                    return iced::window::drag(id);
                }
            }
            Message::ToggleDropdownMenu => {
                self.show_dropdown_menu = !self.show_dropdown_menu;
                if self.show_dropdown_menu {
                    self.show_titlebar = true;
                }
            }
            Message::CloseDropdownMenu => {
                self.show_dropdown_menu = false;
            }
            Message::OpenMenuModal => {
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
            }
            Message::CloseMenuModal => {
                self.show_menu_modal = false;
            }
            Message::ToggleFilePicker => {
                self.show_file_picker = !self.show_file_picker;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
            }
            Message::FilePickerSearchChanged(s) => {
                self.file_picker_search = s;
            }
            Message::PlayFileInFocused(path) => {
                if let Some(id) = self.focused_player_id() {
                    self.loading_player_ids.insert(id);
                    self.loading_player_ticks.insert(id, 0);
                    let title = format::format_video_title(&path);
                    if let Ok(mut handle) = self.create_video_handle(id, &path, &title) {
                        handle.set_muted(self.settings.is_global_muted);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                            *p = handle;
                        }
                        self.toast_message = Some(format!("Playing: {title}"));
                        self.toast_time_remaining = 3;
                    }
                }
            }
            Message::SelectSearchFolder(folder) => {
                self.selected_search_folder = folder;
            }
            Message::SetWindowOpacity(opacity) => {
                self.settings.window_opacity = opacity.clamp(0.1, 1.0);
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::PlayerClicked(id) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                    self.player_overlay_ticks = 120;
                }
            }
            Message::TogglePlay(id) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.toggle_play();
                }
                self.player_overlay_ticks = 120;
            }
            Message::TogglePlayFocused => {
                if let Some(id) = self.focused_player_id() {
                    return self.update(Message::TogglePlay(id));
                }
            }
            Message::NextVideo(id) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                self.loading_player_ids.insert(id);
                self.loading_player_ticks.insert(id, 0);
                self.player_overlay_ticks = 120;
                let curr_path = self.players.iter().find(|p| p.id == id).map(|p| p.state.path.clone());
                for _ in 0..3 {
                    if let Some(video_rec) = self.get_next_video_rec(curr_path.as_deref()) {
                        if let Ok(mut new_handle) = self.create_video_handle(id, &video_rec.path, &video_rec.name) {
                            new_handle.set_muted(self.settings.is_global_muted);
                            new_handle.set_subtitles_visible(self.subtitles_enabled);
                            if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                                *p = new_handle;
                            }
                            break;
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
                    self.loading_player_ids.insert(id);
                    self.loading_player_ticks.insert(id, 0);
                    let curr_path = self.players.iter().find(|p| p.id == id).map(|p| p.state.path.clone());
                    for _ in 0..3 {
                        if let Some(prev_rec) = self.get_prev_video_rec(curr_path.as_deref()) {
                            if let Ok(mut new_handle) = self.create_video_handle(id, &prev_rec.path, &prev_rec.name) {
                                new_handle.set_muted(self.settings.is_global_muted);
                                new_handle.set_subtitles_visible(self.subtitles_enabled);
                                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                                    *p = new_handle;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            Message::Seek(id, pos) => {
                if let Some(pos_idx) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos_idx;
                    self.focused_player_idx = pos_idx;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.seek(pos);
                }
                self.player_overlay_ticks = 120;
            }
            Message::SeekRatio(id, ratio) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    let dur = p.duration();
                    if dur > Duration::ZERO {
                        let target_secs = dur.as_secs_f64() * (ratio.clamp(0.0, 1.0) as f64);
                        let target = Duration::from_secs_f64(target_secs);
                        p.seek(target);

                        let pos_s = target.as_secs_f64();
                        let dur_s = dur.as_secs_f64();
                        self.toast_message = Some(format!(
                            "Seek [{} / {}]",
                            format::format_time_str(pos_s),
                            format::format_time_str(dur_s)
                        ));
                        self.toast_time_remaining = 2;
                    }
                }
                self.player_overlay_ticks = 120;
            }
            Message::PlayerHovered(id) => {
                self.hovered_player_id = Some(id);
                self.player_overlay_ticks = 120;
            }
            Message::PlayerUnhovered(id) => {
                if self.hovered_player_id == Some(id) {
                    self.hovered_player_id = None;
                    self.player_overlay_ticks = 0;
                }
            }
            Message::SeekRelativeFocused(secs) => {
                self.player_overlay_ticks = 120;
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                        p.seek_relative(secs);
                        let pos = p.position();
                        let dur = p.duration();
                        let sign = if secs > 0.0 { "+" } else { "" };
                        self.toast_message = Some(format!(
                            "Seek {sign}{:.0}s  [{} / {}]",
                            secs,
                            format::format_time_str(pos.as_secs_f64()),
                            format::format_time_str(dur.as_secs_f64())
                        ));
                        self.toast_time_remaining = 2;
                    }
                }
            }
            Message::SetVolume(id, vol) => {
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.set_volume(vol);
                }
                self.player_overlay_ticks = 120;
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
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    let muted = !p.state.is_muted;
                    p.set_muted(muted);
                    self.toast_message = Some(if muted { "Muted".to_string() } else { "Unmuted".to_string() });
                    self.toast_time_remaining = 2;
                }
                self.player_overlay_ticks = 120;
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
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.settings.layout = match self.settings.layout {
                    LayoutMode::Grid => LayoutMode::Row,
                    LayoutMode::Row => LayoutMode::Column,
                    LayoutMode::Column => LayoutMode::Grid,
                };
                self.toast_message = Some(format!("Layout: {:?}", self.settings.layout));
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
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
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::ToggleScrollMode => {
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Scroll => PlaybackMode::Normal,
                    _ => PlaybackMode::Scroll,
                };
                self.toast_message = Some(match self.settings.playback_mode {
                    PlaybackMode::Scroll => "Scroll Mode: Enabled".to_string(),
                    _ => "Scroll Mode: Disabled".to_string(),
                });
                self.toast_time_remaining = 2;
            }
            Message::ToggleFlipMode => {
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Flip => PlaybackMode::Normal,
                    _ => PlaybackMode::Flip,
                };
                self.toast_message = Some(match self.settings.playback_mode {
                    PlaybackMode::Flip => "Flip Mode: Enabled".to_string(),
                    _ => "Flip Mode: Disabled".to_string(),
                });
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
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
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
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
            }
            Message::CycleFocusedPlayer => {
                if !self.players.is_empty() {
                    let next_idx = (self.focused_player_idx + 1) % self.players.len();
                    if next_idx != self.focused_player_idx {
                        self.focus_border_ticks = 20;
                    }
                    self.focused_player_idx = next_idx;
                    self.toast_message = Some(format!("Focused Player: {}", self.focused_player_idx + 1));
                    self.toast_time_remaining = 1;
                }
            }
            Message::SetFocusedPlayer(idx) => {
                if idx < self.players.len() {
                    let was_already_active = self.focused_player_idx == idx;
                    self.focused_player_idx = idx;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
            }
            Message::OpenSearchModal => {
                self.show_search_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
            }
            Message::CloseSearchModal => {
                self.show_search_modal = false;
            }
            Message::SearchInputChanged(val) => {
                self.search_input = val;
            }
            Message::PerformSearch => {
                self.active_search_query = self.search_input.clone();
                let folders = if self.selected_search_folder == "All" {
                    self.settings.media_folders.clone()
                } else {
                    vec![self.selected_search_folder.clone()]
                };

                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.toast_message = Some(format!("Total files: {total}"));
                    self.toast_time_remaining = 3;
                }
                self.show_search_modal = false;
            }
            Message::OpenSettingsModal => {
                self.show_settings_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
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
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
            }
            Message::CloseHelpModal => {
                self.show_help_modal = false;
            }
            Message::PickFolders => {
                let starting_dir = self.settings.media_folders.first().cloned();
                return Task::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new().set_title("Select Media Folder(s)");
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
                    self.scan_progress = None;
                    let folders = self.settings.media_folders.clone();
                    let db_path = self.config_mgr.database_path();

                    return Task::stream(iced::stream::channel(100, |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

                        let scan_handle = tokio::spawn(async move {
                            let scanner = Scanner::default();
                            scanner.scan_and_index(&folders, db_path, Some(tx)).await
                        });

                        while let Some(progress) = rx.recv().await {
                            let _ = output.send(Message::ScanProgressUpdate(progress)).await;
                        }

                        let res = match scan_handle.await {
                            Ok(inner_res) => inner_res,
                            Err(join_err) => Err(join_err.to_string()),
                        };

                        let _ = output.send(Message::ScanFinished(res)).await;
                    }));
                }
            }
            Message::ScanProgressUpdate(progress) => {
                match progress.stage {
                    ScanStage::Listing => {
                        let name_part = if progress.current_name.is_empty() {
                            String::new()
                        } else {
                            format!(": {}", progress.current_name)
                        };
                        self.toast_message = Some(format!(
                            "Scanning{} ({}% - {} found)",
                            name_part,
                            progress.percent,
                            progress.files_found
                        ));
                        self.toast_time_remaining = 2;
                    }
                    ScanStage::Indexing => {
                        if !progress.current_name.is_empty() {
                            self.toast_message = Some(format!("Added: {} ({}%)", progress.current_name, progress.percent));
                            self.toast_time_remaining = 2;
                        }
                    }
                }
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
            Message::ToggleSubtitles => {
                self.subtitles_enabled = !self.subtitles_enabled;
                for p in &mut self.players {
                    p.set_subtitles_visible(self.subtitles_enabled);
                }
                self.toast_message = Some(if self.subtitles_enabled {
                    "Subtitles: Enabled".to_string()
                } else {
                    "Subtitles: Disabled".to_string()
                });
                self.toast_time_remaining = 2;
            }
            Message::EscapePressed => {
                if self.show_help_modal
                    || self.show_search_modal
                    || self.show_settings_modal
                    || self.show_menu_modal
                    || self.show_dropdown_menu
                    || self.show_file_picker
                {
                    self.show_help_modal = false;
                    self.show_search_modal = false;
                    self.show_settings_modal = false;
                    self.show_menu_modal = false;
                    self.show_dropdown_menu = false;
                    self.show_file_picker = false;
                } else {
                    self.show_menu_modal = true;
                    self.show_dropdown_menu = false;
                }
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
            Message::VideoFrameTick => {
                self.spinner_ticks = self.spinner_ticks.wrapping_add(1);
                if self.player_overlay_ticks > 0 {
                    self.player_overlay_ticks -= 1;
                }
                if self.focus_border_ticks > 0 {
                    self.focus_border_ticks -= 1;
                }
                for p in &mut self.players {
                    if p.update_frame() {
                        self.loading_player_ids.remove(&p.id);
                        self.loading_player_ticks.remove(&p.id);
                    }
                }
            }
            Message::WatchdogTick => {
                // Auto-clear loading state if it exceeds 10 seconds to avoid indefinite spinner
                let stale_loading: Vec<PlayerId> = self.loading_player_ticks.iter_mut()
                    .filter_map(|(&id, ticks)| {
                        *ticks += 1;
                        if *ticks >= 10 { Some(id) } else { None }
                    })
                    .collect();
                for id in stale_loading {
                    self.loading_player_ids.remove(&id);
                    self.loading_player_ticks.remove(&id);
                }
                self.loading_player_ids.retain(|id| self.players.iter().any(|p| p.id == *id));
                self.loading_player_ticks.retain(|id, _| self.players.iter().any(|p| p.id == *id));

                let mut finished_ids = Vec::new();
                let mut stuck_ids = Vec::new();

                for p in &mut self.players {
                    if p.is_finished() {
                        finished_ids.push(p.id);
                    } else if p.check_stuck() {
                        stuck_ids.push(p.id);
                    }
                }

                for id in finished_ids {
                    log::info!("Player {id} video reached end, advancing to next video");
                    let _ = self.update(Message::NextVideo(id));
                }

                for id in stuck_ids {
                    log::warn!(
                        "Player {id} playback stuck for {}s, skipping to next video",
                        VideoHandle::STUCK_THRESHOLD_SECONDS
                    );
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
                    self.loading_player_ids.insert(rand_id);
                    self.loading_player_ticks.insert(rand_id, 0);
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
            iced::time::every(Duration::from_millis(16)).map(|_| Message::VideoFrameTick),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::WatchdogTick),
            iced::event::listen_with(|event, _status, window_id| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                    Some(Message::KeyPressed(key))
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) => {
                    Some(Message::KeyReleased(key))
                }
                iced::Event::Window(iced::window::Event::Unfocused) => {
                    Some(Message::KeyReleased(iced::keyboard::Key::Named(iced::keyboard::key::Named::Alt)))
                }
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Some(Message::CursorMoved(window_id, position))
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) => {
                    Some(Message::RightClickPressed(window_id))
                }
                _ => None,
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
        let main_content: Element<'_, Message> = if self.available_videos.is_empty() {
            self.view_welcome()
        } else if self.show_file_picker {
            row![
                container(self.view_players()).width(Length::Fill).height(Length::Fill),
                self.view_file_picker(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            row![container(self.view_players()).width(Length::Fill).height(Length::Fill)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let mut root_stack_children: Vec<Element<'_, Message>> = vec![main_content];

        // 1.5. Dropdown backdrop for dismissal when clicking outside
        if self.show_dropdown_menu {
            root_stack_children.push(Element::from(
                mouse_area(container(Space::new()).width(Length::Fill).height(Length::Fill))
                    .on_press(Message::CloseDropdownMenu),
            ));
        }

        // 2. Sliding Titlebar & Dropdown Menu Overlay
        if self.show_titlebar {
            root_stack_children.push(self.view_titlebar());
        }

        // 3. Floating Notice (Matches Electron Notice.vue)
        if let Some(ref toast) = self.toast_message {
            let toast_widget = container(
                row![
                    text(toast).size(16).color(iced::Color::WHITE),
                    button(text("✕").size(12))
                        .style(theme::window_control_button_style)
                        .on_press(Message::DismissToast)
                        .padding(2),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([6, 18])
            .style(theme::notice_pill_style);

            let toast_layer = container(
                column![
                    Space::new().height(Length::Fixed(35.0)),
                    toast_widget,
                ]
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill);

            root_stack_children.push(Element::from(toast_layer));
        }

        // 4. Scan Toast Banner (Matches Electron scan toast across top)
        if self.is_scanning {
            let (label, status_text) = if let Some(ref progress) = self.scan_progress {
                match progress.stage {
                    ScanStage::Listing => {
                        let name = if progress.current_name.is_empty() {
                            "Discovering files...".to_string()
                        } else {
                            format!("Listing: {}", progress.current_name)
                        };
                        let stat = format!("{}% ({} found)", progress.percent, progress.files_found);
                        (name, stat)
                    }
                    ScanStage::Indexing => {
                        let name = if progress.current_name.is_empty() {
                            "Indexing library...".to_string()
                        } else {
                            format!("Indexing: {}", progress.current_name)
                        };
                        let stat = format!("{}% ({} files)", progress.percent, progress.total);
                        (name, stat)
                    }
                }
            } else {
                ("Scanning media folders...".to_string(), "In progress".to_string())
            };

            let scan_banner = container(
                row![
                    text(label).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    text(status_text).size(13).color(theme::COLOR_PRIMARY),
                ]
                .padding([4, 24])
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fixed(28.0))
            .style(theme::scan_toast_banner_style);

            root_stack_children.push(Element::from(scan_banner));
        }

        // 5. Alt Drag Overlay (Matches Electron Alt overlay)
        if self.is_alt_pressed {
            let alt_overlay = container(
                column![
                    text("Drag to move").size(22).color(iced::Color::WHITE),
                    text("X to quit").size(16).color(theme::COLOR_TEXT_DIM),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            });

            root_stack_children.push(Element::from(mouse_area(alt_overlay).on_press(Message::DragWindow)));
        }

        // 6. Floating Modals (Zero reflow - video playback continues)
        if self.show_search_modal {
            root_stack_children.push(self.view_search_modal());
        } else if self.show_settings_modal {
            root_stack_children.push(self.view_settings_modal());
        } else if self.show_help_modal {
            root_stack_children.push(self.view_help_modal());
        } else if self.show_menu_modal {
            root_stack_children.push(self.view_menu_modal());
        }

        Stack::with_children(root_stack_children)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_titlebar(&self) -> Element<'_, Message> {
        let badge_btn = button(
            row![
                iced::widget::image(self.app_icon_handle.clone())
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
                text("Wazoo").size(16).color(iced::Color::WHITE),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(theme::titlebar_badge_style)
        .on_press(Message::ToggleDropdownMenu)
        .padding(iced::Padding {
            top: 2.0,
            right: 14.0,
            bottom: 2.0,
            left: 8.0,
        });

        let drag_strip = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .on_press(Message::DragWindow);

        let window_buttons = row![
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_MINIMIZE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style)
            .on_press(Message::MinimizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_MAXIMIZE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::window_control_button_style)
            .on_press(Message::MaximizeWindow)
            .padding(0),
            button(
                container(
                    svg(svg::Handle::from_memory(SVG_WINDOW_CLOSE))
                        .width(Length::Fixed(10.0))
                        .height(Length::Fixed(10.0))
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(30.0))
            .style(theme::close_window_button_style)
            .on_press(Message::CloseApp)
            .padding(0),
        ]
        .width(Length::Shrink);

        let titlebar_row = container(
            row![
                badge_btn,
                drag_strip,
                window_buttons,
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(30.0)),
        )
        .width(Length::Fill)
        .height(Length::Fixed(30.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme::COLOR_TITLEBAR_BG)),
            ..Default::default()
        });

        if self.show_dropdown_menu {
            let menu_dropdown = container(
                column![
                    button(text("Add Player")).style(theme::menu_item_style).on_press(Message::AddNewPlayer).padding([8, 14]).width(Length::Fill),
                    button(text("Toggle Layout")).style(theme::menu_item_style).on_press(Message::CycleLayout).padding([8, 14]).width(Length::Fill),
                    button(text("Toggle Files")).style(theme::menu_item_style).on_press(Message::ToggleFilePicker).padding([8, 14]).width(Length::Fill),
                    button(text("Search")).style(theme::menu_item_style).on_press(Message::OpenSearchModal).padding([8, 14]).width(Length::Fill),
                    button(text("Settings")).style(theme::menu_item_style).on_press(Message::OpenSettingsModal).padding([8, 14]).width(Length::Fill),
                    button(text("Help")).style(theme::menu_item_style).on_press(Message::OpenHelpModal).padding([8, 14]).width(Length::Fill),
                    button(text("Quit")).style(theme::menu_item_style).on_press(Message::CloseApp).padding([8, 14]).width(Length::Fill),
                ]
                .width(Length::Fixed(160.0)),
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::BLACK)),
                border: iced::Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_right: 6.0,
                        bottom_left: 6.0,
                    },
                    width: 1.0,
                    color: theme::COLOR_BORDER,
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            });

            column![titlebar_row, menu_dropdown].into()
        } else {
            titlebar_row.into()
        }
    }

    fn view_players(&self) -> Element<'_, Message> {
        if self.players.is_empty() {
            return container(text("No active players").size(18).color(theme::COLOR_TEXT_MUTED))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }

        match self.settings.layout {
            LayoutMode::Row => {
                let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    r = r.push(self.view_single_player(p));
                }
                r.into()
            }
            LayoutMode::Column => {
                let mut c = column![].spacing(0).width(Length::Fill).height(Length::Fill);
                for p in &self.players {
                    c = c.push(self.view_single_player(p));
                }
                c.into()
            }
            LayoutMode::Grid => {
                let count = self.players.len();
                if count == 1 {
                    self.view_single_player(&self.players[0])
                } else if count == 2 {
                    row![
                        self.view_single_player(&self.players[0]),
                        self.view_single_player(&self.players[1]),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else if count == 3 {
                    // Electron 3-player special layout: top player spans full width, bottom row has 2
                    column![
                        container(self.view_single_player(&self.players[0]))
                            .width(Length::Fill)
                            .height(Length::FillPortion(1)),
                        row![
                            self.view_single_player(&self.players[1]),
                            self.view_single_player(&self.players[2]),
                        ]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::FillPortion(1)),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    let cols = if count <= 4 { 2 } else if count <= 9 { 3 } else { 4 };
                    let mut rows = column![].spacing(0).width(Length::Fill).height(Length::Fill);
                    for chunk in self.players.chunks(cols) {
                        let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                        for p in chunk {
                            r = r.push(self.view_single_player(p));
                        }
                        rows = rows.push(r);
                    }
                    rows.into()
                }
            }
        }
    }

    fn view_single_player<'a>(&self, p: &'a VideoHandle) -> Element<'a, Message> {
        let player_id = p.id;
        let is_focused = self.focused_player_id() == Some(player_id);
        let is_hovered = self.hovered_player_id == Some(player_id);

        let pos = p.position();
        let dur = p.duration();
        let time_str = format!(
            "{} / {}",
            format::format_time_str(pos.as_secs_f64()),
            format::format_time_str(dur.as_secs_f64())
        );

        let progress_ratio = if dur.as_secs_f64() > 0.0 {
            (pos.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0) as f32
        } else {
            0.0f32
        };

        let opacity = self.current_opacity();
        let video_widget = p.view(opacity);
        let mut stack_children: Vec<Element<'a, Message>> = vec![video_widget];

        let is_loading = self.loading_player_ids.contains(&player_id);

        if is_loading {
            let angle = (self.spinner_ticks * 12) % 360;
            let spinner_svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" fill="none">
<circle cx="18" cy="18" r="14" stroke="rgba(255,255,255,0.15)" stroke-width="3"/>
<path d="M18 4 A14 14 0 0 1 32 18" stroke="#42b883" stroke-width="3" stroke-linecap="round" transform="rotate({} 18 18)"/>
</svg>"##,
                angle
            );

            let loading_card = container(
                column![
                    svg(svg::Handle::from_memory(spinner_svg.into_bytes()))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0)),
                    text("Loading video...")
                        .size(13)
                        .color(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                ]
                .spacing(12)
                .align_x(Alignment::Center),
            )
            .padding([16, 24])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.08, 0.08, 0.08, 0.88))),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(0.26, 0.72, 0.51, 0.4),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            });

            let loading_layer = container(loading_card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

            stack_children.push(Element::from(loading_layer));
        }

        // Overlays show when mouse is actively moving over this specific player (fades after delay), or while player is loading
        let show_overlay = (is_hovered && self.player_overlay_ticks > 0) || is_loading;

        if show_overlay {
            // 1. Top-Left Title Pill (Matches Electron Player.vue)
            let formatted_title = format::format_descriptive_title(&p.state.path);
            let title_pill = container(
                text(formatted_title)
                    .size(22)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .color(iced::Color::WHITE),
            )
            .padding([10, 18])
            .style(theme::title_pill_style);

            let top_row = row![
                title_pill,
                Space::new().width(Length::Fill),
            ]
            .width(Length::Fill);

            // 2. Bottom Progress & Control Overlay (Vue emerald green theme #42b883)
            let seek_slider = slider(
                0.0..=1.0,
                progress_ratio,
                move |ratio| Message::SeekRatio(player_id, ratio),
            )
            .step(0.001)
            .style(theme::progress_slider_style)
            .width(Length::Fill);

            let progress_bar_with_timestamp = Stack::new()
                .push(seek_slider)
                .push(
                    container(text(time_str).size(13).color(iced::Color::WHITE))
                        .width(Length::Fill)
                        .height(Length::Fixed(22.0))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                );

            let play_pause_icon = if p.state.is_playing {
                svg(svg::Handle::from_memory(SVG_PLAYER_PAUSE))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
            } else {
                svg(svg::Handle::from_memory(SVG_PLAYER_PLAY))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
            };

            let volume_icon = if p.state.is_muted {
                svg(svg::Handle::from_memory(SVG_PLAYER_MUTE))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
            } else {
                svg(svg::Handle::from_memory(SVG_PLAYER_VOLUME))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0))
            };

            let next_icon = svg(svg::Handle::from_memory(SVG_PLAYER_NEXT))
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0));

            let controls_row = row![
                // Left: CC + Volume Icon + Volume Slider
                button(
                    container(
                        text(if self.subtitles_enabled { "CC" } else { "cc" })
                            .size(14)
                    )
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink),
                )
                .style(theme::cc_button_style(self.subtitles_enabled))
                .on_press(Message::ToggleSubtitles)
                .padding([4, 8]),
                button(volume_icon)
                    .style(theme::player_control_button_style)
                    .on_press(Message::TogglePlayerMute(player_id))
                    .padding([4, 6]),
                slider(
                    0.0..=1.0,
                    p.state.volume as f32,
                    move |v| Message::SetVolume(player_id, v as f64),
                )
                .step(0.01)
                .style(theme::volume_slider_style)
                .width(Length::Fixed(80.0)),
                Space::new().width(Length::Fill),
                // Right: Play/Pause + Skip Next
                button(play_pause_icon)
                    .style(theme::player_control_button_style)
                    .on_press(Message::TogglePlay(player_id))
                    .padding([4, 8]),
                button(next_icon)
                    .style(theme::player_control_button_style)
                    .on_press(Message::NextVideo(player_id))
                    .padding([4, 8]),
            ]
            .spacing(12)
            .align_y(Alignment::Center);

            let bottom_overlay = container(
                column![
                    controls_row,
                    progress_bar_with_timestamp,
                ]
                .spacing(8),
            )
            .padding(iced::Padding {
                top: 8.0,
                right: 14.0,
                bottom: 12.0,
                left: 14.0,
            })
            .width(Length::Fill)
            .style(theme::controls_overlay_style);

            let overlays_column = if is_loading {
                column![
                    Space::new().height(Length::Fixed(20.0)),
                    top_row,
                    Space::new().height(Length::Fill),
                ]
            } else {
                column![
                    Space::new().height(Length::Fixed(20.0)),
                    top_row,
                    Space::new().height(Length::Fill),
                    bottom_overlay,
                ]
            }
            .width(Length::Fill)
            .height(Length::Fill);

            stack_children.push(Element::from(overlays_column));
        }

        let show_border = is_focused && self.focus_border_ticks > 0;
        if show_border {
            let focus_ring = container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::focus_ring_style);
            stack_children.push(Element::from(focus_ring));
        }

        let player_stack = Stack::with_children(stack_children)
            .width(Length::Fill)
            .height(Length::Fill);

        let player_box = container(player_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::player_container_style(opacity));

        mouse_area(player_box)
            .on_press(Message::PlayerClicked(player_id))
            .on_enter(Message::PlayerHovered(player_id))
            .on_exit(Message::PlayerUnhovered(player_id))
            .into()
    }

    fn view_file_picker(&self) -> Element<'_, Message> {
        let search_filter = self.file_picker_search.to_lowercase();

        // Group videos by folder
        let mut grouped: BTreeMap<String, Vec<&VideoRecord>> = BTreeMap::new();
        for v in &self.available_videos {
            let title = format::format_video_title(&v.path);
            if !search_filter.is_empty() && !title.to_lowercase().contains(&search_filter) {
                continue;
            }
            let folder = format::format_video_folder(&v.path);
            let folder_key = if folder.is_empty() { "Other".to_string() } else { folder };
            grouped.entry(folder_key).or_default().push(v);
        }

        let mut folders_col = column![].spacing(8);
        for (folder, files) in grouped {
            let count = files.len();
            let header = container(
                row![
                    text(folder).size(13).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    text(format!("({count})")).size(12).color(theme::COLOR_TEXT_MUTED),
                ]
                .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(theme::COLOR_BTN_HOVER)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            let mut files_list = column![].spacing(2);
            for f in files {
                let file_title = format::format_video_title(&f.path);
                let p_clone = f.path.clone();
                files_list = files_list.push(
                    button(text(file_title).size(12).color(iced::Color::WHITE))
                        .style(theme::menu_item_style)
                        .on_press(Message::PlayFileInFocused(p_clone))
                        .padding([4, 8])
                        .width(Length::Fill),
                );
            }

            folders_col = folders_col.push(column![header, files_list].spacing(4));
        }

        let content = column![
            row![
                text("Files").size(18).color(iced::Color::WHITE),
                Space::new().width(Length::Fill),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::ToggleFilePicker),
            ]
            .align_y(Alignment::Center),
            text_input("Search files...", &self.file_picker_search)
                .on_input(Message::FilePickerSearchChanged)
                .style(theme::dark_input_style)
                .padding(8),
            scrollable(folders_col).height(Length::Fill),
        ]
        .spacing(12)
        .padding(16);

        container(content)
            .width(Length::Fixed(320.0))
            .height(Length::Fill)
            .style(theme::file_picker_drawer_style)
            .into()
    }

    fn view_welcome(&self) -> Element<'_, Message> {
        let card = container(
            column![
                text("📁").size(48),
                text("Welcome to Wazoo").size(24).color(iced::Color::WHITE),
                text("Add your media folders in settings to get started.").size(14).color(theme::COLOR_TEXT_MUTED),
                button(text("Open Settings to Add Media Folders"))
                    .style(theme::action_button_style)
                    .on_press(Message::OpenSettingsModal)
                    .padding([10, 20]),
            ]
            .spacing(16)
            .align_x(Alignment::Center),
        )
        .padding(40)
        .style(theme::welcome_card_style);

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_search_modal(&self) -> Element<'_, Message> {
        let mut folder_chips = row![
            button(text("All"))
                .style(theme::folder_chip_style(self.selected_search_folder == "All"))
                .on_press(Message::SelectSearchFolder("All".to_string()))
                .padding([4, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        for folder in &self.settings.media_folders {
            let label = folder.split(['/', '\\']).filter(|s| !s.is_empty()).last().unwrap_or(folder);
            folder_chips = folder_chips.push(
                button(text(label))
                    .style(theme::folder_chip_style(self.selected_search_folder == *folder))
                    .on_press(Message::SelectSearchFolder(folder.clone()))
                    .padding([4, 12]),
            );
        }

        let card = container(
            column![
                row![
                    text("Folder").size(14).color(theme::COLOR_TEXT_MUTED),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseSearchModal),
                ]
                .align_y(Alignment::Center),
                scrollable(folder_chips).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
                row![
                    text_input("Search videos...", &self.search_input)
                        .on_input(Message::SearchInputChanged)
                        .on_submit(Message::PerformSearch)
                        .style(theme::dark_input_style)
                        .padding(10)
                        .width(Length::Fill),
                    button(text("🔍 Search"))
                        .style(theme::action_button_style)
                        .on_press(Message::PerformSearch)
                        .padding([10, 16]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text(format!("Total videos: {}", self.available_videos.len()))
                    .size(12)
                    .color(theme::COLOR_TEXT_MUTED),
            ]
            .spacing(14)
            .width(Length::Fixed(480.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSearchModal)
    }

    fn view_settings_modal(&self) -> Element<'_, Message> {
        let mut folders_col = column![text("Media Folders").size(14).color(theme::COLOR_TEXT_MUTED)].spacing(6);
        for folder in &self.settings.media_folders {
            let f = folder.clone();
            folders_col = folders_col.push(
                container(
                    row![
                        text(folder).size(13).color(iced::Color::WHITE),
                        Space::new().width(Length::Fill),
                        button(text("✕").size(12))
                            .style(theme::close_window_button_style)
                            .on_press(Message::RemoveMediaFolder(f)),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([6, 10])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme::COLOR_CARD_BG)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        let opacity_val = (self.settings.window_opacity * 100.0).round() as u32;

        let scan_btn_text = if self.is_scanning {
            if let Some(ref progress) = self.scan_progress {
                match progress.stage {
                    ScanStage::Listing => {
                        if progress.percent > 0 {
                            format!("Listing... {}% ({} found)", progress.percent, progress.files_found)
                        } else if progress.files_found > 0 {
                            format!("Listing... ({} found)", progress.files_found)
                        } else {
                            "Listing files...".to_string()
                        }
                    }
                    ScanStage::Indexing => {
                        format!("Loading... {}% ({} files)", progress.percent, progress.total)
                    }
                }
            } else {
                "Listing files...".to_string()
            }
        } else {
            "Scan Folders".to_string()
        };

        let scan_btn = if self.is_scanning {
            button(text(scan_btn_text))
                .style(theme::action_button_style)
                .padding([8, 14])
        } else {
            button(text(scan_btn_text))
                .style(theme::action_button_style)
                .on_press(Message::StartScan)
                .padding([8, 14])
        };

        let mut scan_controls = column![
            row![
                button(text("Add Folder"))
                    .style(theme::action_button_style)
                    .on_press(Message::PickFolders)
                    .padding([8, 14]),
                scan_btn,
            ]
            .spacing(10),
        ]
        .spacing(6);

        if self.is_scanning {
            if let Some(ref progress) = self.scan_progress {
                let info_str = match progress.stage {
                    ScanStage::Listing => {
                        if progress.current_name.is_empty() {
                            format!("Discovering files: {}% ({} found)", progress.percent, progress.files_found)
                        } else {
                            format!("Scanning {}: {}% ({} found)", progress.current_name, progress.percent, progress.files_found)
                        }
                    }
                    ScanStage::Indexing => {
                        if progress.current_name.is_empty() {
                            format!("Indexing database: {}% ({} files)", progress.percent, progress.total)
                        } else {
                            format!("Adding {}: {}%", progress.current_name, progress.percent)
                        }
                    }
                };
                scan_controls = scan_controls.push(
                    text(info_str).size(12).color(theme::COLOR_PRIMARY)
                );
            }
        }

        let content = column![
            row![
                text("Settings").size(20).color(iced::Color::WHITE),
                Space::new().width(Length::Fill),
                button(text("✕").size(14))
                    .style(theme::window_control_button_style)
                    .on_press(Message::CloseSettingsModal),
            ]
            .align_y(Alignment::Center),
            column![
                text("Window Opacity").size(14).color(theme::COLOR_TEXT_MUTED),
                row![
                    slider(0.1..=1.0, self.settings.window_opacity, Message::SetWindowOpacity)
                        .step(0.01)
                        .style(theme::volume_slider_style)
                        .width(Length::Fill),
                    text(format!("{opacity_val}%")).size(13).color(theme::COLOR_TEXT_DIM),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(6),
            folders_col,
            scan_controls,
            text(format!("Total Videos: {}", self.available_videos.len()))
                .size(13)
                .color(theme::COLOR_TEXT_MUTED),
        ]
        .spacing(16)
        .width(Length::Fixed(480.0));

        let card = container(scrollable(content))
            .padding(20)
            .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseSettingsModal)
    }

    fn view_help_modal(&self) -> Element<'_, Message> {
        let shortcuts = [
            ("Esc", "Toggle Menu / Close Modal"),
            ("?", "Toggle Help"),
            ("5", "Toggle Scroll Mode"),
            ("6", "Toggle Flip Mode"),
            ("h", "Toggle File Picker"),
            ("n", "Add Player"),
            ("x", "Remove Player"),
            ("Tab", "Focus Next Player"),
            ("l", "Toggle Layout"),
            ("c", "Toggle Subtitles"),
            ("< OR >", "Prev / Next Frame"),
            ("[space]", "Play / Pause"),
            ("↓ ↑", "Prev / Next Video"),
            ("← →", "Seek Back / Forward"),
            ("s", "Toggle Play Mode (Shuffle / Sequential)"),
            ("m", "Toggle Mute"),
            ("[ OR ]", "Adjust Volume"),
            ("j OR /", "Search Videos"),
            ("Alt + X", "Close App"),
            ("Alt + Drag", "Move Window"),
        ];

        let mut shortcuts_list = column![].spacing(8);
        for (key, desc) in shortcuts {
            let key_badge = container(text(key).size(12).color(iced::Color::WHITE))
                .padding([4, 8])
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme::COLOR_BTN_BG)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            shortcuts_list = shortcuts_list.push(
                row![
                    key_badge,
                    Space::new().width(Length::Fill),
                    text(desc).size(13).color(theme::COLOR_TEXT_DIM),
                ]
                .align_y(Alignment::Center),
            );
        }

        let card = container(
            column![
                row![
                    text("Keyboard Shortcuts").size(20).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseHelpModal),
                ]
                .align_y(Alignment::Center),
                scrollable(shortcuts_list).height(Length::Fixed(360.0)),
            ]
            .spacing(14)
            .width(Length::Fixed(460.0)),
        )
        .padding(20)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseHelpModal)
    }

    fn view_menu_modal(&self) -> Element<'_, Message> {
        let card = container(
            column![
                row![
                    text("Menu").size(18).color(iced::Color::WHITE),
                    Space::new().width(Length::Fill),
                    button(text("✕").size(14))
                        .style(theme::window_control_button_style)
                        .on_press(Message::CloseMenuModal),
                ]
                .align_y(Alignment::Center),
                column![
                    button(text("Add Player")).style(theme::menu_item_style).on_press(Message::AddNewPlayer).padding([8, 12]).width(Length::Fill),
                    button(text("Toggle Layout")).style(theme::menu_item_style).on_press(Message::CycleLayout).padding([8, 12]).width(Length::Fill),
                    button(text("Toggle Files")).style(theme::menu_item_style).on_press(Message::ToggleFilePicker).padding([8, 12]).width(Length::Fill),
                    button(text("Search")).style(theme::menu_item_style).on_press(Message::OpenSearchModal).padding([8, 12]).width(Length::Fill),
                    button(text("Settings")).style(theme::menu_item_style).on_press(Message::OpenSettingsModal).padding([8, 12]).width(Length::Fill),
                    button(text("Help")).style(theme::menu_item_style).on_press(Message::OpenHelpModal).padding([8, 12]).width(Length::Fill),
                    button(text("Quit")).style(theme::menu_item_style).on_press(Message::CloseApp).padding([8, 12]).width(Length::Fill),
                ]
                .spacing(4),
            ]
            .spacing(12)
            .width(Length::Fixed(240.0)),
        )
        .padding(16)
        .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseMenuModal)
    }

    fn wrap_modal_with_backdrop<'a>(
        card: container::Container<'a, Message>,
        on_close: Message,
    ) -> Element<'a, Message> {
        let backdrop_top = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_bottom = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_left = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close.clone());

        let backdrop_right = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close);

        let center_row = row![
            backdrop_left,
            card,
            backdrop_right,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Shrink);

        let modal_layout = column![
            backdrop_top,
            center_row,
            backdrop_bottom,
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(modal_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::modal_backdrop_style)
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
        .style(|app: &WazooApp, _theme: &Theme| iced::theme::Style {
            background_color: if app.available_videos.is_empty() {
                Color::from_rgba(0.05, 0.05, 0.05, app.current_opacity())
            } else {
                Color::TRANSPARENT
            },
            text_color: Color::WHITE,
        })
        .window(iced::window::Settings {
            size: iced::Size::new(1280.0, 720.0),
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .run()
}
