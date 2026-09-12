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

pub struct WazooApp {
    pub(crate) settings: WazooSettings,
    pub(crate) config_mgr: ConfigManager,
    pub(crate) db: Database,
    pub(crate) players: Vec<VideoHandle>,
    pub(crate) scroll_engine: ScrollEngine,
    pub(crate) available_videos: Vec<VideoRecord>,
    pub(crate) active_search_query: String,
    pub(crate) search_input: String,
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
    pub(crate) show_dropdown_menu: bool,
    pub(crate) is_alt_pressed: bool,
    pub(crate) player_overlay_ticks: usize,
    pub(crate) title_pill_ticks: usize,
    pub(crate) window_id: Option<iced::window::Id>,
    pub(crate) app_icon_handle: iced::widget::image::Handle,
    pub(crate) toast_message: Option<String>,
    pub(crate) toast_time_remaining: usize,
    pub(crate) next_player_id: PlayerId,
    pub(crate) is_scanning: bool,
    pub(crate) scan_progress: Option<ScanProgress>,
    pub(crate) focused_player_idx: usize,
    pub(crate) focus_border_ticks: usize,
    pub(crate) is_shuffle_mode: bool,
    pub(crate) hovered_player_id: Option<PlayerId>,
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

        let toast_msg = if is_cli {
            if videos.is_empty() {
                Some(format!("No videos found for query: \"{}\"", active_query))
            } else {
                Some(format!("Query: \"{}\" ({} videos)", active_query, videos.len()))
            }
        } else if !settings.last_query.is_empty() {
            Some(format!("Restored query: \"{}\" ({} videos)", settings.last_query, videos.len()))
        } else {
            Some("Welcome to Wazoo".to_string())
        };

        let mut app = Self {
            settings: settings.clone(),
            config_mgr,
            db,
            players: Vec::new(),
            scroll_engine,
            available_videos: videos,
            active_search_query: active_query.clone(),
            search_input: active_query,
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
            show_dropdown_menu: false,
            is_alt_pressed: false,
            player_overlay_ticks: 0,
            title_pill_ticks: 0,
            window_id: None,
            app_icon_handle: icon_handle,
            toast_message: toast_msg,
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
            preloaded_player: None,
            is_preloading: false,
            cursor_position: Point::ORIGIN,
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
        format!("Wazoo - Ambient Media Engine ({} videos)", self.available_videos.len())
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
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
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) => {
                    Some(Message::RightClickPressed(window_id))
                }
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                    Some(Message::LeftClickReleased)
                }
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Message::WindowResized(window_id, size))
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
