/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Player Lifecycle, Async Initialization, Standby & Session Persistence
 *
 * Manages player focus lookups, off-thread VideoHandle loading, pre-warmed standby handles,
 * scroll-mode preloading/cleanup, 3D cube player spawning, session saving, and search reconciliation.
 */

use super::WazooApp;
use crate::message::Message;
use crate::state::AppPlayer;
use iced::Task;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wazoo_core::{PlaybackMode, VideoRecord, VideoSession};
use wazoo_media::{BufferConfig, PlayerId, StartTime, VideoHandle};

impl WazooApp {
    /// Returns the `PlayerId` of the currently focused video player, if any player exists.
    pub fn focused_player_id(&self) -> Option<PlayerId> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(self.players[idx].id)
        }
    }

    /// Returns an immutable reference to the currently focused `AppPlayer`, if any player exists.
    pub(crate) fn focused_player(&self) -> Option<&AppPlayer> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(&self.players[idx])
        }
    }

    /// Returns a mutable reference to the currently focused `AppPlayer`, if any player exists.
    pub(crate) fn focused_player_mut(&mut self) -> Option<&mut AppPlayer> {
        if self.players.is_empty() {
            None
        } else {
            let idx = self.focused_idx % self.players.len();
            Some(&mut self.players[idx])
        }
    }

    pub(crate) fn buffer_config(&self) -> BufferConfig {
        let player_count = self.players.len().max(1) as u32;
        let per_player_mb = (self.settings.buffer_size_mb / player_count).clamp(8, 24);
        BufferConfig {
            duration_secs: self.settings.buffer_duration_secs.clamp(2, 5),
            size_mb: per_player_mb,
            read_chunk_kb: 512,
            preferred_audio_language: self.settings.preferred_audio_language.clone(),
            preferred_subtitle_language: self.settings.preferred_subtitle_language.clone(),
            i18n_language: Some(self.settings.language.clone()),
            gamma: self.settings.gamma as f64,
            contrast: self.settings.contrast as f64,
            brightness: self.settings.brightness as f64,
            saturation: self.settings.saturation as f64,
            playback_speed: self.settings.playback_speed as f64,
            crt_enabled: self.settings.filters_enabled && self.settings.filter_crt,
            wavy_enabled: self.settings.filters_enabled && self.settings.filter_wavy,
            fog_enabled: self.settings.filters_enabled && self.settings.filter_fog,
            crt_settings: self.settings.crt,
        }
    }

    /// Pre-warms a standby player in a background worker thread with the next video and its initial
    /// presentation frame already decoded off-thread, so the next player or 3D cube added attaches
    /// in microseconds without freezing the UI thread or hitching active video playback/animations.
    pub(crate) fn replenish_standby_player(&self) {
        if !self.settings.use_standby_player {
            return;
        }

        let expected_start = if self.settings.playback_mode == PlaybackMode::Scroll
            || self.settings.playback_mode == PlaybackMode::Flip
        {
            StartTime::Random
        } else {
            StartTime::Beginning
        };

        if let Ok(guard) = self.standby_player.lock() {
            if let Some(ref h) = *guard {
                if !h.state.path.is_empty() {
                    return;
                }
            }
        }
        let standby = Arc::clone(&self.standby_player);
        let config = self.buffer_config();
        let current_path = self
            .focused_player_id()
            .and_then(|id| self.players.player(id))
            .or_else(|| self.players.first())
            .map(|p| p.state.path.as_str());
        let is_shuffle = self
            .focused_player_id()
            .map(|id| self.is_player_shuffle(id))
            .unwrap_or(self.default_shuffle_mode);
        let video_rec = self.get_next_video_rec_with_mode(current_path, is_shuffle);
        std::thread::Builder::new()
            .name("wazoo-prewarm-player".into())
            .spawn(move || {
                if let Ok(guard) = standby.lock() {
                    if let Some(ref h) = *guard {
                        if !h.state.path.is_empty() {
                            return;
                        }
                    }
                }
                let handle_res = if let Some(ref rec) = video_rec {
                    VideoHandle::with_buffering_and_start(
                        0,
                        &rec.path,
                        &rec.name,
                        config,
                        expected_start,
                    )
                } else {
                    VideoHandle::new_idle(config)
                };

                if let Ok(mut handle) = handle_res {
                    handle.set_muted(true);
                    handle.set_paused(true);

                    // Decode initial presentation frame off-thread so the player is 100% ready
                    // when attached without hitching any animations or existing video streams.
                    let start = std::time::Instant::now();
                    while !handle.has_decoded_frame()
                        && start.elapsed() < Duration::from_millis(1500)
                    {
                        handle.update_frame();
                        if handle.has_decoded_frame() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }

                    if let Ok(mut guard) = standby.lock() {
                        *guard = Some(handle);
                    }
                }
            })
            .ok();
    }

    /// Extracts a pre-warmed standby player handle from memory if available and fully initialized with
    /// a video stream and decoded presentation frame, automatically triggering a background replenishment.
    pub(crate) fn take_ready_standby_player(&self) -> Option<VideoHandle> {
        self.take_standby_player_if(|_| true)
    }

    /// Extracts a pre-warmed standby player handle satisfying a custom predicate (e.g. not matching
    /// the currently playing video, or passing file-picker search filters), automatically triggering replenishment.
    pub(crate) fn take_standby_player_if(
        &self,
        predicate: impl FnOnce(&VideoHandle) -> bool,
    ) -> Option<VideoHandle> {
        if !self.settings.use_standby_player {
            return None;
        }
        let handle = self.standby_player.lock().ok().and_then(|mut g| {
            if let Some(ref h) = *g {
                if !h.state.path.is_empty() && predicate(h) {
                    return g.take();
                }
            }
            None
        })?;
        self.replenish_standby_player();
        Some(handle)
    }

    /// Spawns background libmpv initialization for a player so the UI thread and loading static
    /// animation never stall while opening or probing streams over slow storage or WiFi.
    pub(crate) fn spawn_player_load_task(
        &self,
        id: PlayerId,
        generation: u64,
        path: String,
        name: String,
        start_time: StartTime,
    ) -> Task<Message> {
        let buffer_config = self.buffer_config();
        let holder = Arc::new(Mutex::new(None));
        let holder_clone = Arc::clone(&holder);

        Task::perform(
            async move {
                let res = tokio::task::spawn_blocking(move || {
                    VideoHandle::with_buffering_and_start(
                        id,
                        &path,
                        &name,
                        buffer_config,
                        start_time,
                    )
                })
                .await;
                let final_res = match res {
                    Ok(inner) => inner,
                    Err(e) => Err(format!("Player load task join error: {e}")),
                };
                if let Ok(mut g) = holder_clone.lock() {
                    *g = Some(final_res);
                }
            },
            move |_| Message::PlayerHandleReady {
                player_id: id,
                generation,
                result: holder,
            },
        )
    }

    /// Transitions an existing player into loading state with a zero-blocking placeholder handle
    /// and schedules off-thread libmpv initialization for the target media path.
    pub(crate) fn start_async_player_load(
        &mut self,
        id: PlayerId,
        path: &str,
        name: &str,
        start_time: impl Into<StartTime>,
    ) -> Task<Message> {
        let start_time = start_time.into();
        let config = self.buffer_config();
        let mut generation = 0;

        if let Some(p) = self.players.player_mut(id) {
            p.cancel_seek_debounce();
            p.start_loading();
            p.load_generation = p.load_generation.wrapping_add(1);
            generation = p.load_generation;

            let prev_muted = p.state.is_muted;
            let prev_volume = p.state.volume;
            let prev_crossfade = p.handle.crossfade_volume();

            let mut placeholder = VideoHandle::new_placeholder(id, path, name, config, start_time);
            placeholder.state.is_muted = prev_muted;
            placeholder.state.volume = prev_volume;
            placeholder.set_crossfade_volume(prev_crossfade);
            p.handle = placeholder;
        }
        self.update_static_frame();
        self.spawn_player_load_task(id, generation, path.to_string(), name.to_string(), start_time)
    }

    pub(crate) fn add_player_internal(&mut self) -> (Option<PlayerId>, Task<Message>) {
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

        // If standby player is pre-warmed with a video and decoded off-thread, attach it instantly
        if let Some(mut handle) = self.take_ready_standby_player() {
            handle.id = id;
            handle.state.id = id;
            handle.set_muted(initial_muted);
            handle.set_subtitles_visible(self.subtitles_enabled);
            handle.set_paused(false);
            let path = handle.state.path.clone();
            let mut player = AppPlayer::new(handle, self.default_shuffle_mode);
            player.flip.reset(self.settings.flip_interval_secs);
            self.push_player_nav_entry(id, path, None);
            self.players.push(player);
            return (Some(id), Task::none());
        }

        if let Some(video_rec) = self.get_next_video_rec(None) {
            let task = self.spawn_placeholder_player(
                id,
                video_rec,
                start_time,
                initial_muted,
                false,
            );
            self.update_static_frame();
            return (Some(id), task);
        }
        (None, Task::none())
    }

    /// Spawns an independent video player dedicated to the 3D cube.
    pub(crate) fn spawn_cube_player(&mut self) -> (Option<PlayerId>, Task<Message>) {
        let id = self.next_player_id;
        self.next_player_id += 1;

        // Cube player starts muted by default so it doesn't clash with existing audio
        let initial_muted = true;
        let start_time = StartTime::Random;

        // 1. If standby player is pre-warmed in memory, adopt it immediately and seek to random
        if let Some(mut handle) = self.take_ready_standby_player() {
            handle.id = id;
            handle.state.id = id;
            handle.set_muted(initial_muted);
            handle.set_subtitles_visible(self.subtitles_enabled);
            handle.set_paused(false);
            handle.seek_random();
            let path = handle.state.path.clone();
            let mut player = AppPlayer::new_cube(handle, self.default_shuffle_mode);
            player.flip.reset(self.settings.flip_interval_secs);
            self.push_player_nav_entry(id, path, None);
            self.players.push(player);
            return (Some(id), Task::none());
        }

        // 2. Otherwise attach a zero-blocking placeholder and initialize libmpv off-thread
        if let Some(video_rec) = self.get_next_video_rec(None) {
            let task = self.spawn_placeholder_player(
                id,
                video_rec,
                start_time,
                initial_muted,
                true,
            );
            return (Some(id), task);
        }
        (None, Task::none())
    }

    fn spawn_placeholder_player(
        &mut self,
        id: PlayerId,
        video_rec: VideoRecord,
        start_time: StartTime,
        initial_muted: bool,
        is_cube: bool,
    ) -> Task<Message> {
        let mut placeholder = VideoHandle::new_placeholder(
            id,
            &video_rec.path,
            &video_rec.name,
            self.buffer_config(),
            start_time,
        );
        placeholder.state.is_muted = initial_muted;
        let mut player = if is_cube {
            AppPlayer::new_cube(placeholder, self.default_shuffle_mode)
        } else {
            AppPlayer::new(placeholder, self.default_shuffle_mode)
        };
        player.flip.reset(self.settings.flip_interval_secs);
        player.load_generation = 1;
        player.start_loading();
        self.push_player_nav_entry(id, video_rec.path.clone(), None);
        self.players.push(player);
        self.spawn_player_load_task(id, 1, video_rec.path, video_rec.name, start_time)
    }

    /// Cleans up and stops all independent 3D cube players.
    pub(crate) fn cleanup_cube_players(&mut self) {
        self.players.retain_mut(|p| {
            if p.is_cube {
                p.stop();
                false
            } else {
                true
            }
        });
        if self.focused_idx >= self.players.len() && !self.players.is_empty() {
            self.focused_idx = self.players.len() - 1;
        }
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
        // Do NOT overwrite normal grid session_videos while running in Scroll Mode.
        // Scroll mode dynamically spawns and despawns transient feed items.
        if self.settings.playback_mode != PlaybackMode::Scroll {
            let normal_players: Vec<&AppPlayer> =
                self.players.iter().filter(|p| !p.is_cube).collect();
            if !normal_players.is_empty() {
                let sessions: Vec<VideoSession> = normal_players
                    .into_iter()
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
        }
        self.settings.cube_speed = self.cube.speed_multiplier;
        self.settings.cube_size = self.cube.size_multiplier;
        self.settings.last_query = self.search.active_query.clone();
        self.settings.last_folders = self.search.active_folders.clone();
        let mut settings_to_save = self.settings.clone();
        if self.cube.desktop_overlay {
            if let Some(ref prev) = self.cube.pre_desktop_bounds {
                settings_to_save.window_bounds = prev.clone();
            }
        }
        let _ = self.config_mgr.save_settings(&settings_to_save);
        self.window.bounds_dirty = false;
    }

    /// Reconciles active players against `self.available_videos`.
    /// Any player currently playing a file that does NOT exist in `self.available_videos`
    /// is switched to play a video that DOES exist in `self.available_videos`.
    /// Each switching player is assigned a distinct video when possible.
    pub fn reconcile_players_with_available_videos(
        &mut self,
        skip_player_id: Option<PlayerId>,
    ) -> Task<Message> {
        if self.available_videos.is_empty() {
            return Task::none();
        }

        if self.players.is_empty() {
            let target_count = self.settings.player_count.clamp(1, 12);
            let mut tasks = Vec::new();
            for _ in 0..target_count {
                let (_, task) = self.add_player_internal();
                tasks.push(task);
            }
            return Task::batch(tasks);
        }

        let needs_switch: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|p| skip_player_id != Some(p.id))
            .filter(|p| !self.available_videos.iter().any(|v| v.path == p.state.path))
            .map(|p| p.id)
            .collect();

        if needs_switch.is_empty() {
            return Task::none();
        }

        // Track paths already being played by players that are NOT switching
        let mut used_paths: HashSet<String> =
            self.players.iter().map(|p| p.state.path.clone()).collect();
        for &id in &needs_switch {
            if let Some(p) = self.players.player(id) {
                used_paths.remove(&p.state.path);
            }
        }

        let mut tasks = Vec::new();
        for (idx, id) in needs_switch.into_iter().enumerate() {
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
                    p.load_retry_count = 0;
                }
                let task =
                    self.start_async_player_load(id, &rec_path, &rec_name, StartTime::Beginning);
                self.push_player_nav_entry(id, rec_path, None);
                tasks.push(task);
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
        Task::batch(tasks)
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
        let video_rec = match self
            .get_next_video_rec_for_navigation(curr_path.as_deref(), self.default_shuffle_mode)
        {
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
                        handle.update_frame();
                        if handle.has_decoded_frame() {
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

    /// Resets scroll engine and transitions playback mode back to Normal
    pub(crate) fn cleanup_scroll_mode(&mut self) {
        self.settings.playback_mode = PlaybackMode::Normal;
        self.scroll_engine.clear();
        if let Some(handle) = self.preloaded_player.take() {
            std::thread::spawn(move || drop(handle));
        }
        self.is_preloading = false;
        for p in &mut self.players {
            if !p.is_cube {
                p.reset_crossfade_volume();
            }
        }
        if self.settings.scroll_mode_muted {
            for p in &mut self.players {
                if !p.is_cube {
                    p.set_muted(true);
                }
            }
        } else {
            let has_focused_grid = self
                .players
                .get(self.focused_idx)
                .map(|p| !p.is_cube)
                .unwrap_or(false);
            let mut unmuted_one = false;
            for (i, p) in self.players.iter_mut().enumerate() {
                if !p.is_cube {
                    if (has_focused_grid && i == self.focused_idx)
                        || (!has_focused_grid && !unmuted_one)
                    {
                        p.set_muted(false);
                        unmuted_one = true;
                    } else {
                        p.set_muted(true);
                    }
                }
            }
        }
    }

    /// Formats concise playing info (header, file path, and Plex metadata) for copying to clipboard.
    pub fn format_player_copy_info(&self, id: PlayerId) -> Option<String> {
        let p = self.players.player(id)?;
        let path = p.state.path.trim();
        if path.is_empty() {
            return None;
        }

        let meta = self
            .get_metadata_for_path(path)
            .cloned()
            .or_else(|| self.db.get_metadata_by_path(path).ok().flatten());

        let header_title = format_copy_header_title(path, &p.state.name, meta.as_ref());
        let mut lines = vec![header_title, format!("File: {path}")];

        if let Some(ref m) = meta {
            append_plex_metadata_lines(&mut lines, m);
        }

        Some(lines.join("\n"))
    }
}

fn format_copy_header_title(
    path: &str,
    fallback_name: &str,
    meta: Option<&wazoo_core::MediaMetadata>,
) -> String {
    if let Some(m) = meta {
        if let Some(ref st) = m.series_title {
            let series = crate::format::clean_metadata_title(st);
            if !series.is_empty() {
                let ep_title = crate::format::clean_metadata_title(&m.title);
                return match (m.season_number, m.episode_number) {
                    (Some(sn), Some(en)) if !ep_title.is_empty() => {
                        format!("{series} — S{sn:02}E{en:02} - {ep_title}")
                    }
                    (Some(sn), Some(en)) => format!("{series} — S{sn:02}E{en:02}"),
                    _ if !ep_title.is_empty() => format!("{series} — {ep_title}"),
                    _ => series,
                };
            }
        }

        let title = crate::format::clean_metadata_title(&m.title);
        if !title.is_empty() {
            if let Some(year) = m.year {
                let year_tag = format!("({year})");
                if !title.contains(&year_tag) {
                    return format!("{title} {year_tag}");
                }
            }
            return title;
        }
    }

    let desc = crate::format::format_descriptive_title(path);
    if !desc.trim().is_empty() {
        desc
    } else if !fallback_name.trim().is_empty() {
        fallback_name.trim().to_string()
    } else {
        path.to_string()
    }
}

fn append_plex_metadata_lines(lines: &mut Vec<String>, m: &wazoo_core::MediaMetadata) {
    let mut meta_lines = Vec::new();

    if !m.title.trim().is_empty() {
        meta_lines.push(format!("Title: {}", m.title.trim()));
    }
    if let Some(ref series) = m.series_title {
        if !series.trim().is_empty() {
            meta_lines.push(format!("Series: {}", series.trim()));
        }
    }
    match (m.season_number, m.episode_number) {
        (Some(sn), Some(en)) => meta_lines.push(format!("Episode: S{sn:02}E{en:02}")),
        (Some(sn), None) => meta_lines.push(format!("Season: {sn}")),
        (None, Some(en)) => meta_lines.push(format!("Episode: {en}")),
        (None, None) => {}
    }
    if let Some(year) = m.year {
        meta_lines.push(format!("Year: {year}"));
    }
    if let Some(duration_ms) = m.duration_ms {
        if duration_ms > 0 {
            meta_lines.push(format!(
                "Duration: {}",
                crate::format::format_time_str((duration_ms as f64) / 1000.0)
            ));
        }
    }
    if let Some(ref studio) = m.studio {
        if !studio.trim().is_empty() {
            meta_lines.push(format!("Studio: {}", studio.trim()));
        }
    }
    if !m.genres.is_empty() {
        meta_lines.push(format!("Genres: {}", m.genres.join(", ")));
    }
    if !m.directors.is_empty() {
        meta_lines.push(format!("Directors: {}", m.directors.join(", ")));
    }
    if !m.actors.is_empty() {
        meta_lines.push(format!("Cast: {}", m.actors.join(", ")));
    }
    if let Some(ref tagline) = m.tagline {
        if !tagline.trim().is_empty() {
            meta_lines.push(format!("Tagline: {}", tagline.trim()));
        }
    }
    if let Some(ref summary) = m.summary {
        if !summary.trim().is_empty() {
            meta_lines.push(format!("Summary: {}", summary.trim()));
        }
    }

    if !meta_lines.is_empty() {
        lines.push(String::new());
        lines.extend(meta_lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_placeholder_player(id: PlayerId, path: &str, name: &str) -> AppPlayer {
        let handle = VideoHandle::new_placeholder(
            id,
            path,
            name,
            BufferConfig::default(),
            StartTime::Beginning,
        );
        AppPlayer::new(handle, true)
    }

    #[test]
    fn copy_info_returns_none_for_missing_or_empty_player() {
        let (mut app, _) = WazooApp::new_test_app();
        assert!(app.format_player_copy_info(999).is_none());

        app.players.push(make_placeholder_player(1, "", ""));
        assert!(app.format_player_copy_info(1).is_none());
    }
}
