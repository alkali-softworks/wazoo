/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Hardware Input Handlers
 *
 * Implements keyboard shortcut routing, modal key management, and mouse/drag tracking.
 */

use crate::app::{PLAYER_OVERLAY_FADE_TICKS, TITLEBAR_FADE_TICKS, TITLEBAR_HIDE_TICKS, WazooApp};
use crate::keybinds::find_key_action;
use crate::message::Message;
use iced::{
    Point, Task,
    keyboard::{Key, key::Named},
};
use std::time::{Duration, Instant};
use wazoo_core::{KeyAction, PlaybackMode};

impl WazooApp {
    /// Handles keyboard shortcut routing and modal escape handling.
    pub(crate) fn handle_key_pressed(
        &mut self,
        key: Key,
        status: iced::event::Status,
    ) -> Task<Message> {
        // If Alt key pressed
        if key == Key::Named(Named::Alt) || key == Key::Named(Named::AltGraph) {
            self.window.is_alt_pressed = true;
            return Task::none();
        }

        // If Alt is held and key matches close_app: quit app
        if self.window.is_alt_pressed {
            if let Some(KeyAction::CloseApp) = find_key_action(&self.settings.keybinds, &key, true)
            {
                return self.update(Message::CloseApp);
            }
        }

        if key == Key::Named(Named::Escape) {
            return self.update(Message::EscapePressed);
        }

        // If typing in either search input (captured by widget) or if search modal or any modal is open,
        // do NOT allow keystrokes to trigger global shortcuts (e.g. 'm' for mute, 's' for shuffle, etc.)
        if status == iced::event::Status::Captured || self.is_any_modal_open() {
            if self.modals.search {
                if key == Key::Named(Named::Enter) {
                    return self.update(Message::PerformSearch);
                }
                if key == Key::Named(Named::Backspace) && self.search.input.is_empty() {
                    self.search.tags.pop();
                    return iced::widget::operation::focus("search_input");
                }
            }
            return Task::none();
        }

        if let Some(action) =
            find_key_action(&self.settings.keybinds, &key, self.window.is_alt_pressed)
        {
            match action {
                KeyAction::CloseApp => return self.update(Message::CloseApp),
                KeyAction::TogglePlayPause => return self.update(Message::TogglePlayFocused),
                KeyAction::PrevVideo => return self.update(Message::PrevVideoFocused),
                KeyAction::NextVideo => return self.update(Message::NextVideoFocused),
                KeyAction::SeekBackward => return self.update(Message::SeekRelativeFocused(-5.0)),
                KeyAction::SeekForward => return self.update(Message::SeekRelativeFocused(5.0)),
                KeyAction::FocusNext => return self.update(Message::CycleFocusedPlayer),
                KeyAction::SearchVideos => return self.update(Message::OpenSearchModal),
                KeyAction::AddNewPlayer => return self.update(Message::AddNewPlayer),
                KeyAction::RemovePlayer => return self.update(Message::RemoveFocusedPlayer),
                KeyAction::CycleLayout => return self.update(Message::CycleLayout),
                KeyAction::ToggleFilePicker => return self.update(Message::ToggleFilePicker),
                KeyAction::ToggleTranscript => return self.update(Message::ToggleTranscript),
                KeyAction::ToggleBookmarks => return self.update(Message::ToggleBookmarksModal),
                KeyAction::ToggleHistory => return self.update(Message::ToggleHistoryDrawer),
                KeyAction::ToggleMute => return self.update(Message::ToggleMuteFocused),
                KeyAction::TogglePlayMode => return self.update(Message::ToggleShuffleMode),
                KeyAction::VolumeDown => return self.update(Message::AdjustVolumeFocused(-0.1)),
                KeyAction::VolumeUp => return self.update(Message::AdjustVolumeFocused(0.1)),
                KeyAction::ToggleScroll => return self.update(Message::ToggleScrollMode),
                KeyAction::ToggleFlip => return self.update(Message::ToggleFlipMode),
                KeyAction::ToggleSubtitles => return self.update(Message::ToggleSubtitles),
                KeyAction::PrevFrame => return self.update(Message::StepFrameBackwardFocused),
                KeyAction::NextFrame => return self.update(Message::StepFrameForwardFocused),
                KeyAction::RandomSeek => return self.update(Message::RandomSeekFocused),
                KeyAction::ShowTitleOverlay => return self.update(Message::ShowTitleOverlay),
                KeyAction::Player1 => return self.update(Message::SetPlayerCount(1)),
                KeyAction::Player2 => return self.update(Message::SetPlayerCount(2)),
                KeyAction::Player3 => return self.update(Message::SetPlayerCount(3)),
                KeyAction::Player4 => return self.update(Message::SetPlayerCount(4)),
                KeyAction::SpeedOrBookmarkDown => {
                    if self.cube.is_present() {
                        return self.update(Message::RemoveCube);
                    } else if self.settings.playback_mode == PlaybackMode::Scroll {
                        return self.update(Message::AdjustScrollSpeed(-0.1));
                    } else {
                        return self.update(Message::RemoveBookmarkFocused);
                    }
                }
                KeyAction::SpeedOrBookmarkUp => {
                    if self.cube.is_present() {
                        return self.update(Message::SpawnCube);
                    } else if self.settings.playback_mode == PlaybackMode::Scroll {
                        return self.update(Message::AdjustScrollSpeed(0.1));
                    } else {
                        return self.update(Message::AddBookmarkFocused);
                    }
                }
                KeyAction::ToggleAlwaysOnTop => return self.update(Message::ToggleAlwaysOnTop),
                KeyAction::MarkIn => return self.update(Message::MarkInFocused),
                KeyAction::MarkOut => return self.update(Message::MarkOutFocused),
                KeyAction::ClearMarkIn => return self.update(Message::ClearMarkInFocused),
                KeyAction::ClearMarkOut => return self.update(Message::ClearMarkOutFocused),
                KeyAction::OpenHelp => return self.update(Message::OpenHelpModal),
                KeyAction::OpenSettings => return self.update(Message::OpenSettingsModal),
                KeyAction::ToggleFilters => return self.update(Message::ToggleFilters),
                KeyAction::ToggleCube => return self.update(Message::ToggleCubeScreensaver),
                KeyAction::ToggleCubeOverlay => return self.update(Message::ToggleDesktopCubeScreensaver),
                KeyAction::ToggleFullscreen => return self.update(Message::ToggleFullscreen),
                KeyAction::FitWindow => return self.update(Message::FitWindow),
            }
        }
        Task::none()
    }

    /// Handles cursor movement, tracking window focus, titlebar auto-hide/fade ticks,
    /// and initiating window drag operations if the titlebar was pressed.
    pub(crate) fn handle_cursor_moved(
        &mut self,
        win_id: iced::window::Id,
        pos: Point,
    ) -> Task<Message> {
        self.window.id = Some(win_id);
        self.window.cursor_position = pos;
        self.window.is_focused = true;
        self.window.unfocused_frame_ticks = 0;

        if self.window.is_resizing() {
            return Task::none();
        }

        if self.window.is_dragging {
            let is_still_moving = self
                .window
                .last_drag_move
                .map(|t| t.elapsed() < Duration::from_millis(250))
                .unwrap_or(false);
            if is_still_moving {
                // While window is actively moving during drag, keep titlebar visible
                // even if cursor wobbles or flies around across the window
                self.titlebar.show = true;
                self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                self.titlebar.hover_ticks = 0;
            } else {
                // Window stopped moving! The OS drag has concluded (handles WM eating mouseup)
                self.window.is_dragging = false;
                self.titlebar.drag_pending = false;
                self.titlebar.press_origin = None;
                self.window.last_drag_move = None;
                if !self.is_point_in_titlebar(pos) {
                    self.titlebar.hide_ticks = self.titlebar.hide_ticks.min(TITLEBAR_FADE_TICKS);
                }
            }
        } else if self.titlebar.show_dropdown_menu {
            self.titlebar.show = true;
            self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
            self.titlebar.hover_ticks = 0;
            self.titlebar.slide_ticks = crate::app::TITLEBAR_SLIDE_TICKS;
            if self.hovered_player_id.is_some() {
                let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                self.overlay.ticks = self.overlay.ticks.min(current_fade);
            }
        } else if self.is_point_in_titlebar(pos) {
            if self.titlebar.show {
                self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                if self.hovered_player_id.is_some() {
                    let current_fade = self.overlay.fade_in_ticks.min(PLAYER_OVERLAY_FADE_TICKS);
                    self.overlay.ticks = self.overlay.ticks.min(current_fade);
                }
            }
        } else {
            self.titlebar.hover_ticks = 0;
            // When cursor leaves the titlebar area, start fading out smoothly without sticky delay
            if self.titlebar.show
                && !self.titlebar.show_dropdown_menu
                && !self.titlebar.drag_pending
            {
                let target_hide = if self.titlebar.slide_ticks < crate::app::TITLEBAR_SLIDE_TICKS {
                    (self.titlebar.slide_ticks * TITLEBAR_FADE_TICKS)
                        / crate::app::TITLEBAR_SLIDE_TICKS
                } else {
                    TITLEBAR_FADE_TICKS
                };
                self.titlebar.hide_ticks = self.titlebar.hide_ticks.min(target_hide.max(1));
            }
        }

        if self.titlebar.drag_pending {
            if let Some(origin) = self.titlebar.press_origin {
                let dist = (pos.x - origin.x).hypot(pos.y - origin.y);
                if dist > 5.0 {
                    self.window.is_dragging = true;
                    self.window.last_drag_move = Some(Instant::now());
                    self.titlebar.drag_pending = false;
                    self.titlebar.press_origin = None;
                    self.titlebar.last_click = None;
                    self.titlebar.show = true;
                    self.titlebar.hide_ticks = TITLEBAR_HIDE_TICKS;
                    return iced::window::drag(win_id);
                }
            }
        }

        Task::none()
    }
}
