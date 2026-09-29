/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Continuous Scroll Engine
 *
 * Calculates player positioning, seamless lookahead loading, despawning offscreen players,
 * and proximity-based audio volume transitions for the infinite stream mode.
 */

use crate::player::PlayerId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScrollItem {
    pub player_id: PlayerId,
    pub y_pos: f32,
    pub height: f32,
}

pub struct ScrollEngine {
    pub scroll_speed: f32,
    pub items: HashMap<PlayerId, ScrollItem>,
    pub window_width: f32,
    pub window_height: f32,
    pub is_global_muted: bool,
}

impl ScrollEngine {
    pub fn new(window_height: f32) -> Self {
        Self {
            scroll_speed: 1.0,
            items: HashMap::new(),
            window_width: 0.0,
            window_height,
            is_global_muted: true,
        }
    }

    pub fn with_window_size(window_width: f32, window_height: f32) -> Self {
        Self {
            scroll_speed: 1.0,
            items: HashMap::new(),
            window_width,
            window_height,
            is_global_muted: true,
        }
    }

    pub fn set_window_height(&mut self, height: f32) {
        self.window_height = height;
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
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

    pub fn update_height(&mut self, player_id: PlayerId, height: f32) -> bool {
        if height <= 0.0 {
            return false;
        }
        if let Some(item) = self.items.get_mut(&player_id) {
            if (item.height - height).abs() > 1.0 {
                item.height = height;
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn default_item_height(&self) -> f32 {
        if self.window_width > 0.0 {
            (self.window_width / (16.0 / 9.0)).max(180.0)
        } else {
            (self.window_height / 2.0).max(180.0)
        }
    }

    pub fn item_height_for_aspect_ratio(&self, aspect_ratio: f32) -> f32 {
        if aspect_ratio > 0.05 && self.window_width > 0.0 {
            (self.window_width / aspect_ratio).max(180.0)
        } else {
            self.default_item_height()
        }
    }

    /// Initialize a gapless vertical stack of players starting from y = 0.0 downwards.
    pub fn init_stack(&mut self, player_ids: &[PlayerId]) {
        self.items.clear();
        let item_h = self.default_item_height();
        let mut y = 0.0;
        for &id in player_ids {
            self.add_item(id, y, item_h);
            y += item_h;
        }
    }

    /// Initialize a gapless vertical stack of players with custom or real heights.
    pub fn init_stack_with_heights(&mut self, items: &[(PlayerId, f32)]) {
        self.items.clear();
        let mut y = 0.0;
        for &(id, h) in items {
            let item_h = if h > 0.0 {
                h
            } else {
                self.default_item_height()
            };
            self.add_item(id, y, item_h);
            y += item_h;
        }
    }

    /// Recalculates and stacks all active player positions gaplessly.
    /// Matches the wazoo-desktop recalculateScrollPositions logic.
    pub fn recalculate_positions(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let mut sorted_ids: Vec<PlayerId> = self.items.keys().copied().collect();
        sorted_ids.sort_by(|&a, &b| {
            let y_a = self.items.get(&a).map(|i| i.y_pos).unwrap_or(0.0);
            let y_b = self.items.get(&b).map(|i| i.y_pos).unwrap_or(0.0);
            y_a.partial_cmp(&y_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let first_id = sorted_ids[0];
        let mut current_top = self.items.get(&first_id).map(|i| i.y_pos).unwrap_or(0.0);
        if current_top > 0.0 {
            current_top = 0.0;
        }

        let total_height: f32 = sorted_ids
            .iter()
            .filter_map(|id| self.items.get(id))
            .map(|i| i.height)
            .sum();

        if total_height < self.window_height {
            current_top = self.window_height - total_height;
        }

        for id in sorted_ids {
            if let Some(item) = self.items.get_mut(&id) {
                item.y_pos = current_top;
                current_top += item.height;
            }
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

    /// Check if the bottom edge of the content has left empty space on the screen (with optional lookahead margin).
    /// Returns the target `y` coordinate where a new player should be spawned, or None if screen is full.
    pub fn needs_new_player_with_margin(&self, margin: f32) -> Option<f32> {
        if self.items.is_empty() {
            return Some(0.0);
        }

        let mut max_bottom = -f32::INFINITY;
        for item in self.items.values() {
            let bottom = item.y_pos + item.height;
            if bottom > max_bottom {
                max_bottom = bottom;
            }
        }

        if max_bottom < self.window_height + margin {
            Some(max_bottom)
        } else {
            None
        }
    }

    /// Check if the bottom edge of the content has left empty space on the screen.
    /// Returns the target `y` coordinate where a new player should be spawned, or None if screen is full.
    pub fn needs_new_player(&self) -> Option<f32> {
        self.needs_new_player_with_margin(0.0)
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
