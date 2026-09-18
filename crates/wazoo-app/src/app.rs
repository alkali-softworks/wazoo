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
use iced::{Point, Subscription, Task, Theme};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wazoo_core::{ConfigManager, Database, PlaybackMode, VideoRecord, VideoSession, WazooSettings};
use wazoo_media::{BufferConfig, PlayerId, ScrollEngine, StartTime, VideoHandle};
use wazoo_scanner::ScanProgress;

/// Player controls overlay visibility duration: 2.5 seconds at 60 FPS (150 ticks)
pub const PLAYER_OVERLAY_HIDE_TICKS: usize = 150;
/// Player controls overlay fade out duration: ~200ms at 60 FPS (12 ticks)
pub const PLAYER_OVERLAY_FADE_TICKS: usize = 12;
/// Window titlebar hide delay: ~400ms at 60 FPS (25 ticks)
pub const TITLEBAR_HIDE_TICKS: usize = 50;
/// Window titlebar fade out duration: ~200ms at 60 FPS (12 ticks)
pub const TITLEBAR_FADE_TICKS: usize = 12;
/// Window titlebar show pre-delay: ~130ms at 60 FPS (8 ticks) to prevent accidental popups on quick swipes
pub const TITLEBAR_SHOW_DELAY_TICKS: usize = 8;
/// File picker search input debounce delay: ~200ms at 60 FPS (12 ticks)
pub const FILE_PICKER_DEBOUNCE_TICKS: usize = 12;
/// Maximum number of videos retained in the session play history drawer
pub const MAX_PLAY_HISTORY_ENTRIES: usize = 1000;

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

pub struct WazooApp {
    pub settings: WazooSettings,
    pub config_mgr: ConfigManager,
    pub db: Database,
    pub players: Vec<VideoHandle>,
    pub scroll_engine: ScrollEngine,
    pub available_videos: Vec<VideoRecord>,
    pub active_search_query: String,
    pub search_input: String,
    pub search_tags: Vec<String>,
    pub folder_input: String,
    pub active_search_folders: Vec<String>,
    pub selected_search_folders: Vec<String>,
    pub active_search_folder: String,
    pub selected_search_folder: String,
    pub show_search_modal: bool,
    pub show_settings_modal: bool,
    pub show_help_modal: bool,
    pub show_menu_modal: bool,
    pub show_bookmarks_modal: bool,
    pub show_file_picker: bool,
    pub file_picker_search: String,
    pub(crate) file_picker_debounce_ticks: usize,
    pub(crate) file_picker_entries: Vec<crate::views::file_picker::PrecomputedVideoMeta>,
    pub(crate) file_picker_groups: Vec<crate::views::file_picker::FilePickerGroup>,
    pub show_history_drawer: bool,
    pub play_history: Vec<PlayHistoryItem>,
    pub history_search: String,
    pub show_titlebar: bool,
    pub titlebar_hide_ticks: usize,
    pub titlebar_hover_ticks: usize,
    pub show_dropdown_menu: bool,
    pub is_alt_pressed: bool,
    pub player_overlay_ticks: usize,
    pub title_pill_ticks: usize,
    pub window_id: Option<iced::window::Id>,
    pub app_icon_handle: iced::widget::image::Handle,
    pub toast_message: Option<String>,
    pub toast_time_remaining: usize,
    pub last_total_videos: usize,
    pub next_player_id: PlayerId,
    pub is_scanning: bool,
    pub current_scan_id: u64,
    pub scan_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub scan_progress: Option<ScanProgress>,
    pub focused_player_idx: usize,
    pub focus_border_ticks: usize,
    pub is_shuffle_mode: bool,
    pub hovered_player_id: Option<PlayerId>,
    pub open_audio_menu_player_id: Option<PlayerId>,
    pub subtitles_enabled: bool,
    pub loading_player_ids: HashSet<PlayerId>,
    pub loading_player_ticks: HashMap<PlayerId, usize>,
    pub spinner_ticks: u32,
    pub preloaded_player: Option<VideoHandle>,
    pub is_preloading: bool,
    pub cursor_position: Point,
    pub titlebar_press_origin: Option<Point>,
    pub titlebar_drag_pending: bool,
    pub is_window_dragging: bool,
    pub last_window_drag_move: Option<Instant>,
    pub last_titlebar_click: Option<Instant>,
    pub expanded_folders: HashSet<String>,
    pub show_transcript: bool,
    pub transcript_cues: Vec<wazoo_media::SubtitleCue>,
    pub transcript_search: String,
    pub transcript_loading: bool,
    pub transcript_video_path: Option<String>,
    pub transcript_track_index: usize,
    pub show_transcript_menu: bool,
    pub is_window_focused: bool,
    pub unfocused_frame_ticks: u32,
    pub window_bounds_dirty: bool,
    pub player_nav_history: HashMap<PlayerId, PlayerNavHistory>,
    pub player_shuffle_modes: HashMap<PlayerId, bool>,
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
        let has_complete_keybinds = config_mgr.has_complete_keybinds_in_settings();
        let mut settings = config_mgr.load_settings();

        // On boot, write the complete keybind settings to the settings file if missing or incomplete
        if !has_complete_keybinds {
            let _ = config_mgr.save_settings(&settings);
        }

        // 1. Direct file playback: check if a video file path was provided
        let file_to_play: Option<String> = if let Some(ref file_path) = cli.file {
            let canon_path = std::fs::canonicalize(file_path).unwrap_or_else(|_| file_path.clone());
            let path_str = canon_path.to_string_lossy().to_string();

            // Check if file is inside any configured media folder or in DB under another folder
            let in_other_folder = db
                .is_video_in_other_folder(&path_str, &settings.media_folders)
                .unwrap_or(false);

            if !in_other_folder {
                let file_stem = canon_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                let clean_name = wazoo_scanner::clean_video_name(file_stem);
                let _ = db.insert_misc_video(&clean_name, &path_str);
            }

            Some(path_str)
        } else {
            None
        };

        let cli_query_clean = cli
            .query
            .map(|q| q.trim().to_string())
            .filter(|q| !q.is_empty());
        let is_cli = cli_query_clean.is_some();

        let all_label = wazoo_core::i18n::t(&settings.language, "common.all");
        let (folders, active_query, selected_folder) = if let Some(ref q) = cli_query_clean {
            settings.last_query = q.clone();
            settings.last_folders.clear();
            let _ = config_mgr.save_settings(&settings);
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

        let mut videos: Vec<VideoRecord> = if !active_query.is_empty() {
            let filtered = db
                .search_videos(&active_query, &folders)
                .unwrap_or_default();
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
            db.search_videos("", &folders).unwrap_or_default()
        } else {
            db.get_all_videos().unwrap_or_default()
        };

        // If available_videos is empty and we have a file_to_play, load all videos (which includes the new file)
        if videos.is_empty() && file_to_play.is_some() {
            videos = db.get_all_videos().unwrap_or_default();
        }
        let mut scroll_engine = ScrollEngine::with_window_size(
            settings.window_bounds.width as f32,
            settings.window_bounds.height as f32,
        );
        scroll_engine.set_speed(settings.scroll_speed);
        scroll_engine.is_global_muted = settings.is_global_muted;

        let icon_handle = iced::widget::image::Handle::from_bytes(APP_ICON_BYTES);

        let total_videos = videos.len();
        let toast_msg = if is_cli {
            if videos.is_empty() {
                Some(wazoo_core::t(&settings.language, "wazoo.no_videos_found"))
            } else {
                Some(wazoo_core::t_with(
                    &settings.language,
                    "wazoo.total_files",
                    &[("total", &format::format_number(total_videos))],
                ))
            }
        } else {
            None
        };
        let toast_time_remaining = if toast_msg.is_some() { 3 } else { 0 };

        let mut app = Self {
            settings: settings.clone(),
            config_mgr,
            db,
            players: Vec::new(),
            scroll_engine,
            available_videos: videos,
            active_search_query: active_query.clone(),
            search_tags: active_query
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            search_input: String::new(),
            folder_input: String::new(),
            active_search_folders: folders.clone(),
            selected_search_folders: folders,
            active_search_folder: selected_folder.clone(),
            selected_search_folder: selected_folder,
            show_search_modal: false,
            show_settings_modal: false,
            show_help_modal: false,
            show_menu_modal: false,
            show_bookmarks_modal: false,
            show_file_picker: false,
            file_picker_search: String::new(),
            file_picker_debounce_ticks: 0,
            file_picker_entries: Vec::new(),
            file_picker_groups: Vec::new(),
            show_titlebar: false,
            titlebar_hide_ticks: 0,
            titlebar_hover_ticks: 0,
            show_dropdown_menu: false,
            is_alt_pressed: false,
            player_overlay_ticks: 0,
            title_pill_ticks: 0,
            window_id: None,
            app_icon_handle: icon_handle,
            toast_message: toast_msg,
            toast_time_remaining,
            last_total_videos: total_videos,
            next_player_id: 1,
            is_scanning: false,
            current_scan_id: 0,
            scan_cancel: None,
            scan_progress: None,
            focused_player_idx: 0,
            focus_border_ticks: 0,
            is_shuffle_mode: true,
            hovered_player_id: None,
            open_audio_menu_player_id: None,
            subtitles_enabled: true,
            loading_player_ids: HashSet::new(),
            loading_player_ticks: HashMap::new(),
            spinner_ticks: 0,
            preloaded_player: None,
            is_preloading: false,
            cursor_position: Point::new(-1000.0, -1000.0),
            titlebar_press_origin: None,
            titlebar_drag_pending: false,
            is_window_dragging: false,
            last_window_drag_move: None,
            last_titlebar_click: None,
            expanded_folders: HashSet::new(),
            show_transcript: false,
            transcript_cues: Vec::new(),
            transcript_search: String::new(),
            transcript_loading: false,
            transcript_video_path: None,
            transcript_track_index: 0,
            show_transcript_menu: false,
            show_history_drawer: false,
            play_history: Vec::new(),
            history_search: String::new(),
            is_window_focused: true,
            unfocused_frame_ticks: 0,
            window_bounds_dirty: false,
            player_nav_history: HashMap::new(),
            player_shuffle_modes: HashMap::new(),
        };

        // Initialize players based on settings or restore saved session
        let count = settings.player_count.clamp(1, 12);
        let restored_sessions = if is_cli || file_to_play.is_some() {
            Vec::new()
        } else {
            settings.session_videos
        };

        let mut direct_file_player_id: Option<PlayerId> = None;

        if let Some(ref path) = file_to_play {
            let id = app.next_player_id;
            app.next_player_id += 1;
            direct_file_player_id = Some(id);
            app.player_shuffle_modes.insert(id, app.is_shuffle_mode);
            let name = format::format_video_title(path);
            if let Ok(mut handle) = app.create_video_handle_with_start(id, path, &name, Some(0.0)) {
                handle.set_muted(false);
                handle.set_volume(1.0);
                handle.set_subtitles_visible(app.subtitles_enabled);
                app.players.push(handle);
                app.push_player_nav_entry(id, path.clone(), Some(0.0));
            }
            app.title_pill_ticks = 240;
        } else {
            for session in restored_sessions.into_iter().take(count) {
                if std::path::Path::new(&session.path).exists() {
                    let id = app.next_player_id;
                    app.next_player_id += 1;
                    app.player_shuffle_modes.insert(id, session.is_shuffle);
                    if app.players.is_empty() {
                        app.is_shuffle_mode = session.is_shuffle;
                    }
                    let name = format::format_video_title(&session.path);
                    let start_secs = if session.position_secs > 0.05 {
                        Some(session.position_secs)
                    } else {
                        None
                    };
                    if let Ok(mut handle) =
                        app.create_video_handle_with_start(id, &session.path, &name, start_secs)
                    {
                        handle.set_muted(session.is_muted);
                        handle.set_volume(session.volume);
                        handle.set_subtitles_visible(app.subtitles_enabled);
                        app.players.push(handle);
                        app.push_player_nav_entry(id, session.path.clone(), start_secs);
                    }
                }
            }
        }

        // Fill remaining players if any
        while app.players.len() < count {
            if app.add_player_internal().is_none() {
                break;
            }
        }

        // If a last_query was active, reconcile active players so any player playing a video
        // not in the queried files list switches to a matching video from available_videos.
        // If playing a direct file, do not reconcile that player!
        if !app.settings.last_query.is_empty() {
            app.reconcile_players_with_available_videos(direct_file_player_id);
        }

        // Save session state if CLI query or direct file successfully populated players
        if (is_cli || file_to_play.is_some()) && !app.players.is_empty() {
            app.save_session_state();
        }

        app.apply_file_picker_search();

        let preload_task = if app.settings.playback_mode == PlaybackMode::Scroll {
            let items_with_heights: Vec<(PlayerId, f32)> = app
                .players
                .iter()
                .map(|p| (p.id, app.calculate_player_scroll_height(p)))
                .collect();
            app.scroll_engine
                .init_stack_with_heights(&items_with_heights);
            while let Some(spawn_y) = app.scroll_engine.needs_new_player() {
                if let Some(id) = app.add_player_internal() {
                    let item_h = app
                        .players
                        .iter()
                        .find(|p| p.id == id)
                        .map(|p| app.calculate_player_scroll_height(p))
                        .unwrap_or_else(|| app.scroll_engine.default_item_height());
                    app.scroll_engine.add_item(id, spawn_y, item_h);
                } else {
                    break;
                }
            }
            for p in &mut app.players {
                let vol = app.scroll_engine.calculate_player_volume(p.id);
                p.set_volume(vol);
            }
            app.trigger_preload_task()
        } else {
            Task::none()
        };

        // Boot focus task: ensure window receives keyboard focus and activation on boot
        let focus_task = Task::perform(
            async {
                #[cfg(target_os = "linux")]
                {
                    crate::platform::ensure_window_focused_linux().await;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            },
            |_| Message::GainWindowFocus,
        );

        (app, Task::batch([preload_task, focus_task]))
    }

    pub(crate) fn is_any_modal_open(&self) -> bool {
        self.show_search_modal
            || self.show_settings_modal
            || self.show_help_modal
            || self.show_bookmarks_modal
            || self.show_menu_modal
    }

    pub(crate) fn is_modal_or_menu_open(&self) -> bool {
        self.is_any_modal_open() || self.show_dropdown_menu
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
        if !self.show_titlebar {
            0.0
        } else if self.is_window_dragging
            || self.titlebar_drag_pending
            || self.is_point_in_titlebar(self.cursor_position)
            || self.show_dropdown_menu
            || self.titlebar_hide_ticks >= TITLEBAR_FADE_TICKS
        {
            1.0
        } else {
            (self.titlebar_hide_ticks as f32 / TITLEBAR_FADE_TICKS as f32).clamp(0.0, 1.0)
        }
    }

    pub(crate) fn player_overlay_alpha(&self) -> f32 {
        if self.player_overlay_ticks >= PLAYER_OVERLAY_FADE_TICKS {
            1.0
        } else {
            (self.player_overlay_ticks as f32 / PLAYER_OVERLAY_FADE_TICKS as f32).clamp(0.0, 1.0)
        }
    }

    pub fn focused_player_id(&self) -> Option<PlayerId> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_player_idx % self.players.len();
            Some(self.players[idx].id)
        }
    }

    pub(crate) fn focused_player(&self) -> Option<&VideoHandle> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_player_idx % self.players.len();
            Some(&self.players[idx])
        }
    }

    pub(crate) fn focused_player_mut(&mut self) -> Option<&mut VideoHandle> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_player_idx % self.players.len();
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
            self.transcript_cues.clear();
            self.transcript_video_path = None;
            self.transcript_loading = false;
            self.transcript_track_index = 0;
            self.show_transcript_menu = false;
            return Task::none();
        };

        if path.is_empty() {
            self.transcript_cues.clear();
            self.transcript_video_path = None;
            self.transcript_loading = false;
            self.transcript_track_index = 0;
            self.show_transcript_menu = false;
            return Task::none();
        }

        // Sync track index with selected player subtitle track if video path changed
        if self.transcript_video_path.as_deref() != Some(&path) {
            self.transcript_track_index = selected_track_pos;
            self.show_transcript_menu = false;
        } else if !self.transcript_cues.is_empty() {
            return Task::none();
        }

        self.transcript_video_path = Some(path.clone());
        self.transcript_loading = true;
        self.transcript_cues.clear();
        let path_clone = path.clone();
        let track_idx = self.transcript_track_index;
        let sub_track = sub_tracks.get(track_idx).cloned();
        Task::perform(
            async move {
                if let Some(track) = sub_track {
                    if let Some(ext_file) = track.external_filename {
                        if let Ok(content) = tokio::fs::read_to_string(&ext_file).await {
                            let cues = wazoo_media::parse_subtitles(&content);
                            if !cues.is_empty() {
                                return cues;
                            }
                        }
                    }
                    wazoo_media::load_subtitles_for_stream(path, track.ff_index, track_idx).await
                } else {
                    wazoo_media::load_subtitles_for_stream(path, None, track_idx).await
                }
            },
            move |cues| Message::TranscriptLoaded(path_clone, cues),
        )
    }

    pub(crate) fn is_player_shuffle(&self, id: PlayerId) -> bool {
        self.player_shuffle_modes
            .get(&id)
            .copied()
            .unwrap_or(self.is_shuffle_mode)
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
        self.get_next_video_rec_with_mode(current_path, self.is_shuffle_mode)
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
        if let Some(p) = self.players.iter().find(|pl| pl.id == id) {
            let pos_secs = p.position().as_secs_f64();
            let dur_secs = p.duration().as_secs_f64();
            let saved_pos = if dur_secs > 10.0 && (dur_secs - pos_secs) < 3.0 {
                None
            } else if pos_secs > 0.5 {
                Some(pos_secs)
            } else {
                None
            };
            if let Some(hist) = self.player_nav_history.get_mut(&id) {
                if let Some(top) = hist.back_stack.last_mut() {
                    if top.path == p.state.path {
                        top.position_secs = saved_pos;
                    }
                }
            }
        }
    }

    pub fn record_play_history(&mut self, path: &str) {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.play_history.last().map(|e| e.path.as_str()) == Some(trimmed) {
            return;
        }
        let title = format::format_video_title(trimmed);
        let folder = format::format_video_folder(trimmed);
        self.play_history.push(PlayHistoryItem {
            path: trimmed.to_string(),
            title,
            folder,
        });
        if self.play_history.len() > MAX_PLAY_HISTORY_ENTRIES {
            self.play_history.remove(0);
        }
    }

    pub fn push_player_nav_entry(&mut self, id: PlayerId, path: String, pos: Option<f64>) {
        self.record_play_history(&path);
        let hist = self.player_nav_history.entry(id).or_default();
        if hist.back_stack.last().map(|e| &e.path) != Some(&path) {
            hist.back_stack.push(PlaybackHistoryEntry {
                path,
                position_secs: pos,
            });
            if hist.back_stack.len() > 100 {
                hist.back_stack.remove(0);
            }
        }
    }

    pub(crate) fn buffer_config(&self) -> BufferConfig {
        BufferConfig {
            duration_secs: self.settings.buffer_duration_secs,
            size_mb: self.settings.buffer_size_mb,
            read_chunk_kb: 512,
            preferred_audio_language: self.settings.preferred_audio_language.clone(),
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
        if self.is_alt_pressed {
            0.85
        } else {
            self.settings.window_opacity.clamp(0.05, 1.0)
        }
    }

    pub(crate) fn add_player_internal(&mut self) -> Option<PlayerId> {
        let id = self.next_player_id;
        self.next_player_id += 1;

        // In scroll mode, mute follows the scroll feed's global mute setting.
        // In ambient grid mode, new players start muted by default so they don't
        // create unexpected noise or audio overlap.
        let initial_muted = if self.settings.playback_mode == PlaybackMode::Scroll {
            self.settings.is_global_muted
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

        for _ in 0..3 {
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
                        self.push_player_nav_entry(id, video_rec.path.clone(), None);
                        self.player_shuffle_modes.insert(id, self.is_shuffle_mode);
                        self.players.push(handle);
                        self.loading_player_ids.insert(id);
                        self.loading_player_ticks.insert(id, 0);
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
                    is_shuffle: self.is_player_shuffle(p.id),
                })
                .collect();
            self.settings.session_videos = sessions;
        }
        self.settings.last_query = self.active_search_query.clone();
        self.settings.last_folders = self.active_search_folders.clone();
        let _ = self.config_mgr.save_settings(&self.settings);
        self.window_bounds_dirty = false;
    }

    /// Reconciles active players against `self.available_videos`.
    /// Any player currently playing a file that does NOT exist in `self.available_videos`
    /// is switched to play a video that DOES exist in `self.available_videos`.
    /// Each switching player is assigned a distinct video when possible.
    pub fn reconcile_players_with_available_videos(
        &mut self,
        skip_player_id: Option<PlayerId>,
    ) {
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
            if let Some(p) = self.players.iter().find(|pl| pl.id == id) {
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

                self.loading_player_ids.insert(id);
                self.loading_player_ticks.insert(id, 0);

                if let Ok(mut new_handle) = self.create_video_handle(id, &rec_path, &rec_name) {
                    new_handle.set_muted(prev_muted);
                    new_handle.set_volume(prev_volume);
                    new_handle.set_subtitles_visible(self.subtitles_enabled);
                    self.push_player_nav_entry(id, rec_path.clone(), None);
                    if let Some(p) = self.players.iter_mut().find(|pl| pl.id == id) {
                        *p = new_handle;
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
                    self.preloaded_player = None;
                    self.is_preloading = false;
                }
            }
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

                    // Background pre-buffer: decode initial presentation frame off-thread
                    // so the player is 100% ready when attached to the scroll feed without hitching.
                    let start = std::time::Instant::now();
                    while !handle.has_decoded_frame()
                        && start.elapsed() < Duration::from_millis(2000)
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
        if self.settings.playback_mode == PlaybackMode::Scroll {
            self.settings.playback_mode = PlaybackMode::Normal;
            self.scroll_engine.clear();
            self.preloaded_player = None;
            self.is_preloading = false;
            if self.settings.is_global_muted {
                for p in &mut self.players {
                    p.set_muted(true);
                }
            } else {
                for (i, p) in self.players.iter_mut().enumerate() {
                    if i == self.focused_player_idx {
                        p.set_muted(false);
                        p.set_volume(1.0);
                    } else {
                        p.set_muted(true);
                    }
                }
            }
        }
    }

    pub fn is_all_folder(&self, folder: &str) -> bool {
        wazoo_core::i18n::is_all_folder(folder)
    }

    pub fn is_all_search_selected(&self) -> bool {
        self.selected_search_folders.is_empty()
            || (self.selected_search_folders.len() == 1
                && self.is_all_folder(&self.selected_search_folders[0]))
    }

    pub(crate) fn reset_search_folder_selection(&mut self) {
        self.selected_search_folders.clear();
        self.selected_search_folder = self.t("common.all");
    }

    pub(crate) fn toggle_search_folder(&mut self, folder: &str) {
        if self.is_all_folder(folder) {
            self.reset_search_folder_selection();
            return;
        }

        if let Some(pos) = self.selected_search_folders.iter().position(|f| f == folder) {
            self.selected_search_folders.remove(pos);
        } else {
            self.selected_search_folders.push(folder.to_string());
        }

        if self.selected_search_folders.is_empty() {
            self.selected_search_folder = self.t("common.all");
        } else if self.selected_search_folders.len() == 1 {
            self.selected_search_folder = self.selected_search_folders[0].clone();
        } else {
            self.selected_search_folder = self.selected_search_folders.join(", ");
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
            self.toast_message = Some(msg);
            self.toast_time_remaining = 3;
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
        self.toast_message = Some(msg);
        self.toast_time_remaining = 3;
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

        if self.settings.playback_mode == PlaybackMode::Flip {
            let interval_secs = self.settings.flip_interval_secs.max(1);
            subs.push(
                iced::time::every(Duration::from_secs(interval_secs))
                    .map(|_| Message::FlipModeTick),
            );
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

