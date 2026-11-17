/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Application Bootstrap & Initial Player Setup
 *
 * Handles CLI file ingestion, initial query resolution, library loading,
 * initial WazooApp state construction, and async initial player creation.
 */

use super::{LONG_TOAST_SECS, WazooApp};
use crate::assets::APP_ICON_BYTES;
use crate::cli::CliArgs;
use crate::format;
use crate::message::Message;
use crate::state::PlayerList;
use iced::Task;
use std::sync::{Arc, Mutex};
use wazoo_core::{ConfigManager, Database, VideoRecord, WazooSettings};
use wazoo_media::{PlayerId, ScrollEngine, StartTime};

pub(super) struct InitialSearch {
    pub(super) folders: Vec<String>,
    pub(super) active_query: String,
    pub(super) selected_folder: String,
    pub(super) is_cli: bool,
}

pub(super) fn process_direct_cli_files(
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

pub(super) fn resolve_initial_search(
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

pub(super) fn load_initial_videos(
    db: &Database,
    active_query: &str,
    folders: &[String],
    include_metadata: bool,
    is_cli: bool,
    has_file_to_play: bool,
) -> Vec<VideoRecord> {
    let mut videos = if !active_query.is_empty() {
        let filtered = db
            .search_videos_with_options(active_query, folders, include_metadata)
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
        db.search_videos("", folders).unwrap_or_default()
    } else {
        db.get_all_videos().unwrap_or_default()
    };

    if videos.is_empty() && has_file_to_play {
        videos = db.get_all_videos().unwrap_or_default();
    }

    videos
}

pub(super) fn build_initial_toast(
    is_cli: bool,
    language: &str,
    total_videos: usize,
) -> (Option<String>, usize) {
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

pub(super) fn boot_focus_task() -> Task<Message> {
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
    pub(super) fn build_initial_app(
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
        let raw_static_frames = crate::assets::load_raw_color_static_frames();
        let (init_w, init_h, init_pix) = if let Some(first) = raw_static_frames.first() {
            (first.width, first.height, first.pixels.to_vec())
        } else {
            (0, 0, Vec::new())
        };
        let static_frame = Arc::new(Mutex::new(wazoo_media::pipeline::FrameData {
            width: init_w,
            height: init_h,
            pixels: init_pix,
            new_frame: true,
            frame_seq: 1,
        }));
        let total_videos = videos.len();
        let metadata_cache = db.get_all_metadata_map().unwrap_or_default();
        let (toast_message, toast_time_remaining) =
            build_initial_toast(search.is_cli, &settings.language, total_videos);

        let theme_palette = crate::theme::ThemePalette::from_settings(&settings.theme);
        let mut modals = crate::state::ModalState::default();
        modals.sync_picker_from_theme(&settings.theme);

        Self {
            settings: settings.clone(),
            theme_palette,
            config_mgr,
            db,
            players: PlayerList::new(),
            scroll_engine,
            available_videos: videos,
            metadata_cache,
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
            modals,
            drawers: crate::state::DrawerState::default(),
            titlebar: crate::state::TitlebarState::default(),
            overlay: crate::state::OverlayState {
                toast_message,
                toast_time_remaining,
                ..Default::default()
            },
            window: crate::state::WindowState::default(),
            app_icon_handle: icon_handle,
            raw_static_frames,
            static_frame,
            last_static_frame_idx: 0,
            last_total_videos: total_videos,
            next_player_id: 1,
            scanner: crate::state::ScannerState::default(),
            plex: crate::state::PlexState::default(),
            focused_idx: 0,
            hovered_player_id: None,
            open_audio_menu_id: None,
            subtitles_enabled: true,
            default_shuffle_mode: true,
            preloaded_player: None,
            is_preloading: false,
            standby_player: Arc::new(Mutex::new(None)),
            cube: crate::state::CubeState {
                speed_multiplier: settings.cube_speed,
                size_multiplier: settings.cube_size,
                ..Default::default()
            },
        }
    }

    pub(super) fn init_initial_players(
        &mut self,
        file_to_play: Option<&str>,
        is_cli: bool,
    ) -> Task<Message> {
        let count = self.settings.player_count.clamp(1, 12);
        let restored_sessions = if is_cli || file_to_play.is_some() {
            Vec::new()
        } else {
            self.settings.session_videos.clone()
        };

        let mut direct_file_player_id: Option<PlayerId> = None;
        let mut tasks: Vec<Task<Message>> = Vec::new();

        if let Some(path) = file_to_play {
            let id = self.next_player_id;
            self.next_player_id += 1;
            direct_file_player_id = Some(id);
            let name = format::format_video_title(path);
            let task = self.push_boot_placeholder_player(
                id,
                path,
                &name,
                Some(0.0),
                false,
                1.0,
                self.default_shuffle_mode,
                true,
            );
            tasks.push(task);
            self.overlay.title_pill_ticks = 240;
        } else {
            for session in restored_sessions.into_iter().take(count) {
                let is_http = session.path.starts_with("http://")
                    || session.path.starts_with("https://");
                let in_library = self.available_videos.iter().any(|v| v.path == session.path);
                if is_http || in_library || std::path::Path::new(&session.path).exists() {
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
                    let will_reconcile = !self.settings.last_query.is_empty() && !in_library;
                    let task = self.push_boot_placeholder_player(
                        id,
                        &session.path,
                        &name,
                        start_secs,
                        session.is_muted,
                        session.volume,
                        session.is_shuffle,
                        !will_reconcile,
                    );
                    tasks.push(task);
                }
            }
        }

        // Fill remaining players if any
        while self.players.len() < count {
            let (added, task) = self.add_player_internal();
            if added.is_none() {
                break;
            }
            tasks.push(task);
        }

        // If a last_query was active, reconcile active players so any player playing a video
        // not in the queried files list switches to a matching video from available_videos.
        // If playing a direct file, do not reconcile that player!
        if !self.settings.last_query.is_empty() {
            tasks.push(self.reconcile_players_with_available_videos(direct_file_player_id));
        }

        self.update_static_frame();

        // Save session state if CLI query or direct file successfully populated players
        if (is_cli || file_to_play.is_some()) && !self.players.is_empty() {
            self.save_session_state();
        }
        Task::batch(tasks)
    }

    #[allow(clippy::too_many_arguments)]
    fn push_boot_placeholder_player(
        &mut self,
        id: PlayerId,
        path: &str,
        name: &str,
        start_secs: Option<f64>,
        is_muted: bool,
        volume: f64,
        shuffle: bool,
        spawn_load: bool,
    ) -> Task<Message> {
        let start_time = StartTime::from(start_secs);
        let mut placeholder = wazoo_media::VideoHandle::new_placeholder(
            id,
            path,
            name,
            self.buffer_config(),
            start_time,
        );
        placeholder.state.is_muted = is_muted;
        placeholder.state.volume = volume;
        let mut player = crate::state::AppPlayer::new(placeholder, shuffle);
        player.flip.reset(self.settings.flip_interval_secs);
        player.load_generation = 1;
        player.start_loading();
        self.players.push(player);
        self.push_player_nav_entry(id, path.to_string(), start_secs);
        if spawn_load {
            self.spawn_player_load_task(id, 1, path.to_string(), name.to_string(), start_time)
        } else {
            Task::none()
        }
    }

    pub(super) fn init_scroll_mode(&mut self) -> Task<Message> {
        let items_with_heights: Vec<(PlayerId, f32)> = self
            .players
            .iter()
            .map(|p| (p.id, self.calculate_player_scroll_height(p)))
            .collect();
        self.scroll_engine
            .init_stack_with_heights(&items_with_heights);
        let mut tasks = Vec::new();
        while let Some(spawn_y) = self.scroll_engine.needs_new_player() {
            let (added, task) = self.add_player_internal();
            if let Some(id) = added {
                tasks.push(task);
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
            p.set_crossfade_volume(vol);
        }
        tasks.push(self.trigger_preload_task());
        Task::batch(tasks)
    }
}
