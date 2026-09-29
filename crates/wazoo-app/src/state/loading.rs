/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Buffering, Preload & Loading Spinner State
 */

use std::collections::{HashMap, HashSet};
use wazoo_media::{PlayerId, VideoHandle};

#[derive(Debug, Default)]
pub struct LoadingState {
    pub spinner_ticks: u32,
    pub player_ids: HashSet<PlayerId>,
    pub player_ticks: HashMap<PlayerId, usize>,
    pub preloaded_player: Option<VideoHandle>,
    pub is_preloading: bool,
}

impl LoadingState {
    pub fn is_player_loading(&self, id: &PlayerId) -> bool {
        self.player_ids.contains(id)
    }

    pub fn start(&mut self, id: PlayerId) {
        self.player_ids.insert(id);
        self.player_ticks.insert(id, 0);
    }

    pub fn stop(&mut self, id: &PlayerId) {
        self.player_ids.remove(id);
        self.player_ticks.remove(id);
    }

    pub fn clear(&mut self) {
        self.player_ids.clear();
        self.player_ticks.clear();
    }

    pub fn spinner_angle(&self) -> f32 {
        ((self.spinner_ticks * 12) % 360) as f32
    }
}
