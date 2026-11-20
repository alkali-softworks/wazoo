/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Bookmark CRUD & Jump-to-Bookmark Handlers
 */

use crate::app::{DEFAULT_TOAST_SECS, LONG_TOAST_SECS, WazooApp};
use crate::format;
use crate::message::Message;
use crate::state::AppPlayer;
use iced::Task;
use wazoo_core::Bookmark;

impl WazooApp {
    pub(super) fn handle_add_bookmark_focused(&mut self) -> Task<Message> {
        if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player(id) {
                let path = p.path().to_string();
                if !path.is_empty() {
                    let name = format::format_descriptive_title(&path);
                    let query = self.search.active_query.clone();
                    let position_secs = p.position().as_secs_f64();
                    let is_shuffle = self.is_player_shuffle(id);

                    let msg = if let Some(existing) =
                        self.settings.bookmarks.iter_mut().find(|b| b.path == path)
                    {
                        existing.name = name.clone();
                        existing.query = query;
                        existing.position_secs = position_secs;
                        existing.is_shuffle = is_shuffle;
                        self.t_with("bookmarks.updated", &[("name", &name)])
                    } else {
                        self.settings.bookmarks.push(Bookmark {
                            name: name.clone(),
                            query,
                            path,
                            position_secs,
                            is_shuffle,
                        });
                        self.t_with("bookmarks.added", &[("name", &name)])
                    };
                    let _ = self.config_mgr.save_settings(&self.settings);
                    self.show_toast_for(msg, LONG_TOAST_SECS);
                }
            }
        }
        Task::none()
    }

    pub(super) fn handle_remove_bookmark_focused(&mut self) -> Task<Message> {
        if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player(id) {
                let path = p.path();
                if let Some(pos) = self.settings.bookmarks.iter().position(|b| b.path == path) {
                    let removed = self.settings.bookmarks.remove(pos);
                    let _ = self.config_mgr.save_settings(&self.settings);
                    let msg = self.t_with("bookmarks.removed", &[("name", &removed.name)]);
                    self.show_toast_for(msg, LONG_TOAST_SECS);
                } else {
                    self.show_toast_for(self.t("bookmarks.not_found"), DEFAULT_TOAST_SECS);
                }
            }
        }
        Task::none()
    }

    pub(super) fn handle_remove_bookmark(&mut self, idx: usize) -> Task<Message> {
        if idx < self.settings.bookmarks.len() {
            let removed = self.settings.bookmarks.remove(idx);
            let _ = self.config_mgr.save_settings(&self.settings);
            let msg = self.t_with("bookmarks.removed", &[("name", &removed.name)]);
            self.show_toast_for(msg, DEFAULT_TOAST_SECS);
        }
        Task::none()
    }

    pub(super) fn handle_jump_to_bookmark(&mut self, b: Bookmark) -> Task<Message> {
        if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player_mut(id) {
                p.flip.reset(self.settings.flip_interval_secs);
            }
        }
        // 1. Restore shuffle vs linear mode
        self.default_shuffle_mode = b.is_shuffle;
        if let Some(p) = self.focused_player_mut() {
            p.shuffle = b.is_shuffle;
        }

        // 2. Update the global search query to match the bookmark's query
        self.search.active_query = b.query.clone();
        self.search.tags = b
            .query
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.search.input.clear();
        self.settings.last_query = b.query.clone();
        self.settings.last_folders = self.search.active_folders.clone();
        let _ = self.config_mgr.save_settings(&self.settings);

        // 3. Query the database using the updated search query
        let folders = self.search.active_folders.clone();

        if let Ok(results) = self.db.search_videos_with_options(
            &self.search.active_query,
            &folders,
            self.settings.search_include_metadata,
        ) {
            self.last_total_videos = results.len();
            self.available_videos = results;
            self.drawers.file_picker_entries.clear();
            self.apply_file_picker_search();
        }

        // 4. Load the bookmarked video and position in the focused player
        let mut tasks = Vec::new();
        let focused_id = if let Some(id) = self.focused_player_id() {
            if let Some(p) = self.players.player_mut(id) {
                p.nav_history.forward_stack.clear();
                p.load_retry_count = 0;
            }
            self.record_current_player_nav_position(id);
            let title = format::format_video_title(&b.path);
            tasks.push(self.start_async_player_load(
                id,
                &b.path,
                &title,
                Some(b.position_secs),
            ));
            self.push_player_nav_entry(id, b.path.clone(), Some(b.position_secs));
            Some(id)
        } else {
            let id = self.next_player_id;
            self.next_player_id += 1;
            let title = format::format_video_title(&b.path);
            let start_time = wazoo_media::StartTime::from(Some(b.position_secs));
            let mut placeholder = wazoo_media::VideoHandle::new_placeholder(
                id,
                &b.path,
                &title,
                self.buffer_config(),
                start_time,
            );
            placeholder.state.is_muted = false;
            let mut player = AppPlayer::new(placeholder, b.is_shuffle);
            player.load_generation = 1;
            player.start_loading();
            self.push_player_nav_entry(id, b.path.clone(), Some(b.position_secs));
            self.players.push(player);
            self.focused_idx = 0;
            self.update_static_frame();
            tasks.push(self.spawn_player_load_task(id, 1, b.path.clone(), title, start_time));
            Some(id)
        };

        // 5. Reconcile other players if what they are playing does not exist in the new queried list of files
        tasks.push(self.reconcile_players_with_available_videos(focused_id));
        self.save_session_state();

        let mode_str = if b.is_shuffle {
            self.t("bookmarks.shuffle")
        } else {
            self.t("bookmarks.linear")
        };
        let time_str = format::format_time_str(b.position_secs);
        let msg = self.t_with(
            "bookmarks.jumped",
            &[("name", &b.name), ("time", &time_str), ("mode", &mode_str)],
        );
        self.show_toast_for(msg, LONG_TOAST_SECS);
        self.modals.bookmarks = false;
        if self.drawers.show_transcript {
            tasks.push(self.load_transcript_for_focused_player());
        }
        Task::batch(tasks)
    }
}
