/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Core Application State
 * 
 * Defines the central WazooApp struct, initial state construction, player lifecycle management,
 * active session persistence, search reconciliation, and event subscription bindings.
 */

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use iced::{Point, Subscription, Task, Theme};
use wazoo_core::{ConfigManager, Database, PlaybackMode, VideoRecord, VideoSession, WazooSettings};
use wazoo_media::{BufferConfig, PlayerId, ScrollEngine, VideoHandle};
use wazoo_scanner::ScanProgress;
use crate::assets::APP_ICON_BYTES;
use crate::format;
use crate::message::Message;

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

pub struct WazooApp {
    pub(crate) settings: WazooSettings,
    pub(crate) config_mgr: ConfigManager,
    pub(crate) db: Database,
    pub(crate) players: Vec<VideoHandle>,
    pub(crate) scroll_engine: ScrollEngine,
    pub(crate) available_videos: Vec<VideoRecord>,
    pub(crate) active_search_query: String,
    pub(crate) search_input: String,
    pub(crate) search_tags: Vec<String>,
    pub(crate) folder_input: String,
    pub(crate) selected_search_folder: String,
    pub(crate) show_search_modal: bool,
    pub(crate) show_settings_modal: bool,
    pub(crate) show_help_modal: bool,
    pub(crate) show_menu_modal: bool,
    pub(crate) show_bookmarks_modal: bool,
    pub(crate) show_file_picker: bool,
    pub(crate) file_picker_search: String,
    pub(crate) show_titlebar: bool,
    pub(crate) titlebar_hide_ticks: usize,
    pub(crate) titlebar_hover_ticks: usize,
    pub(crate) show_dropdown_menu: bool,
    pub(crate) is_alt_pressed: bool,
    pub(crate) player_overlay_ticks: usize,
    pub(crate) title_pill_ticks: usize,
    pub(crate) window_id: Option<iced::window::Id>,
    pub(crate) app_icon_handle: iced::widget::image::Handle,
    pub(crate) toast_message: Option<String>,
    pub(crate) toast_time_remaining: usize,
    pub(crate) last_total_videos: usize,
    pub(crate) next_player_id: PlayerId,
    pub(crate) is_scanning: bool,
    pub(crate) current_scan_id: u64,
    pub(crate) scan_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub(crate) scan_progress: Option<ScanProgress>,
    pub(crate) focused_player_idx: usize,
    pub(crate) focus_border_ticks: usize,
    pub(crate) is_shuffle_mode: bool,
    pub(crate) hovered_player_id: Option<PlayerId>,
    pub(crate) open_audio_menu_player_id: Option<PlayerId>,
    pub(crate) subtitles_enabled: bool,
    pub(crate) loading_player_ids: HashSet<PlayerId>,
    pub(crate) loading_player_ticks: HashMap<PlayerId, usize>,
    pub(crate) spinner_ticks: u32,
    pub(crate) preloaded_player: Option<VideoHandle>,
    pub(crate) is_preloading: bool,
    pub(crate) cursor_position: Point,
    pub(crate) titlebar_press_origin: Option<Point>,
    pub(crate) titlebar_drag_pending: bool,
    pub(crate) last_titlebar_click: Option<Instant>,
    pub(crate) expanded_folders: HashSet<String>,
    pub(crate) show_transcript: bool,
    pub(crate) transcript_cues: Vec<wazoo_media::SubtitleCue>,
    pub(crate) transcript_search: String,
    pub(crate) transcript_loading: bool,
    pub(crate) transcript_video_path: Option<String>,
}

impl WazooApp {
    pub fn new(cli_query: Option<String>) -> (Self, Task<Message>) {
        let config_mgr = ConfigManager::new();
        let mut settings = config_mgr.load_settings();
        let db = Database::open(config_mgr.database_path())
            .expect("Failed to initialize SQLite database");

        let cli_query_clean = cli_query.map(|q| q.trim().to_string()).filter(|q| !q.is_empty());
        let is_cli = cli_query_clean.is_some();

        let (folders, active_query, selected_folder) = if let Some(ref q) = cli_query_clean {
            settings.last_query = q.clone();
            settings.last_folder = "All".to_string();
            let _ = config_mgr.save_settings(&settings);
            (Vec::new(), q.clone(), "All".to_string())
        } else {
            let f = if settings.last_folder.is_empty() || settings.last_folder == "All" {
                Vec::new()
            } else {
                vec![settings.last_folder.clone()]
            };
            let sel = if settings.last_folder.is_empty() {
                "All".to_string()
            } else {
                settings.last_folder.clone()
            };
            (f, settings.last_query.clone(), sel)
        };

        let videos: Vec<VideoRecord> = if !active_query.is_empty() {
            let filtered = db.search_videos(&active_query, &folders).unwrap_or_default();
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
        let mut scroll_engine = ScrollEngine::new(settings.window_bounds.height as f32);
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
            selected_search_folder: selected_folder,
            show_search_modal: false,
            show_settings_modal: false,
            show_help_modal: false,
            show_menu_modal: false,
            show_bookmarks_modal: false,
            show_file_picker: false,
            file_picker_search: String::new(),
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
            last_titlebar_click: None,
            expanded_folders: HashSet::new(),
            show_transcript: false,
            transcript_cues: Vec::new(),
            transcript_search: String::new(),
            transcript_loading: false,
            transcript_video_path: None,
        };

        // Initialize players based on settings or restore saved session
        let count = settings.player_count.clamp(1, 12);
        let restored_sessions = if is_cli {
            Vec::new()
        } else {
            settings.session_videos.clone()
        };

        for session in restored_sessions.into_iter().take(count) {
            if std::path::Path::new(&session.path).exists() {
                let id = app.next_player_id;
                app.next_player_id += 1;
                let name = format::format_video_title(&session.path);
                let buffer_config = BufferConfig {
                    duration_secs: app.settings.buffer_duration_secs,
                    size_mb: app.settings.buffer_size_mb,
                    read_chunk_kb: 512,
                    preferred_audio_language: app.settings.preferred_audio_language.clone(),
                };
                let start_secs = if session.position_secs > 0.05 {
                    Some(session.position_secs)
                } else {
                    None
                };
                if let Ok(mut handle) = VideoHandle::with_buffering_and_start(id, &session.path, &name, buffer_config, start_secs) {
                    handle.set_muted(session.is_muted);
                    handle.set_volume(session.volume);
                    handle.set_subtitles_visible(app.subtitles_enabled);
                    app.players.push(handle);
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
        // not in the queried files list switches to a matching video from available_videos
        if !app.settings.last_query.is_empty() {
            app.reconcile_players_with_available_videos(None);
        }

        // Save session state if CLI query successfully populated players
        if is_cli && !app.players.is_empty() {
            app.save_session_state();
        }

        let preload_task = if app.settings.playback_mode == PlaybackMode::Scroll {
            let ids: Vec<PlayerId> = app.players.iter().map(|p| p.id).collect();
            app.scroll_engine.init_stack(&ids);
            let item_h = app.scroll_engine.default_item_height();
            while let Some(spawn_y) = app.scroll_engine.needs_new_player() {
                if let Some(id) = app.add_player_internal() {
                    app.scroll_engine.add_item(id, spawn_y, item_h);
                } else {
                    break;
                }
            }
            if app.settings.session_videos.is_empty() || is_cli {
                for p in &mut app.players {
                    p.seek_random();
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

    pub(crate) fn is_point_in_titlebar(&self, pos: Point) -> bool {
        let width = if self.settings.window_bounds.width > 0 {
            self.settings.window_bounds.width as f32
        } else {
            f32::MAX
        };
        pos.x >= 0.0 && pos.x <= width && pos.y >= 0.0 && pos.y < 35.0
    }

    pub(crate) fn titlebar_alpha(&self) -> f32 {
        if !self.show_titlebar {
            0.0
        } else if self.is_point_in_titlebar(self.cursor_position) || self.show_dropdown_menu || self.titlebar_hide_ticks >= TITLEBAR_FADE_TICKS {
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

    pub(crate) fn focused_player_id(&self) -> Option<PlayerId> {
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

    pub(crate) fn load_transcript_for_focused_player(&mut self) -> Task<Message> {
        if let Some(player) = self.focused_player() {
            let path = player.state.path.clone();
            if path.is_empty() {
                self.transcript_cues.clear();
                self.transcript_video_path = None;
                self.transcript_loading = false;
                return Task::none();
            }
            if self.transcript_video_path.as_deref() == Some(&path) && !self.transcript_cues.is_empty() {
                return Task::none();
            }
            self.transcript_video_path = Some(path.clone());
            self.transcript_loading = true;
            self.transcript_cues.clear();
            let path_clone = path.clone();
            Task::perform(
                wazoo_media::load_subtitles(path),
                move |cues| Message::TranscriptLoaded(path_clone, cues),
            )
        } else {
            self.transcript_cues.clear();
            self.transcript_video_path = None;
            self.transcript_loading = false;
            Task::none()
        }
    }

    pub(crate) fn get_next_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
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

    pub(crate) fn get_prev_video_rec(&self, current_path: Option<&str>) -> Option<VideoRecord> {
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

    pub(crate) fn create_video_handle(&self, id: PlayerId, path: &str, name: &str) -> Result<VideoHandle, String> {
        self.create_video_handle_with_start(id, path, name, None)
    }

    pub(crate) fn create_video_handle_with_start(&self, id: PlayerId, path: &str, name: &str, start_secs: Option<f64>) -> Result<VideoHandle, String> {
        let buffer_config = BufferConfig {
            duration_secs: self.settings.buffer_duration_secs,
            size_mb: self.settings.buffer_size_mb,
            read_chunk_kb: 512,
            preferred_audio_language: self.settings.preferred_audio_language.clone(),
        };
        VideoHandle::with_buffering_and_start(id, path, name, buffer_config, start_secs)
    }

    pub fn current_opacity(&self) -> f32 {
        if self.is_alt_pressed {
            0.85
        } else {
            self.settings.window_opacity.clamp(0.1, 1.0)
        }
    }

    pub(crate) fn add_player_internal(&mut self) -> Option<PlayerId> {
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

    /// Persists the exact current playback session (file, timestamp, mute, volume) to settings
    pub(crate) fn save_session_state(&mut self) {
        if !self.players.is_empty() {
            let sessions: Vec<VideoSession> = self
                .players
                .iter()
                .map(|p| VideoSession {
                    path: p.state.path.clone(),
                    position_secs: p.position().as_secs_f64(),
                    is_muted: p.state.is_muted,
                    volume: p.state.volume,
                })
                .collect();
            self.settings.session_videos = sessions;
            self.settings.last_query = self.active_search_query.clone();
            self.settings.last_folder = self.selected_search_folder.clone();
            let _ = self.config_mgr.save_settings(&self.settings);
        }
    }

    /// Reconciles active players against `self.available_videos`.
    /// Any player currently playing a file that does NOT exist in `self.available_videos`
    /// is switched to play a video that DOES exist in `self.available_videos`.
    /// Each switching player is assigned a distinct video when possible.
    pub(crate) fn reconcile_players_with_available_videos(&mut self, skip_player_id: Option<PlayerId>) {
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
        let mut used_paths: HashSet<String> = self.players.iter().map(|p| p.state.path.clone()).collect();
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

            let candidate = if !unused.is_empty() {
                if self.is_shuffle_mode {
                    let r = rand::random::<usize>() % unused.len();
                    Some(unused[r])
                } else {
                    Some(unused[0])
                }
            } else {
                let fallback_idx = if self.is_shuffle_mode {
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
                    if let Some(p) = self.players.iter_mut().find(|pl| pl.id == id) {
                        *p = new_handle;
                    }
                }
            }
        }

        if self.settings.playback_mode == PlaybackMode::Scroll {
            if let Some(ref preloaded) = self.preloaded_player {
                if !self.available_videos.iter().any(|v| v.path == preloaded.state.path) {
                    self.preloaded_player = None;
                    self.is_preloading = false;
                }
            }
        }
    }

    /// Triggers an asynchronous background preload task for the next video in scroll mode
    pub(crate) fn trigger_preload_task(&mut self) -> Task<Message> {
        if self.preloaded_player.is_some() || self.is_preloading || self.settings.playback_mode != PlaybackMode::Scroll {
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

        let buffer_config = BufferConfig {
            duration_secs: self.settings.buffer_duration_secs,
            size_mb: self.settings.buffer_size_mb,
            read_chunk_kb: 512,
            preferred_audio_language: self.settings.preferred_audio_language.clone(),
        };
        let path = video_rec.path;
        let name = video_rec.name;

        let holder = Arc::new(Mutex::new(None));
        let holder_clone = Arc::clone(&holder);

        Task::perform(
            async move {
                let res = tokio::task::spawn_blocking(move || {
                    let mut handle = VideoHandle::with_buffering(id, &path, &name, buffer_config)?;
                    handle.seek_random();
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
        self.db.get_video_count().unwrap_or(self.available_videos.len())
    }

    /// Resets scroll engine and transitions playback mode back to Normal
    pub(crate) fn cleanup_scroll_mode(&mut self) {
        if self.settings.playback_mode == PlaybackMode::Scroll {
            self.settings.playback_mode = PlaybackMode::Normal;
            self.scroll_engine.clear();
            self.preloaded_player = None;
            self.is_preloading = false;
            for p in &mut self.players {
                p.set_muted(self.settings.is_global_muted);
                p.set_volume(1.0);
            }
        }
    }

    pub(crate) fn show_video_totals_notice(&mut self, total: usize, folder_label: &str) {
        if total == 0 {
            let msg = if folder_label.is_empty() || folder_label == "All" {
                self.t("wazoo.no_videos_found")
            } else {
                let folder_clean = folder_label
                    .split(['/', '\\'])
                    .filter(|s| !s.is_empty())
                    .next_back()
                    .unwrap_or(folder_label);
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
                iced::Event::Mouse(iced::mouse::Event::CursorLeft) => {
                    Some(Message::CursorLeft)
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) => {
                    Some(Message::RightClickPressed(window_id))
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                    Some(Message::LeftClickReleased)
                }
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Message::WindowResized(window_id, size))
                }
                iced::Event::Window(iced::window::Event::Focused) => {
                    Some(Message::WindowFocused)
                }
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
            subs.push(iced::time::every(Duration::from_secs(15)).map(|_| Message::FlipModeTick));
        }

        Subscription::batch(subs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_point_in_titlebar() {
        let (mut app, _) = WazooApp::new(None);
        app.settings.window_bounds.width = 800;
        app.settings.window_bounds.height = 600;

        // Inside titlebar bounds (0 <= x <= 800, 0 <= y < 35)
        assert!(app.is_point_in_titlebar(Point::new(0.0, 0.0)));
        assert!(app.is_point_in_titlebar(Point::new(400.0, 20.0)));
        assert!(app.is_point_in_titlebar(Point::new(800.0, 34.9)));

        // Outside titlebar bounds
        assert!(!app.is_point_in_titlebar(Point::new(400.0, 35.0)));
        assert!(!app.is_point_in_titlebar(Point::new(400.0, 100.0)));
        assert!(!app.is_point_in_titlebar(Point::new(400.0, -1.0)));
        assert!(!app.is_point_in_titlebar(Point::new(-10.0, 10.0)));
        assert!(!app.is_point_in_titlebar(Point::new(801.0, 10.0)));
    }

    #[test]
    fn test_cursor_left_uses_hide_timer() {
        let (mut app, _) = WazooApp::new(None);

        // Move cursor to top of window and wait for pre-delay -> titlebar shows
        let win_id = iced::window::Id::unique();
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 10.0)));
        assert!(!app.show_titlebar); // Pre-delay active
        for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
            let _ = app.update(Message::VideoFrameTick);
        }
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_HIDE_TICKS);
        assert_eq!(app.titlebar_alpha(), 1.0);

        // Cursor leaves window exiting through the top (should NOT instantly vanish)
        let _ = app.update(Message::CursorLeft);
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_HIDE_TICKS);
        assert_eq!(app.cursor_position, Point::new(-1000.0, -1000.0));

        // Grace period (ticks until fade begins): alpha stays 1.0
        let grace_ticks = TITLEBAR_HIDE_TICKS - TITLEBAR_FADE_TICKS;
        for _ in 0..grace_ticks {
            let _ = app.update(Message::VideoFrameTick);
            assert!(app.show_titlebar);
            assert_eq!(app.titlebar_alpha(), 1.0);
        }
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_FADE_TICKS);

        // Fade period (ticks fading down to 1): alpha smoothly decreases
        for i in (1..TITLEBAR_FADE_TICKS).rev() {
            let _ = app.update(Message::VideoFrameTick);
            assert!(app.show_titlebar);
            assert!((app.titlebar_alpha() - (i as f32 / TITLEBAR_FADE_TICKS as f32)).abs() < 0.001);
        }

        // Final frame tick hides titlebar
        let _ = app.update(Message::VideoFrameTick);
        assert!(!app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, 0);
        assert_eq!(app.titlebar_alpha(), 0.0);
    }

    #[test]
    fn test_cursor_left_keeps_titlebar_if_dropdown_open() {
        let (mut app, _) = WazooApp::new(None);

        let _ = app.update(Message::ToggleDropdownMenu);
        assert!(app.show_dropdown_menu);
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_alpha(), 1.0);

        // When dropdown is open, cursor leaving window should keep titlebar visible
        let _ = app.update(Message::CursorLeft);
        assert!(app.show_dropdown_menu);
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_alpha(), 1.0);

        // Closing dropdown should start hide timer when cursor is outside
        let _ = app.update(Message::CloseDropdownMenu);
        assert!(!app.show_dropdown_menu);
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_HIDE_TICKS);

        for _ in 0..TITLEBAR_HIDE_TICKS {
            let _ = app.update(Message::VideoFrameTick);
        }
        assert!(!app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, 0);
    }

    #[test]
    fn test_titlebar_hides_after_ticks_when_cursor_moves_away() {
        let (mut app, _) = WazooApp::new(None);

        let win_id = iced::window::Id::unique();
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 10.0)));
        assert!(!app.show_titlebar); // Pre-delay active
        for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
            let _ = app.update(Message::VideoFrameTick);
        }
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_HIDE_TICKS);

        // Move cursor down into video area
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 50.0)));
        assert!(app.show_titlebar);

        // Tick down frames until 1 frame remaining
        for _ in 0..(TITLEBAR_HIDE_TICKS - 1) {
            let _ = app.update(Message::VideoFrameTick);
            assert!(app.show_titlebar);
        }

        // Final frame tick should hide titlebar
        let _ = app.update(Message::VideoFrameTick);
        assert!(!app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, 0);
    }

    #[test]
    fn test_titlebar_pre_delay_prevents_quick_swipe() {
        let (mut app, _) = WazooApp::new(None);
        let win_id = iced::window::Id::unique();

        // 1. Move cursor into titlebar zone (quick swipe for 3 ticks)
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 10.0)));
        assert!(!app.show_titlebar);
        for _ in 0..3 {
            let _ = app.update(Message::VideoFrameTick);
            assert!(!app.show_titlebar);
            assert_eq!(app.titlebar_alpha(), 0.0);
        }

        // 2. Cursor quickly leaves titlebar zone
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 60.0)));
        let _ = app.update(Message::VideoFrameTick);
        assert!(!app.show_titlebar);
        assert_eq!(app.titlebar_hover_ticks, 0);

        // 3. Now cursor moves into titlebar zone and lingers for full pre-delay
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 10.0)));
        assert!(!app.show_titlebar);
        for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
            let _ = app.update(Message::VideoFrameTick);
        }
        assert!(app.show_titlebar);
        assert_eq!(app.titlebar_hide_ticks, TITLEBAR_HIDE_TICKS);
        assert_eq!(app.titlebar_alpha(), 1.0);
    }

    #[test]
    fn test_player_controls_hide_timer_and_fade() {
        let (mut app, _) = WazooApp::new(None);
        let player_id = 42;

        // 1. Hover over player starts full 2.5s timer
        let _ = app.update(Message::PlayerHovered(player_id));
        assert_eq!(app.hovered_player_id, Some(player_id));
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);
        assert_eq!(app.player_overlay_alpha(), 1.0);

        // 2. Unhover player (moving mouse outside) immediately triggers fade out
        let _ = app.update(Message::PlayerUnhovered(player_id));
        assert_eq!(app.hovered_player_id, Some(player_id));
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_FADE_TICKS);
        assert_eq!(app.player_overlay_alpha(), 1.0);

        // Subsequent cursor movements outside the player must NOT reset overlay ticks
        let win_id = iced::window::Id::unique();
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 60.0))); // Outside player
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_FADE_TICKS);

        // Fade down over 25 ticks
        for i in (1..PLAYER_OVERLAY_FADE_TICKS).rev() {
            let _ = app.update(Message::VideoFrameTick);
            assert_eq!(app.hovered_player_id, Some(player_id));
            assert!((app.player_overlay_alpha() - (i as f32 / PLAYER_OVERLAY_FADE_TICKS as f32)).abs() < 0.001);
        }

        // Final frame tick completes fade and resets hovered player
        let _ = app.update(Message::VideoFrameTick);
        assert_eq!(app.hovered_player_id, None);
        assert_eq!(app.player_overlay_ticks, 0);
        assert_eq!(app.player_overlay_alpha(), 0.0);

        // 3. Test cursor moving into titlebar triggers immediate fade out once pre-delay elapses
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 100.0))); // Into player area
        let _ = app.update(Message::PlayerHovered(player_id));
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 10.0))); // In titlebar
        for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
            let _ = app.update(Message::VideoFrameTick);
        }
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_FADE_TICKS);

        // 4. Test cursor leaving window also triggers immediate fade out
        let _ = app.update(Message::CursorMoved(win_id, Point::new(100.0, 100.0))); // Into player area
        let _ = app.update(Message::PlayerHovered(player_id));
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);
        let _ = app.update(Message::CursorLeft);
        assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_FADE_TICKS);
    }

    #[test]
    fn test_show_video_totals_notice_transitions() {
        let (mut app, _) = WazooApp::new(None);
        app.settings.language = "en".to_string();

        // 1. Initial count when last_total_videos was 0
        app.last_total_videos = 0;
        app.show_video_totals_notice(1000, "All");
        assert_eq!(app.toast_message.as_deref(), Some("1,000 Total files"));
        assert_eq!(app.last_total_videos, 1000);

        // 2. Count increases ("More than before")
        app.show_video_totals_notice(1050, "All");
        assert_eq!(
            app.toast_message.as_deref(),
            Some("1,050 Total files\n50 More than before")
        );
        assert_eq!(app.last_total_videos, 1050);

        // 3. Count decreases ("Less than before")
        app.show_video_totals_notice(200, "All");
        assert_eq!(
            app.toast_message.as_deref(),
            Some("200 Total files\n850 Less than before")
        );
        assert_eq!(app.last_total_videos, 200);

        // 4. Same count (no diff)
        app.show_video_totals_notice(200, "All");
        assert_eq!(app.toast_message.as_deref(), Some("200 Total files"));
        assert_eq!(app.last_total_videos, 200);

        // 5. Zero results with "All" or empty folder
        app.show_video_totals_notice(0, "All");
        assert_eq!(
            app.toast_message.as_deref(),
            Some("No videos found for query")
        );
        assert_eq!(app.last_total_videos, 200); // last_total_videos preserved

        // 6. Zero results with specific folder
        app.show_video_totals_notice(0, "/media/videos/Anime");
        assert_eq!(app.toast_message.as_deref(), Some("No files found in Anime"));
        assert_eq!(app.last_total_videos, 200); // last_total_videos preserved
    }

    #[test]
    fn test_search_tags_comma_and_backspace() {
        use iced::keyboard::{key::Named, Key};

        let (mut app, _) = WazooApp::new(None);
        app.active_search_query.clear();
        let _ = app.update(Message::OpenSearchModal);
        assert!(app.show_search_modal);
        assert!(app.search_tags.is_empty());
        assert!(app.search_input.is_empty());

        // 1. Typing without comma updates search_input
        let _ = app.update(Message::SearchInputChanged("bebop".to_string()));
        assert_eq!(app.search_input, "bebop");
        assert!(app.search_tags.is_empty());

        // 2. Typing comma commits tag and clears search_input
        let _ = app.update(Message::SearchInputChanged("bebop,".to_string()));
        assert_eq!(app.search_tags, vec!["bebop"]);
        assert_eq!(app.search_input, "");

        // 3. Pasting multiple comma-separated items
        let _ = app.update(Message::SearchInputChanged("cowboy, space, spike".to_string()));
        assert_eq!(app.search_tags, vec!["bebop", "cowboy", "space"]);
        assert_eq!(app.search_input, "spike");

        // 4. Backspace when search_input is not empty does NOT pop tag
        let _ = app.update(Message::KeyPressed(Key::Named(Named::Backspace), iced::event::Status::Captured));
        assert_eq!(app.search_tags.len(), 3);

        // 5. Backspace when search_input is empty pops the last tag
        app.search_input.clear();
        let _ = app.update(Message::KeyPressed(Key::Named(Named::Backspace), iced::event::Status::Captured));
        assert_eq!(app.search_tags, vec!["bebop", "cowboy"]);

        // 6. Remove specific tag by index
        let _ = app.update(Message::RemoveSearchTag(0));
        assert_eq!(app.search_tags, vec!["cowboy"]);

        // 7. PerformSearch commits pending text and builds full query
        let _ = app.update(Message::SearchInputChanged("anime".to_string()));
        let _ = app.update(Message::PerformSearch);
        assert_eq!(app.active_search_query, "cowboy, anime");
        assert_eq!(app.search_tags, vec!["cowboy", "anime"]);
        assert_eq!(app.search_input, "");
        assert!(!app.show_search_modal);

        // 8. Re-opening search modal populates tags from active_search_query
        let _ = app.update(Message::OpenSearchModal);
        assert_eq!(app.search_tags, vec!["cowboy", "anime"]);
        assert_eq!(app.search_input, "");
    }

    #[test]
    fn test_search_modal_view_rendering() {
        let (mut app, _) = WazooApp::new(None);
        app.show_search_modal = true;

        // View with empty tags
        let _ = app.view_search_modal();

        // View with multiple tags
        app.search_tags = vec!["bebop".to_string(), "space".to_string()];
        app.search_input = "cowboy".to_string();
        let _ = app.view_search_modal();
    }

    #[test]
    fn test_concurrent_folder_scan_aborts_and_restarts() {
        use std::sync::atomic::Ordering;

        let (mut app, _) = WazooApp::new(None);
        app.settings.media_folders = vec!["/folder1".to_string()];

        // 1. Initial StartScan initializes scan state
        let _ = app.update(Message::StartScan);
        assert!(app.is_scanning);
        assert_eq!(app.current_scan_id, 1);
        let first_cancel = app.scan_cancel.clone().expect("scan_cancel should be set");
        assert!(!first_cancel.load(Ordering::SeqCst));

        // 2. Adding a folder while scanning aborts the first scan and starts a new one immediately
        let _ = app.update(Message::FoldersSelected(vec!["/folder2".to_string()]));
        assert_eq!(app.settings.media_folders, vec!["/folder1", "/folder2"]);
        assert!(first_cancel.load(Ordering::SeqCst), "first scan cancel flag should be set to true");
        assert!(app.is_scanning);
        assert_eq!(app.current_scan_id, 2);
        let second_cancel = app.scan_cancel.clone().expect("second scan_cancel should be set");
        assert!(!second_cancel.load(Ordering::SeqCst));

        // 3. Adding another folder via input aborts second scan and starts a third one immediately
        app.folder_input = "/folder3".to_string();
        let _ = app.update(Message::AddMediaFolder);
        assert_eq!(app.settings.media_folders, vec!["/folder1", "/folder2", "/folder3"]);
        assert!(second_cancel.load(Ordering::SeqCst), "second scan cancel flag should be set to true");
        assert!(app.is_scanning);
        assert_eq!(app.current_scan_id, 3);

        // 4. Stale ScanProgressUpdate and ScanFinished from earlier scans (scan_id 1 or 2) are ignored
        let _ = app.update(Message::ScanFinished(1, Ok(10)));
        assert!(app.is_scanning, "app should still be scanning because scan_id 1 is obsolete");

        let _ = app.update(Message::ScanFinished(2, Ok(20)));
        assert!(app.is_scanning, "app should still be scanning because scan_id 2 is obsolete");

        // 5. ScanFinished matching current_scan_id (3) completes the scan
        let _ = app.update(Message::ScanFinished(3, Ok(30)));
        assert!(!app.is_scanning);
        assert!(app.scan_cancel.is_none());
    }

    #[test]
    fn test_remove_folder_while_scanning() {
        use std::sync::atomic::Ordering;

        let (mut app, _) = WazooApp::new(None);
        app.settings.media_folders = vec!["/folder1".to_string(), "/folder2".to_string()];

        // 1. Initial StartScan
        let _ = app.update(Message::StartScan);
        assert!(app.is_scanning);
        assert_eq!(app.current_scan_id, 1);
        let first_cancel = app.scan_cancel.clone().expect("scan_cancel should be set");
        assert!(!first_cancel.load(Ordering::SeqCst));

        // 2. Removing a folder while scanning aborts current scan and starts over with remaining folders
        let _ = app.update(Message::RemoveMediaFolder("/folder1".to_string()));
        assert_eq!(app.settings.media_folders, vec!["/folder2"]);
        assert!(first_cancel.load(Ordering::SeqCst), "first scan cancel flag should be set to true");
        assert!(app.is_scanning);
        assert_eq!(app.current_scan_id, 2);
        let second_cancel = app.scan_cancel.clone().expect("second scan_cancel should be set");
        assert!(!second_cancel.load(Ordering::SeqCst));

        // 3. Removing the last remaining folder aborts current scan and stops scanning
        let _ = app.update(Message::RemoveMediaFolder("/folder2".to_string()));
        assert!(app.settings.media_folders.is_empty());
        assert!(second_cancel.load(Ordering::SeqCst), "second scan cancel flag should be set to true");
        assert!(!app.is_scanning);
        assert!(app.scan_cancel.is_none());
        assert_eq!(app.current_scan_id, 3);

        // 4. Stale ScanFinished from earlier scan (scan_id 1 or 2) is discarded
        let _ = app.update(Message::ScanFinished(1, Ok(10)));
        assert!(!app.is_scanning);
        let _ = app.update(Message::ScanFinished(2, Ok(10)));
        assert!(!app.is_scanning);
    }

    #[test]
    fn test_settings_total_videos_shows_whole_database_total_not_present_query() {
        let (mut app, _) = WazooApp::new(None);
        app.db = wazoo_core::Database::open_in_memory().expect("in-memory db");

        // Insert 5 videos into the database
        for i in 1..=5 {
            let _ = app.db.insert_or_update_video(&VideoRecord {
                id: i,
                name: format!("video_{i}"),
                path: format!("/media/videos/video_{i}.mp4"),
            });
        }
        assert_eq!(app.total_video_count(), 5);

        // Simulate a search query that filters available_videos down to 1
        app.active_search_query = "video_1".to_string();
        app.available_videos = vec![
            VideoRecord { id: 1, name: "video_1".to_string(), path: "/media/videos/video_1.mp4".to_string() },
        ];
        assert_eq!(app.available_videos.len(), 1);

        // Whole database total should still be 5, not the filtered query length (1)
        assert_eq!(app.total_video_count(), 5);

        // Verify settings modal rendering runs without panic
        app.show_settings_modal = true;
        let _ = app.view_settings_modal();
    }

    #[test]
    fn test_scroll_mode_mutual_exclusion_and_single_player_transition() {
        let (mut app, _) = WazooApp::new(None);

        // 1. Setup multi-player (4 players) in Normal mode
        let _ = app.update(Message::SetPlayerCount(4));
        assert_eq!(app.settings.playback_mode, PlaybackMode::Normal);
        assert_eq!(app.settings.player_count, 4);

        // 2. Switch to Scroll mode from multi-player mode
        let _ = app.update(Message::ToggleScrollMode);
        assert_eq!(app.settings.playback_mode, PlaybackMode::Scroll);
        // Scroll engine items should be initialized cleanly without grid duplication
        assert!(!app.scroll_engine.items.is_empty());
        // First item in stack should start at y = 0
        let first_item = app.scroll_engine.items.values().find(|it| it.y_pos == 0.0);
        assert!(first_item.is_some(), "scroll stack must start at y = 0.0");

        // 3. While in scroll mode, pressing 1 (SetPlayerCount(1)) must end scroll mode first
        let _ = app.update(Message::SetPlayerCount(1));
        assert_eq!(app.settings.playback_mode, PlaybackMode::Normal, "pressing 1 must exit scroll mode");
        assert!(app.scroll_engine.items.is_empty(), "scroll engine must be cleared");
        assert_eq!(app.settings.player_count, 1);
        assert_eq!(app.players.len(), 1);

        // 4. Enter scroll mode again and press 3 (SetPlayerCount(3))
        let _ = app.update(Message::ToggleScrollMode);
        assert_eq!(app.settings.playback_mode, PlaybackMode::Scroll);
        assert!(!app.scroll_engine.items.is_empty());

        let _ = app.update(Message::SetPlayerCount(3));
        assert_eq!(app.settings.playback_mode, PlaybackMode::Normal, "pressing 3 must exit scroll mode");
        assert!(app.scroll_engine.items.is_empty(), "scroll engine must be cleared");
        assert_eq!(app.settings.player_count, 3);
        assert_eq!(app.players.len(), 3);
    }

    #[test]
    fn test_select_audio_track_message_and_view() {
        let (mut app, _) = WazooApp::new(None);
        if let Some(player) = app.players.first_mut() {
            // Setup 2 mock audio tracks on the player
            player.state.audio_tracks = vec![
                wazoo_media::AudioTrack {
                    id: 1,
                    title: Some("English Dub / AAC LC".to_string()),
                    lang: Some("eng".to_string()),
                    codec: Some("vorbis".to_string()),
                    is_selected: true,
                },
                wazoo_media::AudioTrack {
                    id: 2,
                    title: None,
                    lang: Some("jpn".to_string()),
                    codec: Some("vorbis".to_string()),
                    is_selected: false,
                },
            ];
            player.state.current_audio_track_id = Some(1);
            let player_id = player.id;

            // Verify view rendering with multiple audio tracks produces valid Element
            app.hovered_player_id = Some(player_id);
            app.player_overlay_ticks = 100;
            let _ = app.view();

            // Toggle audio menu open
            let _ = app.update(Message::ToggleAudioMenu(player_id));
            assert_eq!(app.open_audio_menu_player_id, Some(player_id));

            // When unhovering while audio menu is open, overlay must NOT fade or hide
            let _ = app.update(Message::PlayerUnhovered(player_id));
            assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);

            // Ticking video frames also preserves overlay while menu is open
            let _ = app.update(Message::VideoFrameTick);
            assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);

            // Re-render view with menu open
            let _ = app.view();

            // Send SelectAudioTrack to switch to Japanese (track 2)
            let _ = app.update(Message::SelectAudioTrack(player_id, 2));

            // Verify menu is closed, toast message is shown, and track updated
            assert_eq!(app.open_audio_menu_player_id, None);
            assert!(app.toast_message.is_some());
            assert_eq!(app.toast_message.as_deref(), Some("Audio: Japanese"));
            assert_eq!(app.players[0].state.current_audio_track_id, Some(2));
            assert_eq!(app.player_overlay_ticks, PLAYER_OVERLAY_HIDE_TICKS);
            assert_eq!(app.settings.preferred_audio_language.as_deref(), Some("Japanese"));

            // Re-render view with new track selected
            let _ = app.view();

            // Test Escape closes menu
            let _ = app.update(Message::ToggleAudioMenu(player_id));
            assert_eq!(app.open_audio_menu_player_id, Some(player_id));
            let _ = app.update(Message::EscapePressed);
            assert_eq!(app.open_audio_menu_player_id, None);

            // When only 1 audio track exists, view should also render cleanly without selector
            app.players[0].state.audio_tracks.truncate(1);
            let _ = app.view();
        }
    }
}

