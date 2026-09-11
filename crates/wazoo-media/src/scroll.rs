use std::collections::HashMap;
use crate::player::PlayerId;

#[derive(Debug, Clone)]
pub struct ScrollItem {
    pub player_id: PlayerId,
    pub y_pos: f32,
    pub height: f32,
}

pub struct ScrollEngine {
    pub scroll_speed: f32,
    pub items: HashMap<PlayerId, ScrollItem>,
    pub window_height: f32,
    pub is_global_muted: bool,
}

impl ScrollEngine {
    pub fn new(window_height: f32) -> Self {
        Self {
            scroll_speed: 1.0,
            items: HashMap::new(),
            window_height,
            is_global_muted: true,
        }
    }

    pub fn set_window_height(&mut self, height: f32) {
        self.window_height = height;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.scroll_speed = speed.clamp(0.1, 10.0);
    }

    pub fn add_item(&mut self, player_id: PlayerId, y_pos: f32, height: f32) {
        self.items.insert(
            player_id,
            ScrollItem {
                player_id,
                y_pos,
                height,
            },
        );
    }

    pub fn remove_item(&mut self, player_id: PlayerId) {
        self.items.remove(&player_id);
    }

    pub fn update_height(&mut self, player_id: PlayerId, height: f32) {
        if let Some(item) = self.items.get_mut(&player_id) {
            item.height = height;
        }
    }

    /// Advance the scroll positions by `scroll_speed`.
    /// Returns a list of player IDs that have scrolled completely off the top of the screen.
    pub fn tick(&mut self) -> Vec<PlayerId> {
        let mut offscreen_players = Vec::new();

        for (id, item) in self.items.iter_mut() {
            item.y_pos -= self.scroll_speed;

            if item.y_pos + item.height < 0.0 {
                offscreen_players.push(*id);
            }
        }

        for id in &offscreen_players {
            self.items.remove(id);
        }

        offscreen_players
    }

    /// Check if the bottom edge of the content has left empty space on the screen.
    /// Returns the target `y` coordinate where a new player should be spawned, or None if screen is full.
    pub fn needs_new_player(&self) -> Option<f32> {
        if self.items.is_empty() {
            return Some(self.window_height);
        }

        let mut max_bottom = -f32::INFINITY;
        for item in self.items.values() {
            let bottom = item.y_pos + item.height;
            if bottom > max_bottom {
                max_bottom = bottom;
            }
        }

        if max_bottom < self.window_height {
            Some(max_bottom)
        } else {
            None
        }
    }

    /// Calculate the volume (0.0 to 1.0) for a given player based on its visibility intersection.
    pub fn calculate_player_volume(&self, player_id: PlayerId) -> f64 {
        if self.is_global_muted {
            return 0.0;
        }

        let item = match self.items.get(&player_id) {
            Some(i) => i,
            None => return 0.0,
        };

        if item.height <= 0.0 {
            return 0.0;
        }

        let video_top = item.y_pos;
        let video_bottom = item.y_pos + item.height;

        let visible_top = video_top.max(0.0);
        let visible_bottom = video_bottom.min(self.window_height);
        let visible_height = (visible_bottom - visible_top).max(0.0);

        (visible_height / item.height).clamp(0.0, 1.0) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_engine() {
        let mut engine = ScrollEngine::new(1000.0);
        engine.is_global_muted = false;

        engine.add_item(1, 0.0, 500.0);
        assert_eq!(engine.calculate_player_volume(1), 1.0);

        // Player partially off top: y = -250, height = 500 -> 250 visible -> 50%
        engine.add_item(2, -250.0, 500.0);
        assert!((engine.calculate_player_volume(2) - 0.5).abs() < 0.01);

        // Needs new player because content ends at 500 while window is 1000
        assert_eq!(engine.needs_new_player(), Some(500.0));
    }
}
