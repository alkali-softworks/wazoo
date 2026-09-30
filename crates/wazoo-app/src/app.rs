/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Core Application State
 *
 * Defines the central WazooApp struct, initial state construction, player lifecycle management,
 * active session persistence, search reconciliation, and event subscription bindings.
 */

use crate::assets::APP_ICON_BYTES;
use crate::cli::CliArgs;
use crate::format;
use crate::message::Message;
use crate::state::{AppPlayer, PlayerList};
use iced::{Point, Subscription, Task, Theme};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wazoo_core::{ConfigManager, Database, PlaybackMode, VideoRecord, VideoSession, WazooSettings};
use wazoo_media::{BufferConfig, PlayerId, ScrollEngine, StartTime, VideoHandle};

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
    System,
}

pub struct WazooApp {
    pub settings: WazooSettings,
    pub config_mgr: ConfigManager,
    pub db: Database,
    pub players: PlayerList,
    pub scroll_engine: ScrollEngine,
    pub available_videos: Vec<VideoRecord>,
    pub search: crate::state::SearchState,
    pub modals: crate::state::ModalState,
    pub drawers: crate::state::DrawerState,
    pub titlebar: crate::state::TitlebarState,
    pub overlay: crate::state::OverlayState,
    pub window: crate::state::WindowState,
    pub app_icon_handle: iced::widget::image::Handle,
    pub last_total_videos: usize,
    pub next_player_id: PlayerId,
    pub scanner: crate::state::ScannerState,
    pub focused_idx: usize,
    pub hovered_player_id: Option<PlayerId>,
    pub open_audio_menu_id: Option<PlayerId>,
    pub subtitles_enabled: bool,
    pub default_shuffle_mode: bool,
    pub preloaded_player: Option<VideoHandle>,
    pub is_preloading: bool,
}

struct InitialSearch {
    folders: Vec<String>,
    active_query: String,
    selected_folder: String,
    is_cli: bool,
}

fn process_direct_cli_files(
    cli: &CliArgs,
    db: &Database,
    media_folders: &[String],
) -> Option<String> {
    let mut file_to_play: Option<String> = None;
    let mut files_to_process = cli.files.clone();
    if files_to_process.is_empty() {
        if let Some(ref single) = cli.file {
            files_to_process.push(single.clone());
        }
    }

    for file_path in &files_to_process {
        let canon_path = std::fs::canonicalize(file_path).unwrap_or_else(|_| file_path.clone());
        let path_str = canon_path.to_string_lossy().to_string();

        // Check if file is inside any configured media folder or in DB under another folder
        let in_other_folder = db
            .is_video_in_other_folder(&path_str, media_folders)
            .unwrap_or(false);

        if !in_other_folder {
            let file_stem = canon_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let clean_name = wazoo_scanner::clean_video_name(file_stem);
            let _ = db.insert_misc_video(&clean_name, &path_str);
        }

        if file_to_play.is_none() {
            file_to_play = Some(path_str);
        }
    }

    file_to_play
}

fn resolve_initial_search(
    cli: &CliArgs,
    settings: &mut WazooSettings,
    config_mgr: &ConfigManager,
) -> InitialSearch {
    let cli_query_clean = cli
        .query
        .as_deref()
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty());
    let is_cli = cli_query_clean.is_some();

    let all_label = wazoo_core::i18n::t(&settings.language, "common.all");
    let (folders, active_query, selected_folder) = if let Some(ref q) = cli_query_clean {
        settings.last_query = q.clone();
        settings.last_folders.clear();
        let _ = config_mgr.save_settings(settings);
        (Vec::new(), q.clone(), all_label)
    } else {
        let f: Vec<String> = settings
            .last_folders
            .iter()
            .filter(|folder| !wazoo_core::i18n::is_all_folder(folder))
            .cloned()
            .collect();
        let sel = if f.is_empty() {
            all_label
        } else if f.len() == 1 {
            f[0].clone()
        } else {
            f.join(", ")
        };
        (f, settings.last_query.clone(), sel)
    };

    InitialSearch {
        folders,
        active_query,
        selected_folder,
        is_cli,
    }
}

fn load_initial_videos(
    db: &Database,
    active_query: &str,
    folders: &[String],
    is_cli: bool,
    has_file_to_play: bool,
) -> Vec<VideoRecord> {
    let mut videos = if !active_query.is_empty() {
        let filtered = db.search_videos(active_query, folders).unwrap_or_default();
        if filtered.is_empty() {
            if is_cli {
                Vec::new()
            } else {
                db.get_all_videos().unwrap_or_default()
            }
        } else {
            filtered
        }
    } else if !folders.is_empty() {
        db.search_videos("", folders).unwrap_or_default()
    } else {
        db.get_all_videos().unwrap_or_default()
    };

    if videos.is_empty() && has_file_to_play {
        videos = db.get_all_videos().unwrap_or_default();
    }

    videos
}

fn build_initial_toast(is_cli: bool, language: &str, total_videos: usize) -> (Option<String>, usize) {
    if !is_cli {
        return (None, 0);
    }

    let msg = if total_videos == 0 {
        wazoo_core::t(language, "wazoo.no_videos_found")
    } else {
        wazoo_core::t_with(
            language,
            "wazoo.total_files",
            &[("total", &format::format_number(total_videos))],
        )
    };

    (Some(msg), LONG_TOAST_SECS)
}

fn boot_focus_task() -> Task<Message> {
    Task::perform(
        async {
            #[cfg(target_os = "linux")]
            {
                crate::platform::ensure_window_focused_linux().await;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        },
        |_| Message::GainWindowFocus,
    )
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

        let file_to_play = process_direct_cli_files(&cli, &db, &settings.media_folders);
        let search = resolve_initial_search(&cli, &mut settings, &config_mgr);
        let videos = load_initial_videos(
            &db,
            &search.active_query,
            &search.folders,
            search.is_cli,
            file_to_play.is_some(),
        );

        let is_cli = search.is_cli;
        let mut app = Self::build_initial_app(config_mgr, db, settings, search, videos);

        app.init_initial_players(file_to_play.as_deref(), is_cli);
        app.apply_file_picker_search();

        if app.settings.playback_mode == PlaybackMode::Flip {
            app.stagger_flip_countdowns();
        }

        let preload_task = if app.settings.playback_mode == PlaybackMode::Scroll {
            app.init_scroll_mode()
        } else {
            Task::none()
        };

        (app, Task::batch([preload_task, boot_focus_task()]))
    }

    fn build_initial_app(
        config_mgr: ConfigManager,
        db: Database,
        settings: WazooSettings,
        search: InitialSearch,
        videos: Vec<VideoRecord>,
    ) -> Self {
        let mut scroll_engine = ScrollEngine::with_window_size(
            settings.window_bounds.width as f32,
            settings.window_bounds.height as f32,
        );
        scroll_engine.set_speed(settings.scroll_speed);
        scroll_engine.scroll_mode_muted = settings.scroll_mode_muted;

        let icon_handle = iced::widget::image::Handle::from_bytes(APP_ICON_BYTES);
        let total_videos = videos.len();
        let (toast_message, toast_time_remaining) =
            build_initial_toast(search.is_cli, &settings.language, total_videos);

        Self {
            settings: settings.clone(),
            config_mgr,
            db,
            players: PlayerList::new(),
            scroll_engine,
            available_videos: videos,
            search: crate::state::SearchState {
                active_query: search.active_query.clone(),
                tags: search
                    .active_query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                input: String::new(),
                folder_input: String::new(),
                active_folders: search.folders.clone(),
                selected_folders: search.folders,
                active_folder: search.selected_folder.clone(),
                selected_folder: search.selected_folder,
            },
            modals: crate::state::ModalState::default(),
            drawers: crate::state::DrawerState::default(),
            titlebar: crate::state::TitlebarState::default(),
            overlay: crate::state::OverlayState {
                toast_message,
                toast_time_remaining,
                ..Default::default()
            },
            window: crate::state::WindowState::default(),
            app_icon_handle: icon_handle,
            last_total_videos: total_videos,
            next_player_id: 1,
            scanner: crate::state::ScannerState::default(),
            focused_idx: 0,
            hovered_player_id: None,
            open_audio_menu_id: None,
            subtitles_enabled: true,
            default_shuffle_mode: true,
            preloaded_player: None,
            is_preloading: false,
        }
    }

    fn init_initial_players(&mut self, file_to_play: Option<&str>, is_cli: bool) {
        let count = self.settings.player_count.clamp(1, 12);
        let restored_sessions = if is_cli || file_to_play.is_some() {
            Vec::new()
        } else {
            self.settings.session_videos.clone()
        };

        let mut direct_file_player_id: Option<PlayerId> = None;

        if let Some(path) = file_to_play {
            let id = self.next_player_id;
            self.next_player_id += 1;
            direct_file_player_id = Some(id);
            let name = format::format_video_title(path);
            if let Ok(mut handle) = self.create_video_handle_with_start(id, path, &name, Some(0.0)) {
                handle.set_muted(false);
                handle.set_volume(1.0);
                handle.set_subtitles_visible(self.subtitles_enabled);
                self.players
                    .push(AppPlayer::new(handle, self.default_shuffle_mode));
                self.push_player_nav_entry(id, path.to_string(), Some(0.0));
            }
            self.overlay.title_pill_ticks = 240;
        } else {
            for session in restored_sessions.into_iter().take(count) {
                if std::path::Path::new(&session.path).exists() {
                    let id = self.next_player_id;
                    self.next_player_id += 1;
                    if self.players.is_empty() {
                        self.default_shuffle_mode = session.is_shuffle;
                    }
                    let name = format::format_video_title(&session.path);
                    let start_secs = if session.position_secs > 0.05 {
                        Some(session.position_secs)
                    } else {
                        None
                    };
                    if let Ok(mut handle) =
                        self.create_video_handle_with_start(id, &session.path, &name, start_secs)
                    {
                        handle.set_muted(session.is_muted);
                        handle.set_volume(session.volume);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.players.push(AppPlayer::new(handle, session.is_shuffle));
                        self.push_player_nav_entry(id, session.path.clone(), start_secs);
                    }
                }
            }
        }

        // Fill remaining players if any
        while self.players.len() < count {
            if self.add_player_internal().is_none() {
                break;
            }
        }

        // If a last_query was active, reconcile active players so any player playing a video
        // not in the queried files list switches to a matching video from available_videos.
        // If playing a direct file, do not reconcile that player!
        if !self.settings.last_query.is_empty() {
            self.reconcile_players_with_available_videos(direct_file_player_id);
        }

        // Save session state if CLI query or direct file successfully populated players
        if (is_cli || file_to_play.is_some()) && !self.players.is_empty() {
            self.save_session_state();
        }
    }

    fn init_scroll_mode(&mut self) -> Task<Message> {
        let items_with_heights: Vec<(PlayerId, f32)> = self
            .players
            .iter()
            .map(|p| (p.id, self.calculate_player_scroll_height(p)))
            .collect();
        self.scroll_engine
            .init_stack_with_heights(&items_with_heights);
        while let Some(spawn_y) = self.scroll_engine.needs_new_player() {
            if let Some(id) = self.add_player_internal() {
                let item_h = self
                    .players
                    .player(id)
                    .map(|p| self.calculate_player_scroll_height(p))
                    .unwrap_or_else(|| self.scroll_engine.default_item_height());
                self.scroll_engine.add_item(id, spawn_y, item_h);
            } else {
                break;
            }
        }
        for p in &mut self.players {
            let vol = self.scroll_engine.calculate_player_volume(p.id);
            p.set_volume(vol);
        }
        self.trigger_preload_task()
    }

    pub(crate) fn is_any_modal_open(&self) -> bool {
        self.modals.is_any_open()
    }

    pub(crate) fn is_modal_or_menu_open(&self) -> bool {
        self.is_any_modal_open() || self.titlebar.show_dropdown_menu
    }

    pub fn is_point_in_titlebar(&self, pos: Point) -> bool {
        let width = if self.settings.window_bounds.width > 0 {
            self.settings.window_bounds.width as f32
        } else {
            f32::MAX
        };
        pos.x >= 0.0 && pos.x <= width && pos.y >= 0.0 && pos.y < 35.0
    }

    pub fn titlebar_alpha(&self) -> f32 {
        if !self.titlebar.show {
            0.0
        } else if self.window.is_dragging
            || self.titlebar.drag_pending
            || self.is_point_in_titlebar(self.window.cursor_position)
            || self.titlebar.show_dropdown_menu
            || self.titlebar.hide_ticks >= TITLEBAR_FADE_TICKS
        {
            1.0
        } else {
            (self.titlebar.hide_ticks as f32 / TITLEBAR_FADE_TICKS as f32).clamp(0.0, 1.0)
        }
    }

    pub fn titlebar_slide_progress(&self) -> f32 {
        if !self.titlebar.show {
            0.0
        } else if self.window.is_dragging
            || self.titlebar.drag_pending
            || self.titlebar.show_dropdown_menu
        {
            1.0
        } else if self.titlebar.hide_ticks < TITLEBAR_FADE_TICKS {
            (self.titlebar.hide_ticks as f32 / TITLEBAR_FADE_TICKS as f32).clamp(0.0, 1.0)
        } else if self.titlebar.slide_ticks >= TITLEBAR_SLIDE_TICKS {
            1.0
        } else {
            (self.titlebar.slide_ticks as f32 / TITLEBAR_SLIDE_TICKS as f32).clamp(0.0, 1.0)
        }
    }

    pub fn dropdown_menu_slide_progress(&self) -> f32 {
        if !self.titlebar.show_dropdown_menu {
            0.0
        } else if self.titlebar.dropdown_menu_slide_ticks >= DROPDOWN_MENU_SLIDE_TICKS {
            1.0
        } else {
            (self.titlebar.dropdown_menu_slide_ticks as f32 / DROPDOWN_MENU_SLIDE_TICKS as f32)
                .clamp(0.0, 1.0)
        }
    }

    pub fn trigger_player_overlay(&mut self) {
        self.overlay.trigger();
    }

    pub fn player_overlay_alpha(&self) -> f32 {
        self.overlay.alpha()
    }

    pub fn should_hide_cursor(&self) -> bool {
        !self.players.is_empty()
            && self.overlay.ticks == 0
            && self.open_audio_menu_id.is_none()
            && self.players.iter().all(|p| !p.is_loading)
            && !self.titlebar.show
            && !self.is_modal_or_menu_open()
    }

    pub fn focused_player_id(&self) -> Option<PlayerId> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(self.players[idx].id)
        }
    }

    pub(crate) fn focused_player(&self) -> Option<&AppPlayer> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(&self.players[idx])
        }
    }

    pub(crate) fn focused_player_mut(&mut self) -> Option<&mut AppPlayer> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(&mut self.players[idx])
        }
    }

    pub(crate) fn load_transcript_for_focused_player(&mut self) -> Task<Message> {
        let (path, sub_tracks, selected_track_pos) = if let Some(player) = self.focused_player() {
            let path = player.state.path.clone();
            let sub_tracks = player.subtitle_tracks().to_vec();
            let selected_pos = sub_tracks.iter().position(|t| t.is_selected).unwrap_or(0);
            (path, sub_tracks, selected_pos)
        } else {
            self.drawers.transcript_cues.clear();
            self.drawers.transcript_video_path = None;
            self.drawers.transcript_loading = false;
            self.drawers.transcript_track_index = 0;
            self.drawers.show_transcript_menu = false;
            return Task::none();
        };

        if path.is_empty() {
            self.drawers.transcript_cues.clear();
            self.drawers.transcript_video_path = None;
            self.drawers.transcript_loading = false;
            self.drawers.transcript_track_index = 0;
            self.drawers.show_transcript_menu = false;
            return Task::none();
        }

        // Sync track index with selected player subtitle track if video path changed
        if self.drawers.transcript_video_path.as_deref() != Some(&path) {
            self.drawers.transcript_track_index = selected_track_pos;
            self.drawers.show_transcript_menu = false;
        } else if !self.drawers.transcript_cues.is_empty() {
            return Task::none();
        }

        self.drawers.transcript_video_path = Some(path.clone());
        self.drawers.transcript_loading = true;
        self.drawers.transcript_cues.clear();
        let path_clone = path.clone();
        let track_idx = self.drawers.transcript_track_index;
        let sub_track = sub_tracks.get(track_idx).cloned();
        let ext_file = sub_track.as_ref().and_then(|t| t.external_filename.clone());
        let ff_index = sub_track.as_ref().and_then(|t| t.ff_index);

        Task::perform(
            async move {
                wazoo_media::load_subtitles_for_track_details(path, ext_file, ff_index, track_idx)
                    .await
            },
            move |cues| Message::TranscriptLoaded(path_clone, track_idx, cues),
        )
    }

    pub fn is_player_shuffle(&self, id: PlayerId) -> bool {
        self.players
            .player(id)
            .map(|p| p.shuffle)
            .unwrap_or(self.default_shuffle_mode)
    }

    pub(crate) fn get_next_video_rec_with_mode(
        &self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if self.available_videos.is_empty() {
            return None;
        }
        if is_shuffle {
            if self.available_videos.len() > 1 {
                if let Some(curr) = current_path {
                    let candidates: Vec<&VideoRecord> = self
                        .available_videos
                        .iter()
                        .filter(|v| v.path != curr)
                        .collect();
                    if !candidates.is_empty() {
                        let idx = rand::random::<usize>() % candidates.len();
                        return Some(candidates[idx].clone());
                    }
                }
            }
            let idx = rand::random::<usize>() % self.available_videos.len();
            Some(self.available_videos[idx].clone())
        } else {
            if let Some(curr) = current_path {
                if let Some(pos) = self.available_videos.iter().position(|v| v.path == curr) {
                    // If we reach the end of the list in sequential mode, start over from the beginning
                    let next_pos = if pos + 1 >= self.available_videos.len() {
                        0
                    } else {
                        pos + 1
                    };
                    return Some(self.available_videos[next_pos].clone());
                }
            }
            // If current video is not found or not provided, start from beginning of list
            Some(self.available_videos[0].clone())
        }
    }

    pub fn get_next_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
        self.get_next_video_rec_with_mode(current_path, self.default_shuffle_mode)
    }

    pub fn get_prev_video_rec_with_mode(
        &self,
        current_path: Option<&str>,
        is_shuffle: bool,
    ) -> Option<VideoRecord> {
        if self.available_videos.is_empty() {
            return None;
        }
        if is_shuffle {
            if self.available_videos.len() > 1 {
                if let Some(curr) = current_path {
                    let candidates: Vec<&VideoRecord> = self
                        .available_videos
                        .iter()
                        .filter(|v| v.path != curr)
                        .collect();
                    if !candidates.is_empty() {
                        let idx = rand::random::<usize>() % candidates.len();
                        return Some(candidates[idx].clone());
                    }
                }
            }
            let idx = rand::random::<usize>() % self.available_videos.len();
            Some(self.available_videos[idx].clone())
        } else {
            if let Some(curr) = current_path {
                if let Some(pos) = self.available_videos.iter().position(|v| v.path == curr) {
                    let prev_pos = if pos == 0 {
                        self.available_videos.len().saturating_sub(1)
                    } else {
                        pos - 1
                    };
                    return Some(self.available_videos[prev_pos].clone());
                }
            }
            self.available_videos.last().cloned()
        }
    }

    pub(crate) fn record_current_player_nav_position(&mut self, id: PlayerId) {
        if let Some(p) = self.players.player_mut(id) {
            let pos_secs = p.position().as_secs_f64();
            let dur_secs = p.duration().as_secs_f64();
            let saved_pos = if dur_secs > 10.0 && (dur_secs - pos_secs) < 3.0 {
                None
            } else if pos_secs > 0.5 {
                Some(pos_secs)
            } else {
                None
            };
            let curr_path = p.state.path.clone();
            if let Some(top) = p.nav_history.back_stack.last_mut() {
                if top.path == curr_path {
                    top.position_secs = saved_pos;
                }
            }
        }
    }

    pub fn record_play_history(&mut self, path: &str) {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.drawers.play_history.last().map(|e| e.path.as_str()) == Some(trimmed) {
            return;
        }
        let title = format::format_video_title(trimmed);
        let folder = format::format_video_folder(trimmed);
        self.drawers.play_history.push(PlayHistoryItem {
            path: trimmed.to_string(),
            title,
            folder,
        });
        if self.drawers.play_history.len() > MAX_PLAY_HISTORY_ENTRIES {
            self.drawers.play_history.remove(0);
        }
    }

    pub fn push_player_nav_entry(&mut self, id: PlayerId, path: String, pos: Option<f64>) {
        self.record_play_history(&path);
        if let Some(player) = self.players.player_mut(id) {
            let hist = &mut player.nav_history;
            if hist.back_stack.last().map(|e| &e.path) != Some(&path) {
                hist.back_stack.push(PlaybackHistoryEntry {
                    path,
                    position_secs: pos,
                });
                if hist.back_stack.len() > MAX_PLAYER_NAV_HISTORY_ENTRIES {
                    hist.back_stack.remove(0);
                }
            }
        }
    }

    pub(crate) fn close_file_picker(&mut self) {
        self.drawers.show_file_picker = false;
        self.drawers.file_picker_entries.clear();
        self.drawers.file_picker_entries.shrink_to_fit();
        self.drawers.file_picker_groups.clear();
        self.drawers.file_picker_groups.shrink_to_fit();
    }

    pub(crate) fn buffer_config(&self) -> BufferConfig {
        let player_count = self.players.len().max(1) as u32;
        let per_player_mb = (self.settings.buffer_size_mb / player_count).clamp(8, 24);
        BufferConfig {
            duration_secs: self.settings.buffer_duration_secs.clamp(2, 5),
            size_mb: per_player_mb,
            read_chunk_kb: 512,
            preferred_audio_language: self.settings.preferred_audio_language.clone(),
            gamma: self.settings.gamma as f64,
            contrast: self.settings.contrast as f64,
            brightness: self.settings.brightness as f64,
            saturation: self.settings.saturation as f64,
            playback_speed: self.settings.playback_speed as f64,
            crt_enabled: self.settings.crt_enabled,
        }
    }

    pub(crate) fn create_video_handle(
        &self,
        id: PlayerId,
        path: &str,
        name: &str,
    ) -> Result<VideoHandle, String> {
        self.create_video_handle_with_start_time(id, path, name, StartTime::Beginning)
    }

    pub(crate) fn create_video_handle_with_start(
        &self,
        id: PlayerId,
        path: &str,
        name: &str,
        start_secs: Option<f64>,
    ) -> Result<VideoHandle, String> {
        self.create_video_handle_with_start_time(id, path, name, start_secs)
    }

    pub(crate) fn create_video_handle_with_start_time(
        &self,
        id: PlayerId,
        path: &str,
        name: &str,
        start_time: impl Into<StartTime>,
    ) -> Result<VideoHandle, String> {
        VideoHandle::with_buffering_and_start(id, path, name, self.buffer_config(), start_time)
    }

    pub fn current_opacity(&self) -> f32 {
        if self.window.is_alt_pressed {
            0.85
        } else {
            self.settings.window_opacity.clamp(0.05, 1.0)
        }
    }

    pub(crate) fn add_player_internal(&mut self) -> Option<PlayerId> {
        let id = self.next_player_id;
        self.next_player_id += 1;

        // In scroll mode, mute follows the scroll feed's global mute setting (muted by default).
        // In ambient grid/normal mode, the first player starts unmuted so the user hears audio
        // when opening the app normally with no prior user preference, while subsequent background players start muted.
        let initial_muted = if self.settings.playback_mode == PlaybackMode::Scroll {
            self.settings.scroll_mode_muted
        } else if self.players.is_empty() {
            false
        } else {
            true
        };
        let start_time = if self.settings.playback_mode == PlaybackMode::Scroll
            || self.settings.playback_mode == PlaybackMode::Flip
        {
            StartTime::Random
        } else {
            StartTime::Beginning
        };

        for _ in 0..MAX_VIDEO_LOAD_RETRIES {
            if let Some(video_rec) = self.get_next_video_rec(None) {
                match self.create_video_handle_with_start_time(
                    id,
                    &video_rec.path,
                    &video_rec.name,
                    start_time,
                ) {
                    Ok(mut handle) => {
                        handle.set_muted(initial_muted);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        let mut player = AppPlayer::new(handle, self.default_shuffle_mode);
                        player.flip.reset(self.settings.flip_interval_secs);
                        player.start_loading();
                        self.push_player_nav_entry(id, video_rec.path.clone(), None);
                        self.players.push(player);
                        return Some(id);
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to create VideoHandle for {}: {}",
                            video_rec.path,
                            err
                        );
                    }
                }
            }
        }
        None
    }

    /// Resets and staggers the flip countdowns across all active players so they do not all cycle at the same time.
    pub fn stagger_flip_countdowns(&mut self) {
        let n = self.players.len();
        if n == 0 {
            return;
        }
        let interval = self.settings.flip_interval_secs.max(1);
        for (i, p) in self.players.iter_mut().enumerate() {
            let staggered = if n > 1 {
                let frac = (i + 1) as f64 / n as f64;
                ((frac * interval as f64).round() as u64).clamp(1, interval)
            } else {
                interval
            };
            p.flip.reset(staggered);
        }
    }

    /// Persists the exact current playback session (file, timestamp, mute, volume, shuffle) to settings
    pub fn save_session_state(&mut self) {
        if !self.players.is_empty() {
            let sessions: Vec<VideoSession> = self
                .players
                .iter()
                .map(|p| VideoSession {
                    path: p.state.path.clone(),
                    position_secs: p.position().as_secs_f64(),
                    is_muted: p.state.is_muted,
                    volume: p.state.volume,
                    is_shuffle: p.shuffle,
                })
                .collect();
            self.settings.session_videos = sessions;
        }
        self.settings.last_query = self.search.active_query.clone();
        self.settings.last_folders = self.search.active_folders.clone();
        let _ = self.config_mgr.save_settings(&self.settings);
        self.window.bounds_dirty = false;
    }

    /// Reconciles active players against `self.available_videos`.
    /// Any player currently playing a file that does NOT exist in `self.available_videos`
    /// is switched to play a video that DOES exist in `self.available_videos`.
    /// Each switching player is assigned a distinct video when possible.
    pub fn reconcile_players_with_available_videos(&mut self, skip_player_id: Option<PlayerId>) {
        if self.available_videos.is_empty() {
            return;
        }

        if self.players.is_empty() {
            let target_count = self.settings.player_count.clamp(1, 12);
            for _ in 0..target_count {
                self.add_player_internal();
            }
            return;
        }

        let needs_switch: Vec<(PlayerId, bool, f64)> = self
            .players
            .iter()
            .filter(|p| skip_player_id != Some(p.id))
            .filter(|p| !self.available_videos.iter().any(|v| v.path == p.state.path))
            .map(|p| (p.id, p.state.is_muted, p.state.volume))
            .collect();

        if needs_switch.is_empty() {
            return;
        }

        // Track paths already being played by players that are NOT switching
        let mut used_paths: HashSet<String> =
            self.players.iter().map(|p| p.state.path.clone()).collect();
        for &(id, _, _) in &needs_switch {
            if let Some(p) = self.players.player(id) {
                used_paths.remove(&p.state.path);
            }
        }

        for (idx, (id, prev_muted, prev_volume)) in needs_switch.into_iter().enumerate() {
            let unused: Vec<&VideoRecord> = self
                .available_videos
                .iter()
                .filter(|v| !used_paths.contains(&v.path))
                .collect();

            let is_shuffle = self.is_player_shuffle(id);
            let candidate = if !unused.is_empty() {
                if is_shuffle {
                    let r = rand::random::<usize>() % unused.len();
                    Some(unused[r])
                } else {
                    Some(unused[0])
                }
            } else {
                let fallback_idx = if is_shuffle {
                    rand::random::<usize>() % self.available_videos.len()
                } else {
                    idx % self.available_videos.len()
                };
                self.available_videos.get(fallback_idx)
            };

            if let Some(video_rec) = candidate {
                let rec_path = video_rec.path.clone();
                let rec_name = video_rec.name.clone();
                used_paths.insert(rec_path.clone());

                if let Some(p) = self.players.player_mut(id) {
                    p.start_loading();
                }

                if let Ok(mut new_handle) = self.create_video_handle(id, &rec_path, &rec_name) {
                    new_handle.set_muted(prev_muted);
                    new_handle.set_volume(prev_volume);
                    new_handle.set_subtitles_visible(self.subtitles_enabled);
                    self.push_player_nav_entry(id, rec_path.clone(), None);
                    if let Some(p) = self.players.player_mut(id) {
                        p.handle = new_handle;
                    }
                }
            }
        }

        if self.settings.playback_mode == PlaybackMode::Scroll {
            if let Some(ref preloaded) = self.preloaded_player {
                if !self
                    .available_videos
                    .iter()
                    .any(|v| v.path == preloaded.state.path)
                {
                    if let Some(handle) = self.preloaded_player.take() {
                        std::thread::spawn(move || drop(handle));
                    }
                    self.is_preloading = false;
                }
            }
        } else if self.preloaded_player.is_some() {
            if let Some(handle) = self.preloaded_player.take() {
                std::thread::spawn(move || drop(handle));
            }
            self.is_preloading = false;
        }
    }

    /// Triggers an asynchronous background preload task for the next video in scroll mode
    pub(crate) fn trigger_preload_task(&mut self) -> Task<Message> {
        if self.preloaded_player.is_some()
            || self.is_preloading
            || self.settings.playback_mode != PlaybackMode::Scroll
        {
            return Task::none();
        }

        let curr_path = self.players.last().map(|p| p.state.path.clone());
        let video_rec = match self.get_next_video_rec(curr_path.as_deref()) {
            Some(rec) => rec,
            None => return Task::none(),
        };

        let id = self.next_player_id;
        self.next_player_id += 1;
        self.is_preloading = true;

        let buffer_config = self.buffer_config();
        let path = video_rec.path;
        let name = video_rec.name;

        let holder = Arc::new(Mutex::new(None));
        let holder_clone = Arc::clone(&holder);

        Task::perform(
            async move {
                let res = tokio::task::spawn_blocking(move || {
                    let mut handle = VideoHandle::with_buffering_and_start(
                        id,
                        &path,
                        &name,
                        buffer_config,
                        StartTime::Random,
                    )?;
                    handle.set_muted(true);
                    handle.set_paused(true);

                    // Background pre-buffer: decode initial presentation frame off-thread
                    // so the player is 100% ready when attached to the scroll feed without hitching.
                    let start = std::time::Instant::now();
                    while !handle.has_decoded_frame()
                        && start.elapsed() < Duration::from_millis(1500)
                    {
                        if handle.update_frame() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }

                    Ok::<VideoHandle, String>(handle)
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r);

                *holder_clone.lock().unwrap() = Some(res);
                holder_clone
            },
            Message::PreloadedPlayerReady,
        )
    }

    /// Calculates the real pixel height for a player in scroll mode based on its native aspect ratio
    /// and the current stream (window) width.
    pub(crate) fn calculate_player_scroll_height(&self, player: &VideoHandle) -> f32 {
        if let Some(ar) = player.aspect_ratio() {
            self.scroll_engine.item_height_for_aspect_ratio(ar)
        } else {
            self.scroll_engine.default_item_height()
        }
    }

    pub fn title(&self) -> String {
        if let Some(player) = self.focused_player() {
            let title = if !player.state.path.is_empty() {
                format::format_descriptive_title(&player.state.path)
            } else if !player.state.name.is_empty() {
                format::clean_name(&player.state.name)
            } else {
                String::new()
            };

            if !title.is_empty() {
                return title;
            }
        }

        for player in &self.players {
            let title = if !player.state.path.is_empty() {
                format::format_descriptive_title(&player.state.path)
            } else if !player.state.name.is_empty() {
                format::clean_name(&player.state.name)
            } else {
                String::new()
            };

            if !title.is_empty() {
                return title;
            }
        }

        "Wazoo".to_string()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    #[inline]
    pub fn t<'a>(&'a self, key: &'a str) -> String {
        wazoo_core::t(&self.settings.language, key)
    }

    #[inline]
    pub fn t_with<'a>(&'a self, key: &'a str, args: &[(&str, &str)]) -> String {
        wazoo_core::t_with(&self.settings.language, key, args)
    }

    pub(crate) fn total_video_count(&self) -> usize {
        self.db
            .get_video_count()
            .unwrap_or(self.available_videos.len())
    }

    /// Resets scroll engine and transitions playback mode back to Normal
    pub(crate) fn cleanup_scroll_mode(&mut self) {
        self.settings.playback_mode = PlaybackMode::Normal;
        self.scroll_engine.clear();
        if let Some(handle) = self.preloaded_player.take() {
            std::thread::spawn(move || drop(handle));
        }
        self.is_preloading = false;
        if self.settings.scroll_mode_muted {
            for p in &mut self.players {
                p.set_muted(true);
            }
        } else {
            for (i, p) in self.players.iter_mut().enumerate() {
                if i == self.focused_idx {
                    p.set_muted(false);
                    p.set_volume(1.0);
                } else {
                    p.set_muted(true);
                }
            }
        }
    }

    pub fn is_all_folder(&self, folder: &str) -> bool {
        wazoo_core::i18n::is_all_folder(folder)
    }

    pub fn is_all_search_selected(&self) -> bool {
        self.search.selected_folders.is_empty()
            || (self.search.selected_folders.len() == 1
                && self.is_all_folder(&self.search.selected_folders[0]))
    }

    pub(crate) fn reset_search_folder_selection(&mut self) {
        self.search.selected_folders.clear();
        self.search.selected_folder = self.t("common.all");
    }

    pub(crate) fn toggle_search_folder(&mut self, folder: &str) {
        if self.is_all_folder(folder) {
            self.reset_search_folder_selection();
            return;
        }

        if let Some(pos) = self
            .search
            .selected_folders
            .iter()
            .position(|f| f == folder)
        {
            self.search.selected_folders.remove(pos);
        } else {
            self.search.selected_folders.push(folder.to_string());
        }

        if self.search.selected_folders.is_empty() {
            self.search.selected_folder = self.t("common.all");
        } else if self.search.selected_folders.len() == 1 {
            self.search.selected_folder = self.search.selected_folders[0].clone();
        } else {
            self.search.selected_folder = self.search.selected_folders.join(", ");
        }
    }

    pub(crate) fn show_video_totals_notice(&mut self, total: usize, folder_label: &str) {
        if total == 0 {
            let msg = if self.is_all_folder(folder_label) {
                self.t("wazoo.no_videos_found")
            } else {
                let folder_clean = format::folder_basename(folder_label);
                self.t_with("wazoo.no_files_found_in", &[("folder", folder_clean)])
            };
            self.overlay.toast_message = Some(msg);
            self.overlay.toast_time_remaining = LONG_TOAST_SECS;
            return;
        }

        let total_str = format::format_number(total);
        let base_msg = self.t_with("wazoo.total_files", &[("total", &total_str)]);

        let msg = if self.last_total_videos != 0 && total > self.last_total_videos {
            let diff = total - self.last_total_videos;
            let diff_str = format::format_number(diff);
            let diff_msg = self.t_with("wazoo.more_than_before", &[("diff", &diff_str)]);
            format!("{base_msg}\n{diff_msg}")
        } else if self.last_total_videos != 0 && total < self.last_total_videos {
            let diff = self.last_total_videos - total;
            let diff_str = format::format_number(diff);
            let diff_msg = self.t_with("wazoo.less_than_before", &[("diff", &diff_str)]);
            format!("{base_msg}\n{diff_msg}")
        } else {
            base_msg
        };

        self.last_total_videos = total;
        self.overlay.toast_message = Some(msg);
        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            iced::time::every(Duration::from_millis(16)).map(|_| Message::VideoFrameTick),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::WatchdogTick),
            iced::event::listen_with(|event, status, window_id| match event {
                iced::Event::Window(iced::window::Event::Opened { .. }) => {
                    Some(Message::WindowIdReceived(window_id))
                }
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                    Some(Message::KeyPressed(key, status))
                }
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

        if self.settings.playback_mode == PlaybackMode::Scroll {
            subs.push(iced::time::every(Duration::from_millis(16)).map(|_| Message::AnimationTick));
        }

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
