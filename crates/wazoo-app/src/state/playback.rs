/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Playback Coordination, Track Settings & Player History
 */

use crate::app::PlayerNavHistory;
use std::collections::HashMap;
use wazoo_media::PlayerId;

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub focused_idx: usize,
    pub hovered_id: Option<PlayerId>,
    pub open_audio_menu_id: Option<PlayerId>,
    pub subtitles_enabled: bool,
    pub default_shuffle_mode: bool,
    pub shuffle_modes: HashMap<PlayerId, bool>,
    pub nav_history: HashMap<PlayerId, PlayerNavHistory>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            focused_idx: 0,
            hovered_id: None,
            open_audio_menu_id: None,
            subtitles_enabled: true,
            default_shuffle_mode: true,
            shuffle_modes: HashMap::new(),
            nav_history: HashMap::new(),
        }
    }
}

impl PlaybackState {
    pub fn is_player_shuffle(&self, id: PlayerId) -> bool {
        self.shuffle_modes
            .get(&id)
            .copied()
            .unwrap_or(self.default_shuffle_mode)
    }

    pub fn set_player_shuffle(&mut self, id: PlayerId, shuffle: bool) {
        self.shuffle_modes.insert(id, shuffle);
    }

    pub fn remove_player(&mut self, id: &PlayerId) {
        self.shuffle_modes.remove(id);
        self.nav_history.remove(id);
        if self.hovered_id == Some(*id) {
            self.hovered_id = None;
        }
        if self.open_audio_menu_id == Some(*id) {
            self.open_audio_menu_id = None;
        }
    }
}
