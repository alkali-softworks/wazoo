/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * State Reducer & Event Dispatcher
 * 
 * Implements WazooApp::update, processing incoming Message events, executing playback commands,
 * managing modal dialogs, handling window events, and orchestrating asynchronous tasks.
 */

use std::time::{Duration, Instant};
use iced::futures::SinkExt;
use iced::{
    keyboard::{key::Named, Key},
    Point, Task,
};
use wazoo_core::{Bookmark, LayoutMode, PlaybackMode};
use wazoo_media::{PlayerId, VideoHandle};
use wazoo_scanner::{ScanStage, Scanner};
use crate::app::{WazooApp, PLAYER_OVERLAY_FADE_TICKS, PLAYER_OVERLAY_HIDE_TICKS, TITLEBAR_HIDE_TICKS, TITLEBAR_SHOW_DELAY_TICKS};
use crate::format;
use crate::message::Message;

impl WazooApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowIdReceived(id) => {
                self.window_id = Some(id);
                return iced::window::gain_focus(id);
            }
            Message::WindowFocused => {
                self.is_alt_pressed = false;
            }
            Message::WindowUnfocused => {
                self.is_alt_pressed = false;
                self.cursor_position = Point::new(-1000.0, -1000.0);
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
                self.titlebar_hover_ticks = 0;
                if self.hovered_player_id.is_some() {
                    self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                }
            }
            Message::ModifiersChanged(modifiers) => {
                self.is_alt_pressed = modifiers.alt();
            }
            Message::GainWindowFocus => {
                if let Some(id) = self.window_id {
                    return iced::window::gain_focus(id);
                } else {
                    return iced::window::oldest().then(|maybe_id| {
                        if let Some(id) = maybe_id {
                            iced::window::gain_focus(id)
                        } else {
                            Task::none()
                        }
                    });
                }
            }
            Message::PreloadedPlayerReady(holder) => {
                self.is_preloading = false;
                if self.settings.playback_mode != PlaybackMode::Scroll {
                    return Task::none();
                }

                let result = match holder.lock().ok().and_then(|mut g| g.take()) {
                    Some(r) => r,
                    None => return Task::none(),
                };

                match result {
                    Ok(mut handle) => {
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        handle.set_muted(self.settings.is_global_muted);
                        let item_h = self.scroll_engine.default_item_height();
                        // If scroll stream needs a player right now, attach it immediately!
                        if let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(item_h * 0.5) {
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.players.push(handle);
                            return self.trigger_preload_task();
                        } else {
                            self.preloaded_player = Some(handle);
                        }
                    }
                    Err(err) => {
                        log::error!("Background player preload failed: {err}");
                        return self.trigger_preload_task();
                    }
                }
            }
            Message::WindowResized(id, size) => {
                self.window_id = Some(id);
                self.settings.window_bounds.width = size.width as u32;
                self.settings.window_bounds.height = size.height as u32;
                self.scroll_engine.set_window_height(size.height);
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    let item_h = self.scroll_engine.default_item_height();
                    for item in self.scroll_engine.items.values_mut() {
                        item.height = item_h;
                    }
                    self.scroll_engine.recalculate_positions();
                    while let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(item_h * 0.5) {
                        if let Some(mut handle) = self.preloaded_player.take() {
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.players.push(handle);
                        } else {
                            break;
                        }
                    }
                    return self.trigger_preload_task();
                }
            }
            Message::CursorMoved(win_id, pos) => {
                self.window_id = Some(win_id);
                self.cursor_position = pos;

                if self.show_dropdown_menu {
                    self.show_titlebar = true;
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.titlebar_hover_ticks = 0;
                    if self.hovered_player_id.is_some() {
                        self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                    }
                } else if self.is_point_in_titlebar(pos) {
                    if self.show_titlebar {
                        self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                        if self.hovered_player_id.is_some() {
                            self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                        }
                    }
                } else {
                    self.titlebar_hover_ticks = 0;
                }

                if self.titlebar_drag_pending {
                    if let Some(origin) = self.titlebar_press_origin {
                        let dist = (pos.x - origin.x).hypot(pos.y - origin.y);
                        if dist > 5.0 {
                            self.titlebar_drag_pending = false;
                            self.titlebar_press_origin = None;
                            self.last_titlebar_click = None;
                            return iced::window::drag(win_id);
                        }
                    }
                }
            }
            Message::CursorLeft => {
                self.cursor_position = Point::new(-1000.0, -1000.0);
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
                self.titlebar_hover_ticks = 0;
                if self.hovered_player_id.is_some() {
                    self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                }
            }
            Message::TitleBarPressed => {
                let now = Instant::now();
                if let Some(last_click) = self.last_titlebar_click {
                    if now.duration_since(last_click) < Duration::from_millis(400) {
                        self.last_titlebar_click = None;
                        self.titlebar_drag_pending = false;
                        self.titlebar_press_origin = None;
                        return self.update(Message::MaximizeWindow);
                    }
                }
                self.last_titlebar_click = Some(now);
                self.titlebar_drag_pending = true;
                self.titlebar_press_origin = Some(self.cursor_position);
            }
            Message::LeftClickReleased => {
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
            }
            Message::RightClickPressed(win_id) => {
                self.window_id = Some(win_id);
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
            }
            Message::KeyPressed(key, status) => {
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

                // If typing in either search input (captured by widget) or if search modal or any modal is open,
                // do NOT allow keystrokes to trigger global shortcuts (e.g. 'm' for mute, 's' for shuffle, etc.)
                if status == iced::event::Status::Captured || self.is_any_modal_open() {
                    if self.show_search_modal {
                        if key == Key::Named(Named::Enter) {
                            return self.update(Message::PerformSearch);
                        }
                        if key == Key::Named(Named::Backspace) && self.search_input.is_empty() {
                            self.search_tags.pop();
                            return iced::widget::operation::focus("search_input");
                        }
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
                    Key::Character(s) => match s.as_str() {
                        "b" | "B" => return self.update(Message::ToggleBookmarksModal),
                        "r" | "R" => return self.update(Message::RandomSeekFocused),
                        "t" | "T" => return self.update(Message::ShowTitleOverlay),
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
                        "-" => {
                            if self.settings.playback_mode == PlaybackMode::Scroll {
                                return self.update(Message::AdjustScrollSpeed(-0.1));
                            } else {
                                return self.update(Message::RemoveBookmarkFocused);
                            }
                        }
                        "+" | "=" => {
                            if self.settings.playback_mode == PlaybackMode::Scroll {
                                return self.update(Message::AdjustScrollSpeed(0.1));
                            } else {
                                return self.update(Message::AddBookmarkFocused);
                            }
                        }
                        "c" | "C" => return self.update(Message::ToggleSubtitles),
                        "v" | "V" => return self.update(Message::ToggleTranscript),
                        "h" | "H" => return self.update(Message::ToggleFilePicker),
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
                self.last_titlebar_click = None;
                self.titlebar_drag_pending = false;
                self.titlebar_press_origin = None;
                if let Some(id) = self.window_id {
                    return iced::window::toggle_maximize(id);
                }
            }
            Message::CloseApp => {
                self.save_session_state();
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
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    self.hovered_player_id = None;
                    self.player_overlay_ticks = 0;
                } else if !self.is_point_in_titlebar(self.cursor_position) {
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                }
            }
            Message::CloseDropdownMenu => {
                self.show_dropdown_menu = false;
                if !self.is_point_in_titlebar(self.cursor_position) {
                    self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                }
            }
            Message::OpenMenuModal => {
                self.show_menu_modal = true;
                self.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
            }
            Message::CloseMenuModal => {
                self.show_menu_modal = false;
            }
            Message::ToggleFilePicker => {
                self.show_file_picker = !self.show_file_picker;
                if self.show_file_picker {
                    self.show_transcript = false;
                }
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
            }
            Message::ToggleTranscript => {
                self.show_transcript = !self.show_transcript;
                if !self.show_transcript {
                    self.show_transcript_menu = false;
                }
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                if self.show_transcript {
                    self.show_file_picker = false;
                    return self.load_transcript_for_focused_player();
                }
            }
            Message::ToggleTranscriptForPlayer(id) => {
                let is_same_focused = self.focused_player_id() == Some(id);
                if is_same_focused && self.show_transcript {
                    self.show_transcript = false;
                    self.show_transcript_menu = false;
                    return Task::none();
                }
                if let Some(idx) = self.players.iter().position(|p| p.id == id) {
                    self.focused_player_idx = idx;
                }
                self.show_transcript = true;
                self.show_transcript_menu = false;
                self.show_file_picker = false;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                return self.load_transcript_for_focused_player();
            }
            Message::CloseTranscript => {
                self.show_transcript = false;
                self.show_transcript_menu = false;
            }
            Message::TranscriptSearchChanged(s) => {
                self.transcript_search = s;
            }
            Message::TranscriptLoaded(path, cues) => {
                if self.transcript_video_path.as_deref() == Some(&path) {
                    self.transcript_loading = false;
                    self.transcript_cues = cues;
                }
            }
            Message::SeekToSubtitle(secs) => {
                if let Some(id) = self.focused_player_id() {
                    // Offset by +10ms so playback starts cleanly inside the target cue,
                    // avoiding boundary collision with the preceding cue.
                    return self.update(Message::Seek(id, Duration::from_secs_f64((secs + 0.01).max(0.0))));
                }
            }
            Message::ToggleTranscriptSubtitleMenu => {
                self.show_transcript_menu = !self.show_transcript_menu;
            }
            Message::CloseTranscriptSubtitleMenu => {
                self.show_transcript_menu = false;
            }
            Message::SelectTranscriptSubtitleTrack(track_idx, track_id) => {
                self.show_transcript_menu = false;
                self.transcript_track_index = track_idx;
                if let Some(player) = self.focused_player_mut() {
                    player.set_subtitle_track(track_id);
                    let sub_track = player.subtitle_tracks().get(track_idx).cloned();
                    let path = player.state.path.clone();
                    if !path.is_empty() {
                        self.transcript_video_path = Some(path.clone());
                        self.transcript_loading = true;
                        self.transcript_cues.clear();
                        let path_clone = path.clone();
                        return Task::perform(
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
                                    let stream_idx = track.ff_index.map(|i| i as usize).unwrap_or(track_idx);
                                    wazoo_media::load_subtitles_for_track(path, stream_idx).await
                                } else {
                                    wazoo_media::load_subtitles_for_track(path, track_idx).await
                                }
                            },
                            move |cues| Message::TranscriptLoaded(path_clone, cues),
                        );
                    }
                }
            }
            Message::ToggleFolderCollapse(folder) => {
                if self.expanded_folders.contains(&folder) {
                    self.expanded_folders.remove(&folder);
                } else {
                    self.expanded_folders.insert(folder);
                }
            }
            Message::FilePickerSearchChanged(s) => {
                let trimmed = s.trim().to_lowercase();
                if !trimmed.is_empty() && self.file_picker_search.trim().is_empty() {
                    for v in &self.available_videos {
                        let title = format::format_video_title(&v.path).to_lowercase();
                        if title.contains(&trimmed) {
                            let folder = format::format_video_folder(&v.path);
                            let folder_key = if folder.is_empty() { "Other".to_string() } else { folder };
                            self.expanded_folders.insert(folder_key);
                        }
                    }
                } else if trimmed.is_empty() {
                    self.expanded_folders.clear();
                }
                self.file_picker_search = s;
            }
            Message::PlayFileInFocused(path) => {
                if let Some(id) = self.focused_player_id() {
                    self.loading_player_ids.insert(id);
                    self.loading_player_ticks.insert(id, 0);
                    let title = format::format_video_title(&path);
                    let curr_player = self.players.iter().find(|p| p.id == id);
                    let prev_muted = curr_player.map(|p| p.state.is_muted);
                    let prev_volume = curr_player.map(|p| p.state.volume);

                    if let Ok(mut handle) = self.create_video_handle(id, &path, &title) {
                        handle.set_muted(prev_muted.unwrap_or(self.settings.is_global_muted));
                        if let Some(vol) = prev_volume {
                            handle.set_volume(vol);
                        }
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                            *p = handle;
                        }
                        self.toast_message = Some(format!("Playing: {title}"));
                        self.toast_time_remaining = 3;
                        if self.show_transcript {
                            return self.load_transcript_for_focused_player();
                        }
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
            Message::SetLanguage(lang) => {
                self.settings.language = lang;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::PlayerClicked(id) => {
                if self.open_audio_menu_player_id.is_some() {
                    self.open_audio_menu_player_id = None;
                }
                if self.is_modal_or_menu_open() {
                    return Task::none();
                }
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                        if self.show_transcript {
                            return self.load_transcript_for_focused_player();
                        }
                    }
                    self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
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
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
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
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
                let curr_player = self.players.iter().find(|p| p.id == id);
                let curr_path = curr_player.map(|p| p.state.path.clone());
                let prev_muted = curr_player.map(|p| p.state.is_muted);
                let prev_volume = curr_player.map(|p| p.state.volume);

                for _ in 0..3 {
                    if let Some(video_rec) = self.get_next_video_rec(curr_path.as_deref()) {
                        if let Ok(mut new_handle) = self.create_video_handle(id, &video_rec.path, &video_rec.name) {
                            new_handle.set_muted(prev_muted.unwrap_or(self.settings.is_global_muted));
                            if let Some(vol) = prev_volume {
                                new_handle.set_volume(vol);
                            }
                            new_handle.set_subtitles_visible(self.subtitles_enabled);
                            if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                                *p = new_handle;
                            }
                            break;
                        }
                    }
                }
                if self.show_transcript && self.focused_player_id() == Some(id) {
                    return self.load_transcript_for_focused_player();
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
                    let curr_player = self.players.iter().find(|p| p.id == id);
                    let curr_path = curr_player.map(|p| p.state.path.clone());
                    let prev_muted = curr_player.map(|p| p.state.is_muted);
                    let prev_volume = curr_player.map(|p| p.state.volume);

                    for _ in 0..3 {
                        if let Some(prev_rec) = self.get_prev_video_rec(curr_path.as_deref()) {
                            if let Ok(mut new_handle) = self.create_video_handle(id, &prev_rec.path, &prev_rec.name) {
                                new_handle.set_muted(prev_muted.unwrap_or(self.settings.is_global_muted));
                                if let Some(vol) = prev_volume {
                                    new_handle.set_volume(vol);
                                }
                                new_handle.set_subtitles_visible(self.subtitles_enabled);
                                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                                    *p = new_handle;
                                }
                                break;
                            }
                        }
                    }
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
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
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
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
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
            }
            Message::PlayerHovered(id) => {
                if self.is_modal_or_menu_open() || (self.show_titlebar && self.is_point_in_titlebar(self.cursor_position)) {
                    if self.hovered_player_id == Some(id) {
                        self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                    }
                    return Task::none();
                }
                self.hovered_player_id = Some(id);
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
            }
            Message::PlayerUnhovered(id) => {
                // If audio menu is open for this player, do not fade out
                if self.open_audio_menu_player_id == Some(id) {
                    return Task::none();
                }
                // When moving mouse outside of player / hover ends, immediately trigger fade out
                if self.hovered_player_id == Some(id) {
                    self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                }
            }
            Message::SeekRelativeFocused(secs) => {
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
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
                    if vol > 0.0 {
                        if p.state.is_muted {
                            p.set_muted(false);
                        }
                        self.settings.is_global_muted = false;
                        self.scroll_engine.is_global_muted = false;
                        let _ = self.config_mgr.save_settings(&self.settings);
                    } else {
                        p.set_muted(true);
                    }
                }
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
            }
            Message::AdjustVolumeFocused(delta) => {
                if self.settings.playback_mode == PlaybackMode::Scroll && delta > 0.0 {
                    return self.update(Message::GlobalUnmute);
                }
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                        p.adjust_volume(delta);
                        if p.state.volume > 0.0 {
                            if p.state.is_muted {
                                p.set_muted(false);
                            }
                            self.settings.is_global_muted = false;
                            self.scroll_engine.is_global_muted = false;
                            let _ = self.config_mgr.save_settings(&self.settings);
                        } else {
                            p.set_muted(true);
                        }
                        self.toast_message = Some(format!("Volume: {:.0}%", p.state.volume * 100.0));
                        self.toast_time_remaining = 1;
                    }
                }
            }
            Message::SelectAudioTrack(id, track_id) => {
                self.open_audio_menu_player_id = None;
                if let Some(pos) = self.players.iter().position(|p| p.id == id) {
                    let was_already_active = self.focused_player_idx == pos;
                    self.focused_player_idx = pos;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                    }
                }
                let mut selected_pref = None;
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    p.set_audio_track(track_id);
                    if let Some(track) = p.state.audio_tracks.iter().find(|t| t.id == track_id) {
                        let label = wazoo_media::format_audio_track_label(track, 0);
                        self.toast_message = Some(format!("Audio: {label}"));
                        self.toast_time_remaining = 2;
                        selected_pref = Some(wazoo_media::get_track_preference_string(track));
                    }
                }
                if let Some(pref) = selected_pref {
                    self.settings.preferred_audio_language = Some(pref.clone());
                    let _ = self.config_mgr.save_settings(&self.settings);
                    for other in &mut self.players {
                        other.set_preferred_audio_language(Some(pref.clone()));
                    }
                }
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
            }
            Message::ToggleAudioMenu(id) => {
                if self.open_audio_menu_player_id == Some(id) {
                    self.open_audio_menu_player_id = None;
                } else {
                    self.open_audio_menu_player_id = Some(id);
                    self.hovered_player_id = Some(id);
                    self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
                }
            }
            Message::CloseAudioMenu => {
                self.open_audio_menu_player_id = None;
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
                    if !muted {
                        self.settings.is_global_muted = false;
                        let _ = self.config_mgr.save_settings(&self.settings);
                    }
                    self.toast_message = Some(if muted { self.t("player.muted") } else { self.t("player.unmuted") });
                    self.toast_time_remaining = 2;
                }
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
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
                self.scroll_engine.is_global_muted = self.settings.is_global_muted;
                for p in &mut self.players {
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    } else {
                        p.set_muted(self.settings.is_global_muted);
                    }
                }
                self.toast_message = Some(if self.settings.is_global_muted {
                    self.t("wazoo.global_mode_muted")
                } else {
                    self.t("wazoo.global_mode_unmuted")
                });
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::GlobalUnmute => {
                self.settings.is_global_muted = false;
                self.scroll_engine.is_global_muted = false;
                for p in &mut self.players {
                    if self.settings.playback_mode == PlaybackMode::Scroll {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    } else {
                        p.set_muted(false);
                    }
                }
                self.toast_message = Some("Global Mute: OFF".to_string());
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
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
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    let target_count = self.settings.player_count.clamp(1, 12);
                    while self.players.len() > target_count {
                        self.players.pop();
                    }
                    while self.players.len() < target_count {
                        self.add_player_internal();
                    }
                }
                self.settings.layout = match self.settings.layout {
                    LayoutMode::Grid => LayoutMode::Row,
                    LayoutMode::Row => LayoutMode::Column,
                    LayoutMode::Column => LayoutMode::Grid,
                };
                self.toast_message = Some(match self.settings.layout {
                    LayoutMode::Grid => self.t("wazoo.layout_grid"),
                    LayoutMode::Row => self.t("wazoo.layout_row"),
                    LayoutMode::Column => self.t("wazoo.layout_column"),
                });
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::SetPlayerCount(count) => {
                let target = count.clamp(1, 12);
                let was_scroll = self.settings.playback_mode == PlaybackMode::Scroll;
                if was_scroll {
                    self.cleanup_scroll_mode();
                }
                self.settings.player_count = target;
                while self.players.len() > target {
                    self.players.pop();
                }
                while self.players.len() < target {
                    self.add_player_internal();
                }
                for p in &mut self.players {
                    p.set_muted(self.settings.is_global_muted);
                    p.set_volume(1.0);
                }
                self.toast_message = Some(self.t_with("wazoo.set_players_count", &[
                    ("count", &target.to_string()),
                    ("suffix", if target == 1 { "" } else { "s" }),
                ]));
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::ToggleScrollMode => {
                self.show_menu_modal = false;
                self.show_dropdown_menu = false;
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Scroll => PlaybackMode::Normal,
                    _ => PlaybackMode::Scroll,
                };
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.settings.is_global_muted = true;
                    self.scroll_engine.is_global_muted = true;
                    let window_h = self.settings.window_bounds.height as f32;
                    self.scroll_engine.set_window_height(window_h);
                    let item_h = self.scroll_engine.default_item_height();

                    // Scroll mode requires starting from a clean single-player state.
                    // If we were in multi-player mode (e.g. 2-4 player grid), retain only
                    // the focused player and discard extra players before building the scroll stack.
                    if self.players.len() > 1 {
                        let keep_idx = if self.focused_player_idx < self.players.len() {
                            self.focused_player_idx
                        } else {
                            0
                        };
                        let focused_player = self.players.remove(keep_idx);
                        self.players.clear();
                        self.players.push(focused_player);
                        self.focused_player_idx = 0;
                    } else if self.players.is_empty() {
                        self.add_player_internal();
                    }

                    let ids: Vec<PlayerId> = self.players.iter().map(|p| p.id).collect();
                    self.scroll_engine.init_stack(&ids);

                    while let Some(spawn_y) = self.scroll_engine.needs_new_player() {
                        if let Some(id) = self.add_player_internal() {
                            self.scroll_engine.add_item(id, spawn_y, item_h);
                        } else {
                            break;
                        }
                    }

                    for p in &mut self.players {
                        p.seek_random();
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }

                    self.toast_message = Some(self.t("wazoo.scroll_mode_enabled"));
                    self.toast_time_remaining = 2;
                    let _ = self.config_mgr.save_settings(&self.settings);

                    return self.trigger_preload_task();
                } else {
                    self.cleanup_scroll_mode();
                    let target_count = self.settings.player_count.clamp(1, 12);
                    while self.players.len() > target_count {
                        self.players.pop();
                    }
                    while self.players.len() < target_count {
                        self.add_player_internal();
                    }
                    for p in &mut self.players {
                        p.set_muted(self.settings.is_global_muted);
                        p.set_volume(1.0);
                    }
                    self.toast_message = Some(self.t("wazoo.scroll_mode_disabled"));
                }
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
            }
            Message::ToggleFlipMode => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    let target_count = self.settings.player_count.clamp(1, 12);
                    while self.players.len() > target_count {
                        self.players.pop();
                    }
                    while self.players.len() < target_count {
                        self.add_player_internal();
                    }
                }
                self.settings.playback_mode = match self.settings.playback_mode {
                    PlaybackMode::Flip => PlaybackMode::Normal,
                    _ => PlaybackMode::Flip,
                };
                self.toast_message = Some(match self.settings.playback_mode {
                    PlaybackMode::Flip => self.t("wazoo.flip_mode_enabled"),
                    _ => self.t("wazoo.flip_mode_disabled"),
                });
                self.toast_time_remaining = 2;
                let _ = self.config_mgr.save_settings(&self.settings);
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
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    self.cleanup_scroll_mode();
                    let target_count = self.settings.player_count.clamp(1, 12);
                    while self.players.len() > target_count {
                        self.players.pop();
                    }
                    while self.players.len() < target_count {
                        self.add_player_internal();
                    }
                }
                if self.players.len() <= 1 {
                    return Task::none();
                }
                if let Some(id) = self.focused_player_id() {
                    self.players.retain(|p| p.id != id);
                    self.settings.player_count = self.players.len();
                    if self.focused_player_idx >= self.players.len() && !self.players.is_empty() {
                        self.focused_player_idx = self.players.len() - 1;
                    }
                    self.toast_message = Some(format!("Players: {}", self.players.len()));
                    self.toast_time_remaining = 2;
                    let _ = self.config_mgr.save_settings(&self.settings);
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
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
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
            }
            Message::SetFocusedPlayer(idx) => {
                if idx < self.players.len() {
                    let was_already_active = self.focused_player_idx == idx;
                    self.focused_player_idx = idx;
                    if !was_already_active {
                        self.focus_border_ticks = 20;
                        if self.show_transcript {
                            return self.load_transcript_for_focused_player();
                        }
                    }
                }
            }
            Message::OpenSearchModal => {
                self.show_search_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.show_settings_modal = false;
                self.show_help_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
                self.search_tags = self
                    .active_search_query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.search_input.clear();
                return Task::batch([
                    iced::widget::operation::focus("search_input"),
                ]);
            }
            Message::CloseSearchModal => {
                self.show_search_modal = false;
            }
            Message::SearchInputChanged(val) => {
                if val.contains(',') {
                    let mut parts: Vec<&str> = val.split(',').collect();
                    let remainder = parts.pop().unwrap_or("").to_string();
                    for part in parts {
                        let tag = part.trim();
                        if !tag.is_empty() && !self.search_tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                            self.search_tags.push(tag.to_string());
                        }
                    }
                    self.search_input = remainder.trim_start().to_string();
                    return iced::widget::operation::focus("search_input");
                } else {
                    self.search_input = val;
                }
            }
            Message::RemoveSearchTag(idx) => {
                if idx < self.search_tags.len() {
                    self.search_tags.remove(idx);
                }
                return iced::widget::operation::focus("search_input");
            }
            Message::PerformSearch => {
                let pending = self.search_input.trim();
                if !pending.is_empty() && !self.search_tags.iter().any(|t| t.eq_ignore_ascii_case(pending)) {
                    self.search_tags.push(pending.to_string());
                }
                self.search_input.clear();
                self.active_search_query = self.search_tags.join(", ");
                self.settings.last_query = self.active_search_query.clone();
                self.settings.last_folder = self.selected_search_folder.clone();
                let _ = self.config_mgr.save_settings(&self.settings);

                let folder = self.selected_search_folder.clone();
                let folders = if folder.is_empty() || folder == "All" {
                    Vec::new()
                } else {
                    vec![folder.clone()]
                };

                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    let total = results.len();
                    self.available_videos = results;
                    self.show_video_totals_notice(total, &folder);

                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                    if self.show_transcript {
                        return self.load_transcript_for_focused_player();
                    }
                }
                self.show_search_modal = false;
            }
            Message::OpenSettingsModal => {
                self.show_settings_modal = true;
                self.show_dropdown_menu = false;
                self.show_menu_modal = false;
                self.show_search_modal = false;
                self.show_help_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
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
                self.show_search_modal = false;
                self.show_settings_modal = false;
                self.show_bookmarks_modal = false;
                self.show_titlebar = false;
                self.toast_message = None;
                self.hovered_player_id = None;
                self.player_overlay_ticks = 0;
            }
            Message::CloseHelpModal => {
                self.show_help_modal = false;
            }
            Message::ToggleBookmarksModal => {
                self.show_bookmarks_modal = !self.show_bookmarks_modal;
                if self.show_bookmarks_modal {
                    self.show_dropdown_menu = false;
                    self.show_menu_modal = false;
                    self.show_search_modal = false;
                    self.show_settings_modal = false;
                    self.show_help_modal = false;
                    self.show_titlebar = false;
                    self.toast_message = None;
                    self.hovered_player_id = None;
                    self.player_overlay_ticks = 0;
                }
            }
            Message::CloseBookmarksModal => {
                self.show_bookmarks_modal = false;
            }
            Message::AddBookmarkFocused => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter().find(|p| p.id == id) {
                        let path = p.path().to_string();
                        if !path.is_empty() {
                            let name = format::format_descriptive_title(&path);
                            let query = self.active_search_query.clone();
                            let position_secs = p.position().as_secs_f64();
                            let is_shuffle = self.is_shuffle_mode;

                            if let Some(existing) = self.settings.bookmarks.iter_mut().find(|b| b.path == path) {
                                existing.name = name.clone();
                                existing.query = query;
                                existing.position_secs = position_secs;
                                existing.is_shuffle = is_shuffle;
                                self.toast_message = Some(self.t_with("bookmarks.updated", &[("name", &name)]));
                            } else {
                                self.settings.bookmarks.push(Bookmark {
                                    name: name.clone(),
                                    query,
                                    path,
                                    position_secs,
                                    is_shuffle,
                                });
                                self.toast_message = Some(self.t_with("bookmarks.added", &[("name", &name)]));
                            }
                            let _ = self.config_mgr.save_settings(&self.settings);
                            self.toast_time_remaining = 3;
                        }
                    }
                }
            }
            Message::RemoveBookmarkFocused => {
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter().find(|p| p.id == id) {
                        let path = p.path();
                        if let Some(pos) = self.settings.bookmarks.iter().position(|b| b.path == path) {
                            let removed = self.settings.bookmarks.remove(pos);
                            let _ = self.config_mgr.save_settings(&self.settings);
                            self.toast_message = Some(self.t_with("bookmarks.removed", &[("name", &removed.name)]));
                            self.toast_time_remaining = 3;
                        } else {
                            self.toast_message = Some(self.t("bookmarks.not_found"));
                            self.toast_time_remaining = 2;
                        }
                    }
                }
            }
            Message::RemoveBookmark(idx) => {
                if idx < self.settings.bookmarks.len() {
                    let removed = self.settings.bookmarks.remove(idx);
                    let _ = self.config_mgr.save_settings(&self.settings);
                    self.toast_message = Some(self.t_with("bookmarks.removed", &[("name", &removed.name)]));
                    self.toast_time_remaining = 2;
                }
            }
            Message::JumpToBookmark(b) => {
                // 1. Restore shuffle vs linear mode
                self.is_shuffle_mode = b.is_shuffle;

                // 2. Update the global search query to match the bookmark's query
                self.active_search_query = b.query.clone();
                self.search_tags = b
                    .query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.search_input.clear();
                self.settings.last_query = b.query.clone();
                self.settings.last_folder = self.selected_search_folder.clone();
                let _ = self.config_mgr.save_settings(&self.settings);

                // 3. Query the database using the updated search query
                let folders = if self.selected_search_folder.is_empty() || self.selected_search_folder == "All" {
                    Vec::new()
                } else {
                    vec![self.selected_search_folder.clone()]
                };

                if let Ok(results) = self.db.search_videos(&self.active_search_query, &folders) {
                    self.last_total_videos = results.len();
                    self.available_videos = results;
                }

                // 4. Load the bookmarked video and position in the focused player
                let focused_id = if let Some(id) = self.focused_player_id() {
                    self.loading_player_ids.insert(id);
                    self.loading_player_ticks.insert(id, 0);
                    let title = format::format_video_title(&b.path);
                    let curr_player = self.players.iter().find(|p| p.id == id);
                    let prev_muted = curr_player.map(|p| p.state.is_muted);
                    let prev_volume = curr_player.map(|p| p.state.volume);

                    if let Ok(mut handle) = self.create_video_handle_with_start(id, &b.path, &title, Some(b.position_secs)) {
                        handle.set_muted(prev_muted.unwrap_or(self.settings.is_global_muted));
                        if let Some(vol) = prev_volume {
                            handle.set_volume(vol);
                        }
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                            *p = handle;
                        }
                    }
                    Some(id)
                } else {
                    let id = self.next_player_id;
                    self.next_player_id += 1;
                    let title = format::format_video_title(&b.path);
                    if let Ok(mut handle) = self.create_video_handle_with_start(id, &b.path, &title, Some(b.position_secs)) {
                        handle.set_muted(self.settings.is_global_muted);
                        handle.set_subtitles_visible(self.subtitles_enabled);
                        self.players.push(handle);
                        self.focused_player_idx = 0;
                        Some(id)
                    } else {
                        None
                    }
                };

                // 5. Reconcile other players if what they are playing does not exist in the new queried list of files
                self.reconcile_players_with_available_videos(focused_id);
                self.save_session_state();

                let mode_str = if b.is_shuffle { self.t("bookmarks.shuffle") } else { self.t("bookmarks.linear") };
                let time_str = format::format_time_str(b.position_secs);
                self.toast_message = Some(self.t_with(
                    "bookmarks.jumped",
                    &[
                        ("name", &b.name),
                        ("time", &time_str),
                        ("mode", &mode_str),
                    ],
                ));
                self.toast_time_remaining = 3;
                self.show_bookmarks_modal = false;
                if self.show_transcript {
                    return self.load_transcript_for_focused_player();
                }
            }
            Message::RandomSeekFocused => {
                self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
                if let Some(id) = self.focused_player_id() {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                        p.seek_random();
                        let pos = p.position();
                        let dur = p.duration();
                        self.toast_message = Some(format!(
                            "Random Seek  [{} / {}]",
                            format::format_time_str(pos.as_secs_f64()),
                            format::format_time_str(dur.as_secs_f64())
                        ));
                        self.toast_time_remaining = 2;
                    }
                }
            }
            Message::ShowTitleOverlay => {
                if self.title_pill_ticks > 0 {
                    self.title_pill_ticks = 0;
                } else {
                    self.title_pill_ticks = 240; // ~4 seconds
                }
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
                    return self.update(Message::StartScan);
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
                        return self.update(Message::StartScan);
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

                let was_scanning = self.is_scanning;

                // 1. Immediately delete all files belonging to this folder from the SQLite database
                let removed_count = self.db.remove_videos_in_folder(&folder).unwrap_or(0);
                log::info!("Removed media folder '{}' ({} videos deleted from database)", folder, removed_count);

                // 2. Immediately refresh in-memory available_videos from DB
                if !self.active_search_query.is_empty() {
                    let folders = if self.selected_search_folder.is_empty() || self.selected_search_folder == "All" {
                        self.settings.media_folders.clone()
                    } else {
                        vec![self.selected_search_folder.clone()]
                    };
                    self.available_videos = self.db.search_videos(&self.active_search_query, &folders).unwrap_or_default();
                } else {
                    self.available_videos = self.db.get_all_videos().unwrap_or_default();
                }

                // 3. Immediately update active players
                if self.available_videos.is_empty() {
                    self.players.clear();
                    self.loading_player_ids.clear();
                    self.loading_player_ticks.clear();
                    self.focused_player_idx = 0;
                } else {
                    self.reconcile_players_with_available_videos(None);
                    self.save_session_state();
                }

                if self.selected_search_folder == folder {
                    self.selected_search_folder.clear();
                }

                let folder_name = format::format_video_folder(&folder);
                let display_name = if folder_name.is_empty() { folder } else { folder_name };
                self.toast_message = Some(format!("Removed {} ({} files)", display_name, format::format_number(removed_count)));
                self.toast_time_remaining = 2;
                self.last_total_videos = self.available_videos.len();

                if was_scanning {
                    return self.update(Message::StartScan);
                }
            }
            Message::StartScan => {
                // Cancel any currently running scan task
                if let Some(cancel) = self.scan_cancel.take() {
                    cancel.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                self.current_scan_id += 1;

                if !self.settings.media_folders.is_empty() {
                    let scan_id = self.current_scan_id;
                    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    self.scan_cancel = Some(cancel.clone());

                    self.is_scanning = true;
                    self.scan_progress = None;
                    let folders = self.settings.media_folders.clone();
                    let db_path = self.config_mgr.database_path();

                    return Task::stream(iced::stream::channel(100, move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

                        let cancel_inner = cancel.clone();
                        let scan_handle = tokio::spawn(async move {
                            let scanner = Scanner::default();
                            scanner.scan_and_index_with_cancel(&folders, db_path, Some(tx), Some(cancel_inner)).await
                        });

                        while let Some(progress) = rx.recv().await {
                            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                                break;
                            }
                            let _ = output.send(Message::ScanProgressUpdate(scan_id, progress)).await;
                        }

                        let res = match scan_handle.await {
                            Ok(inner_res) => inner_res,
                            Err(join_err) => Err(join_err.to_string()),
                        };

                        if !cancel.load(std::sync::atomic::Ordering::Relaxed) {
                            let _ = output.send(Message::ScanFinished(scan_id, res)).await;
                        }
                    }));
                } else {
                    self.is_scanning = false;
                    self.scan_progress = None;
                }
            }
            Message::ScanProgressUpdate(scan_id, progress) => {
                if scan_id != self.current_scan_id {
                    return Task::none();
                }
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
                            format::format_number(progress.files_found)
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
            Message::ScanFinished(scan_id, res) => {
                if scan_id != self.current_scan_id {
                    return Task::none();
                }
                self.is_scanning = false;
                self.scan_progress = None;
                self.scan_cancel = None;
                match res {
                    Ok(_count) => {
                        let folder = self.selected_search_folder.clone();
                        let folders = if folder.is_empty() || folder == "All" {
                            Vec::new()
                        } else {
                            vec![folder.clone()]
                        };
                        if let Ok(videos) = self.db.search_videos(&self.active_search_query, &folders) {
                            let total = videos.len();
                            self.available_videos = videos;
                            self.show_video_totals_notice(total, &folder);
                            if self.players.is_empty() && !self.available_videos.is_empty() {
                                let count = self.settings.player_count.clamp(1, 12);
                                for _ in 0..count {
                                    self.add_player_internal();
                                }
                            }
                            self.reconcile_players_with_available_videos(None);
                            self.save_session_state();
                        }
                    }
                    Err(err) => {
                        if err != "Scan cancelled" {
                            self.toast_message = Some(format!("Scan error: {err}"));
                            self.toast_time_remaining = 3;
                        }
                    }
                }
            }
            Message::ToggleSubtitles => {
                self.subtitles_enabled = !self.subtitles_enabled;
                for p in &mut self.players {
                    p.set_subtitles_visible(self.subtitles_enabled);
                }
                self.toast_message = Some(if self.subtitles_enabled {
                    self.t("player.subtitles_enabled")
                } else {
                    self.t("player.subtitles_disabled")
                });
                self.toast_time_remaining = 2;
            }
            Message::EscapePressed => {
                if self.show_transcript_menu {
                    self.show_transcript_menu = false;
                    return Task::none();
                }
                if self.open_audio_menu_player_id.is_some() {
                    self.open_audio_menu_player_id = None;
                    return Task::none();
                }
                if self.is_any_modal_open()
                    || self.show_dropdown_menu
                    || self.show_file_picker
                    || self.show_transcript
                {
                    self.show_help_modal = false;
                    self.show_search_modal = false;
                    self.show_settings_modal = false;
                    self.show_menu_modal = false;
                    self.show_bookmarks_modal = false;
                    self.show_dropdown_menu = false;
                    self.show_file_picker = false;
                    self.show_transcript = false;
                    self.show_transcript_menu = false;
                } else {
                    self.show_menu_modal = true;
                    self.show_dropdown_menu = false;
                }
            }
            Message::AnimationTick => {
                if self.settings.playback_mode == PlaybackMode::Scroll {
                    let offscreen = self.scroll_engine.tick();
                    if !offscreen.is_empty() {
                        let (keep, despawned): (Vec<_>, Vec<_>) = self
                            .players
                            .drain(..)
                            .partition(|p| !offscreen.contains(&p.id));
                        self.players = keep;
                        for &id in &offscreen {
                            self.loading_player_ids.remove(&id);
                            self.loading_player_ticks.remove(&id);
                        }
                        if !despawned.is_empty() {
                            std::thread::spawn(move || drop(despawned));
                        }
                    }

                    let item_h = self.scroll_engine.default_item_height();
                    let mut needs_preload = false;

                    // Non-blocking spawn: attach preloaded player seamlessly if ready
                    while let Some(spawn_y) = self.scroll_engine.needs_new_player_with_margin(item_h * 0.5) {
                        if let Some(mut handle) = self.preloaded_player.take() {
                            self.scroll_engine.add_item(handle.id, spawn_y, item_h);
                            let vol = self.scroll_engine.calculate_player_volume(handle.id);
                            handle.set_volume(vol);
                            self.players.push(handle);
                            needs_preload = true;
                        } else {
                            // Preloaded player still preparing in background - do NOT block!
                            break;
                        }
                    }

                    for p in &mut self.players {
                        let vol = self.scroll_engine.calculate_player_volume(p.id);
                        p.set_volume(vol);
                    }

                    if needs_preload || (self.preloaded_player.is_none() && !self.is_preloading) {
                        return self.trigger_preload_task();
                    }
                }
            }
            Message::VideoFrameTick => {
                self.spinner_ticks = self.spinner_ticks.wrapping_add(1);
                if self.open_audio_menu_player_id.is_some() {
                    self.player_overlay_ticks = PLAYER_OVERLAY_HIDE_TICKS;
                } else if self.player_overlay_ticks > 0 {
                    self.player_overlay_ticks -= 1;
                    if self.player_overlay_ticks == 0 {
                        self.hovered_player_id = None;
                    }
                }
                if self.title_pill_ticks > 0 {
                    self.title_pill_ticks -= 1;
                }
                if self.focus_border_ticks > 0 {
                    self.focus_border_ticks -= 1;
                }
                if self.is_point_in_titlebar(self.cursor_position) || self.show_dropdown_menu {
                    if self.show_dropdown_menu {
                        self.show_titlebar = true;
                        self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                        self.titlebar_hover_ticks = 0;
                    } else if self.show_titlebar {
                        self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                    } else {
                        self.titlebar_hover_ticks += 1;
                        if self.titlebar_hover_ticks >= TITLEBAR_SHOW_DELAY_TICKS {
                            self.show_titlebar = true;
                            self.titlebar_hide_ticks = TITLEBAR_HIDE_TICKS;
                            self.titlebar_hover_ticks = 0;
                            if self.hovered_player_id.is_some() {
                                self.player_overlay_ticks = self.player_overlay_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                            }
                        }
                    }
                } else {
                    self.titlebar_hover_ticks = 0;
                    if self.titlebar_hide_ticks > 0 {
                        self.titlebar_hide_ticks -= 1;
                        if self.titlebar_hide_ticks == 0 {
                            self.show_titlebar = false;
                        }
                    }
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
                self.save_session_state();
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
            Message::ModalCardClicked => {}
        }
        Task::none()
    }
}
