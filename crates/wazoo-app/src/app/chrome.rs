/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Window Chrome, Overlays, Cursor & Transcript Helpers
 *
 * Manages titlebar/menu slide & fade progress, floating HUD overlay opacity,
 * cursor auto-hide state, window opacity, file picker cleanup, and transcript loading.
 */

use super::{DROPDOWN_MENU_SLIDE_TICKS, TITLEBAR_FADE_TICKS, TITLEBAR_SLIDE_TICKS, WazooApp};
use crate::message::Message;
use iced::{Point, Task};

impl WazooApp {
    pub(crate) fn is_any_modal_open(&self) -> bool {
        self.modals.is_any_open()
    }

    pub(crate) fn is_modal_or_menu_open(&self) -> bool {
        self.is_any_modal_open() || self.titlebar.show_dropdown_menu
    }

    /// Tests whether the given point falls within the draggable top titlebar zone (height = 35px).
    pub fn is_point_in_titlebar(&self, pos: Point) -> bool {
        let width = if self.settings.window_bounds.width > 0 {
            self.settings.window_bounds.width as f32
        } else {
            f32::MAX
        };
        pos.x >= 0.0 && pos.x <= width && pos.y >= 0.0 && pos.y < 35.0
    }

    /// Computes the rendered opacity [0.0..1.0] of the auto-hiding top titlebar.
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

    /// Computes the slide-down progress [0.0..1.0] of the top titlebar.
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

    /// Computes the slide-down progress [0.0..1.0] of the top dropdown navigation menu.
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

    /// Shows or resets the auto-hide countdown for the floating HUD controls overlay.
    pub fn trigger_player_overlay(&mut self) {
        self.overlay.trigger();
    }

    /// Computes the rendered opacity [0.0..1.0] of the floating HUD controls overlay.
    pub fn player_overlay_alpha(&self) -> f32 {
        self.overlay.alpha()
    }

    /// Determines whether the mouse cursor should be auto-hidden during undisturbed video playback.
    pub fn should_hide_cursor(&self) -> bool {
        !self.players.is_empty()
            && self.overlay.ticks == 0
            && self.open_audio_menu_id.is_none()
            && self.players.iter().all(|p| !p.is_loading)
            && !self.titlebar.show
            && !self.is_modal_or_menu_open()
    }

    pub fn current_opacity(&self) -> f32 {
        if self.window.is_alt_pressed {
            0.85
        } else {
            self.settings.window_opacity.clamp(0.05, 1.0)
        }
    }

    pub(crate) fn close_file_picker(&mut self) {
        self.drawers.show_file_picker = false;
        self.drawers.file_picker_entries.clear();
        self.drawers.file_picker_entries.shrink_to_fit();
        self.drawers.file_picker_groups.clear();
        self.drawers.file_picker_groups.shrink_to_fit();
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
}
