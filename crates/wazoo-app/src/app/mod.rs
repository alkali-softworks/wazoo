/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Core Application State
 *
 * Defines the central WazooApp struct, shared timing/layout constants, session history types,
 * Iced application lifecycle hooks, and delegates domain logic across submodules.
 */

mod boot;
mod chrome;
mod navigation;
mod players;
mod plex_explorer;
mod quick_picker;
mod search;

use crate::cli::CliArgs;
use crate::format;
use crate::message::Message;
use crate::state::{AppPlayer, PlayerList};
use iced::{Subscription, Task, Theme};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wazoo_core::{ConfigManager, Database, PlaybackMode, VideoRecord, WazooSettings};
use wazoo_media::{PlayerId, ScrollEngine, VideoHandle};

/// Player controls overlay visibility duration: 2.5 seconds at 60 FPS (150 ticks)
pub const PLAYER_OVERLAY_HIDE_TICKS: usize = 150;
/// Player controls overlay fade out duration: ~200ms at 60 FPS (12 ticks)
pub const PLAYER_OVERLAY_FADE_TICKS: usize = 12;
/// Player controls overlay vertical position ratio (lower-third layout).
/// Portion of flexible space allocated above the controls overlay.
pub const PLAYER_CONTROLS_TOP_PORTION: u16 = 80;
/// Portion of flexible space allocated below the controls overlay.
pub const PLAYER_CONTROLS_BOTTOM_PORTION: u16 = 30;
/// Maximum width in pixels of the compact floating player controls HUD card
pub const PLAYER_CONTROLS_MAX_WIDTH: f32 = 800.0;
/// Window titlebar hide delay: ~800ms at 60 FPS (50 ticks)
pub const TITLEBAR_HIDE_TICKS: usize = 50;
/// Window titlebar fade out duration: ~200ms at 60 FPS (12 ticks)
pub const TITLEBAR_FADE_TICKS: usize = 12;
/// Window titlebar show pre-delay: ~130ms at 60 FPS (8 ticks) to prevent accidental popups on quick swipes
pub const TITLEBAR_SHOW_DELAY_TICKS: usize = 8;
/// Window titlebar slide animation duration: ~180ms at 60 FPS (11 ticks)
pub const TITLEBAR_SLIDE_TICKS: usize = 11;
/// Dropdown menu slide animation duration: ~150ms at 60 FPS (9 ticks)
pub const DROPDOWN_MENU_SLIDE_TICKS: usize = 9;
/// File picker search input debounce delay: ~200ms at 60 FPS (12 ticks)
pub const FILE_PICKER_DEBOUNCE_TICKS: usize = 12;
/// Playback settings (equalizer/speed) slider debounce delay: ~200ms at 60 FPS (12 ticks)
pub const PLAYBACK_SETTINGS_DEBOUNCE_TICKS: usize = 12;
/// Keyboard relative seek debounce delay: ~200ms at 60 FPS (12 ticks)
pub const SEEK_DEBOUNCE_TICKS: usize = 12;
/// Maximum number of videos retained in the session play history drawer
pub const MAX_PLAY_HISTORY_ENTRIES: usize = 1000;
/// Maximum number of navigation entries retained per player
pub const MAX_PLAYER_NAV_HISTORY_ENTRIES: usize = 1000;
/// Focused player border highlight duration: ~333ms at 60 FPS (20 ticks)
pub const FOCUS_BORDER_TICKS: usize = 20;
/// Maximum number of attempts to pick and load an alternative video if the chosen file fails
pub const MAX_VIDEO_LOAD_RETRIES: usize = 3;
/// Default toast notification display duration in seconds
pub const DEFAULT_TOAST_SECS: usize = 2;
/// Extended toast notification display duration in seconds (e.g. for long filenames or status reports)
pub const LONG_TOAST_SECS: usize = 3;
/// Brief toast notification display duration in seconds (e.g. for rapid toggle actions)
pub const SHORT_TOAST_SECS: usize = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackHistoryEntry {
    pub path: String,
    pub position_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayHistoryItem {
    pub path: String,
    pub title: String,
    pub folder: String,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerNavHistory {
    pub back_stack: Vec<PlaybackHistoryEntry>,
    pub forward_stack: Vec<PlaybackHistoryEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Playback,
    Filters,
    Crt,
    Cube,
    Plex,
    Theme,
    System,
}

pub struct WazooApp {
    pub settings: WazooSettings,
    pub theme_palette: crate::theme::ThemePalette,
    pub config_mgr: ConfigManager,
    pub db: Database,
    pub players: PlayerList,
    pub scroll_engine: ScrollEngine,
    pub available_videos: Vec<VideoRecord>,
    pub metadata_cache: std::collections::HashMap<i64, wazoo_core::MediaMetadata>,
    pub search: crate::state::SearchState,
    pub modals: crate::state::ModalState,
    pub drawers: crate::state::DrawerState,
    pub titlebar: crate::state::TitlebarState,
    pub overlay: crate::state::OverlayState,
    pub window: crate::state::WindowState,
    pub app_icon_handle: iced::widget::image::Handle,
    pub raw_static_frames: Vec<crate::assets::RawStaticFrame>,
    pub static_frame: Arc<Mutex<wazoo_media::pipeline::FrameData>>,
    pub last_static_frame_idx: usize,
    pub last_total_videos: usize,
    pub next_player_id: PlayerId,
    pub scanner: crate::state::ScannerState,
    pub plex: crate::state::PlexState,
    pub focused_idx: usize,
    pub hovered_player_id: Option<PlayerId>,
    pub open_audio_menu_id: Option<PlayerId>,
    pub subtitles_enabled: bool,
    pub default_shuffle_mode: bool,
    pub preloaded_player: Option<VideoHandle>,
    pub is_preloading: bool,
    pub standby_player: Arc<Mutex<Option<VideoHandle>>>,
    pub cube: crate::state::CubeState,
}

impl WazooApp {
    pub fn new(cli_args: impl Into<CliArgs>) -> (Self, Task<Message>) {
        let config_mgr = ConfigManager::new();
        let db = Database::open(config_mgr.database_path())
            .expect("Failed to initialize SQLite database");
        Self::new_with_backend(cli_args, config_mgr, db)
    }

    pub fn new_with_backend(
        cli_args: impl Into<CliArgs>,
        config_mgr: ConfigManager,
        db: Database,
    ) -> (Self, Task<Message>) {
        let cli = cli_args.into();
        let mut settings = config_mgr.load_settings();

        // On boot, write the complete keybind settings to the settings file if missing or incomplete
        if !config_mgr.has_complete_keybinds_in_settings() {
            let _ = config_mgr.save_settings(&settings);
        }

        let file_to_play = boot::process_direct_cli_files(&cli, &db, &settings.media_folders);
        let search = boot::resolve_initial_search(&cli, &mut settings, &config_mgr);
        let videos = boot::load_initial_videos(
            &db,
            &search.active_query,
            &search.folders,
            settings.search_include_metadata,
            search.is_cli,
            file_to_play.is_some(),
        );

        let is_cli = search.is_cli;
        let mut app = Self::build_initial_app(config_mgr, db, settings, search, videos);

        let init_task = app.init_initial_players(file_to_play.as_deref(), is_cli);
        app.replenish_standby_player();
        app.apply_file_picker_search();

        if app.settings.playback_mode == PlaybackMode::Flip {
            app.stagger_flip_countdowns();
        }

        let preload_task = if app.settings.playback_mode == PlaybackMode::Scroll {
            app.init_scroll_mode()
        } else {
            Task::none()
        };

        (app, Task::batch([init_task, preload_task, boot::boot_focus_task()]))
    }

    pub fn title(&self) -> String {
        if let Some(title) = self.focused_player().and_then(format_player_window_title) {
            return title;
        }
        for player in &self.players {
            if let Some(title) = format_player_window_title(player) {
                return title;
            }
        }
        "Wazoo".to_string()
    }

    pub fn theme(&self) -> Theme {
        self.theme_palette.to_iced_theme()
    }

    pub fn sync_theme_palette(&mut self) {
        self.settings.theme.clamp_all();
        self.theme_palette = crate::theme::ThemePalette::from_settings(&self.settings.theme);
    }

    #[inline]
    pub fn t<'a>(&'a self, key: &'a str) -> String {
        wazoo_core::t(&self.settings.language, key)
    }

    #[inline]
    pub fn t_with<'a>(&'a self, key: &'a str, args: &[(&str, &str)]) -> String {
        wazoo_core::t_with(&self.settings.language, key, args)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let is_resizing = !self.cube.desktop_overlay && self.window.is_resizing();
        let frame_sub = if self.cube.desktop_overlay {
            iced::window::frames().map(|_| Message::Tick)
        } else {
            let frame_interval = if is_resizing {
                crate::state::WindowState::RESIZE_DEBOUNCE_DURATION
            } else {
                Duration::from_nanos(16_666_667)
            };
            iced::time::every(frame_interval).map(|_| Message::Tick)
        };

        let subs = vec![
            frame_sub,
            iced::time::every(Duration::from_secs(1)).map(|_| Message::WatchdogTick),
            iced::event::listen_with(|event, status, window_id| match event {
                iced::Event::Window(iced::window::Event::Opened { .. }) => {
                    Some(Message::WindowIdReceived(window_id))
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => Some(Message::KeyPressedWithModifiers(key, status, modifiers)),
                iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) => {
                    Some(Message::KeyReleased(key))
                }
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                    Some(Message::CursorMoved(window_id, position))
                }
                iced::Event::Mouse(iced::mouse::Event::CursorLeft) => Some(Message::CursorLeft),
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClickPressed(window_id)),
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Left,
                )) => Some(Message::LeftClickReleased),
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                    iced::mouse::Button::Right,
                )) => Some(Message::RightClickReleased),
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Message::WindowResized(window_id, size))
                }
                iced::Event::Window(iced::window::Event::Moved(point)) => {
                    Some(Message::WindowMoved(window_id, point))
                }
                iced::Event::Window(iced::window::Event::CloseRequested) => Some(Message::CloseApp),
                iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocused),
                iced::Event::Window(iced::window::Event::Unfocused) => {
                    Some(Message::WindowUnfocused)
                }
                iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                _ => None,
            }),
        ];

        Subscription::batch(subs)
    }

    pub fn new_test_app() -> (Self, Task<Message>) {
        let temp_dir = std::env::temp_dir().join(format!(
            "wazoo_test_app_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let config_mgr = ConfigManager::with_dirs(temp_dir.clone(), temp_dir);
        let db = Database::open_in_memory().expect("in-memory db");
        Self::new_with_backend(None, config_mgr, db)
    }
}

pub fn new_test_app() -> (WazooApp, Task<Message>) {
    WazooApp::new_test_app()
}

fn format_player_window_title(player: &AppPlayer) -> Option<String> {
    let title = if !player.state.path.is_empty() {
        format::format_descriptive_title(&player.state.path)
    } else if !player.state.name.is_empty() {
        format::clean_name(&player.state.name)
    } else {
        String::new()
    };
    if title.is_empty() { None } else { Some(title) }
}
