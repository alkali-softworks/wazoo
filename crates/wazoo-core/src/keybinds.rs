/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Keybind Settings & Actions
 *
 * Centralized declaration of keyboard shortcut bindings and actions with serialization support.
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    CloseApp,
    SearchVideos,
    AddNewPlayer,
    RemovePlayer,
    CycleLayout,
    ToggleFilePicker,
    ToggleTranscript,
    ToggleBookmarks,
    ToggleMute,
    TogglePlayPause,
    TogglePlayMode,
    PrevVideo,
    NextVideo,
    SeekBackward,
    SeekForward,
    FocusNext,
    VolumeDown,
    VolumeUp,
    ToggleScroll,
    ToggleFlip,
    ToggleSubtitles,
    PrevFrame,
    NextFrame,
    RandomSeek,
    ShowTitleOverlay,
    Player1,
    Player2,
    Player3,
    Player4,
    SpeedOrBookmarkDown,
    SpeedOrBookmarkUp,
    ToggleHistory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindSettings {
    pub search_videos: String,
    pub add_player: String,
    pub remove_player: String,
    pub toggle_layout: String,
    pub toggle_file_picker: String,
    pub toggle_transcript: String,
    pub toggle_bookmarks: String,
    pub toggle_history: String,
    pub toggle_mute: String,
    pub play_pause: String,
    pub toggle_play_mode: String,
    pub prev_video: String,
    pub next_video: String,
    pub seek_backward: String,
    pub seek_forward: String,
    pub focus_next: String,
    pub volume_down: String,
    pub volume_up: String,
    pub toggle_scroll: String,
    pub toggle_flip: String,
    pub toggle_subtitles: String,
    pub prev_frame: String,
    pub next_frame: String,
    pub close_app: String,
    pub random_seek: String,
    pub show_title_overlay: String,
    pub player_1: String,
    pub player_2: String,
    pub player_3: String,
    pub player_4: String,
    pub speed_or_bookmark_down: String,
    pub speed_or_bookmark_up: String,
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            search_videos: "j, /".to_string(),
            add_player: "n".to_string(),
            remove_player: "x".to_string(),
            toggle_layout: "l".to_string(),
            toggle_file_picker: "h".to_string(),
            toggle_transcript: "v".to_string(),
            toggle_bookmarks: "b".to_string(),
            toggle_history: "y".to_string(),
            toggle_mute: "m".to_string(),
            play_pause: "Space".to_string(),
            toggle_play_mode: "s".to_string(),
            prev_video: "Down".to_string(),
            next_video: "Up".to_string(),
            seek_backward: "Left".to_string(),
            seek_forward: "Right".to_string(),
            focus_next: "Tab".to_string(),
            volume_down: "[".to_string(),
            volume_up: "]".to_string(),
            toggle_scroll: "5".to_string(),
            toggle_flip: "6".to_string(),
            toggle_subtitles: "c".to_string(),
            prev_frame: ",".to_string(),
            next_frame: ".".to_string(),
            close_app: "Alt+X".to_string(),
            random_seek: "r".to_string(),
            show_title_overlay: "t".to_string(),
            player_1: "1".to_string(),
            player_2: "2".to_string(),
            player_3: "3".to_string(),
            player_4: "4".to_string(),
            speed_or_bookmark_down: "-".to_string(),
            speed_or_bookmark_up: "+, =".to_string(),
        }
    }
}

impl KeybindSettings {
    pub const ALL_KEYS: &'static [&'static str] = &[
        "search_videos",
        "add_player",
        "remove_player",
        "toggle_layout",
        "toggle_file_picker",
        "toggle_transcript",
        "toggle_bookmarks",
        "toggle_history",
        "toggle_mute",
        "play_pause",
        "toggle_play_mode",
        "prev_video",
        "next_video",
        "seek_backward",
        "seek_forward",
        "focus_next",
        "volume_down",
        "volume_up",
        "toggle_scroll",
        "toggle_flip",
        "toggle_subtitles",
        "prev_frame",
        "next_frame",
        "close_app",
        "random_seek",
        "show_title_overlay",
        "player_1",
        "player_2",
        "player_3",
        "player_4",
        "speed_or_bookmark_down",
        "speed_or_bookmark_up",
    ];

    /// Validates if a JSON value contains the complete dictionary of non-empty key bindings.
    pub fn is_complete_json(val: &serde_json::Value) -> bool {
        if let Some(obj) = val.get("keybinds").and_then(|k| k.as_object()) {
            Self::ALL_KEYS.iter().all(|&k| {
                obj.get(k)
                    .and_then(|v| v.as_str())
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    /// Reconciles any empty or blank keybindings with the application defaults.
    pub fn reconcile_with_defaults(&mut self) {
        let def = Self::default();
        if self.search_videos.trim().is_empty() {
            self.search_videos = def.search_videos;
        }
        if self.add_player.trim().is_empty() {
            self.add_player = def.add_player;
        }
        if self.remove_player.trim().is_empty() {
            self.remove_player = def.remove_player;
        }
        if self.toggle_layout.trim().is_empty() {
            self.toggle_layout = def.toggle_layout;
        }
        if self.toggle_file_picker.trim().is_empty() {
            self.toggle_file_picker = def.toggle_file_picker;
        }
        if self.toggle_transcript.trim().is_empty() {
            self.toggle_transcript = def.toggle_transcript;
        }
        if self.toggle_bookmarks.trim().is_empty() {
            self.toggle_bookmarks = def.toggle_bookmarks;
        }
        if self.toggle_history.trim().is_empty() {
            self.toggle_history = def.toggle_history;
        }
        if self.toggle_mute.trim().is_empty() {
            self.toggle_mute = def.toggle_mute;
        }
        if self.play_pause.trim().is_empty() {
            self.play_pause = def.play_pause;
        }
        if self.toggle_play_mode.trim().is_empty() {
            self.toggle_play_mode = def.toggle_play_mode;
        }
        if self.prev_video.trim().is_empty() {
            self.prev_video = def.prev_video;
        }
        if self.next_video.trim().is_empty() {
            self.next_video = def.next_video;
        }
        if self.seek_backward.trim().is_empty() {
            self.seek_backward = def.seek_backward;
        }
        if self.seek_forward.trim().is_empty() {
            self.seek_forward = def.seek_forward;
        }
        if self.focus_next.trim().is_empty() {
            self.focus_next = def.focus_next;
        }
        if self.volume_down.trim().is_empty() {
            self.volume_down = def.volume_down;
        }
        if self.volume_up.trim().is_empty() {
            self.volume_up = def.volume_up;
        }
        if self.toggle_scroll.trim().is_empty() {
            self.toggle_scroll = def.toggle_scroll;
        }
        if self.toggle_flip.trim().is_empty() {
            self.toggle_flip = def.toggle_flip;
        }
        if self.toggle_subtitles.trim().is_empty() {
            self.toggle_subtitles = def.toggle_subtitles;
        }
        if self.prev_frame.trim().is_empty() {
            self.prev_frame = def.prev_frame;
        }
        if self.next_frame.trim().is_empty() {
            self.next_frame = def.next_frame;
        }
        if self.close_app.trim().is_empty() {
            self.close_app = def.close_app;
        }
        if self.random_seek.trim().is_empty() {
            self.random_seek = def.random_seek;
        }
        if self.show_title_overlay.trim().is_empty() {
            self.show_title_overlay = def.show_title_overlay;
        }
        if self.player_1.trim().is_empty() {
            self.player_1 = def.player_1;
        }
        if self.player_2.trim().is_empty() {
            self.player_2 = def.player_2;
        }
        if self.player_3.trim().is_empty() {
            self.player_3 = def.player_3;
        }
        if self.player_4.trim().is_empty() {
            self.player_4 = def.player_4;
        }
        if self.speed_or_bookmark_down.trim().is_empty() {
            self.speed_or_bookmark_down = def.speed_or_bookmark_down;
        }
        if self.speed_or_bookmark_up.trim().is_empty() {
            self.speed_or_bookmark_up = def.speed_or_bookmark_up;
        }
    }

    /// Formats a keybinding into a clean uppercase hint suitable for menu labels.
    /// E.g. "n" -> "N", "j, /" -> "J", "Alt+X" -> "Alt+X"
    pub fn menu_hint(&self, binding: &str) -> String {
        let first = binding.split(',').next().unwrap_or(binding).trim();
        if first.to_lowercase().starts_with("alt+") {
            let rest = &first[4..];
            format!("Alt+{}", rest.to_uppercase())
        } else if first.to_lowercase().starts_with("ctrl+") {
            let rest = &first[5..];
            format!("Ctrl+{}", rest.to_uppercase())
        } else {
            first.to_uppercase()
        }
    }

    /// Formats the help shortcuts list with localized action descriptions.
    pub fn help_shortcuts_with_descriptions(
        &self,
        t: impl Fn(&str) -> String,
    ) -> Vec<(String, String)> {
        let search_display = if self.search_videos == "j, /" || self.search_videos == "j, /, f" {
            "j OR /".to_string()
        } else {
            self.search_videos.replace(',', " OR")
        };
        let prev_next_display = if self.prev_video.eq_ignore_ascii_case("up")
            && self.next_video.eq_ignore_ascii_case("down")
        {
            "↓ ↑".to_string()
        } else {
            format!("{} {}", self.prev_video, self.next_video)
        };
        let seek_display = if self.seek_backward.eq_ignore_ascii_case("left")
            && self.seek_forward.eq_ignore_ascii_case("right")
        {
            "← →".to_string()
        } else {
            format!("{} {}", self.seek_backward, self.seek_forward)
        };
        let volume_display = if self.volume_down == "[" && self.volume_up == "]" {
            "[ OR ]".to_string()
        } else {
            format!("{} OR {}", self.volume_down, self.volume_up)
        };
        let frame_display = if (self.prev_frame == "," || self.prev_frame == "<")
            && (self.next_frame == "." || self.next_frame == ">")
        {
            "< OR >".to_string()
        } else {
            format!("{} OR {}", self.prev_frame, self.next_frame)
        };
        let close_display = if self.close_app.eq_ignore_ascii_case("alt+x") {
            "Alt + X".to_string()
        } else {
            self.close_app.replace('+', " + ")
        };
        let play_pause_display = if self.play_pause.eq_ignore_ascii_case("space") {
            "[space]".to_string()
        } else {
            self.play_pause.clone()
        };

        let speed_or_bookmark_display = if self.speed_or_bookmark_up == "+, =" {
            "+ / =".to_string()
        } else {
            self.speed_or_bookmark_up.replace(',', " /")
        };

        vec![
            (search_display, t("help.shortcuts.search_videos")),
            (prev_next_display, t("help.shortcuts.prev_next_video")),
            (seek_display, t("help.shortcuts.seek_back_forward")),
            (
                self.toggle_play_mode.clone(),
                t("help.shortcuts.toggle_play_mode"),
            ),
            (self.toggle_mute.clone(), t("help.shortcuts.toggle_mute")),
            (play_pause_display, t("help.shortcuts.play_pause")),
            (
                self.toggle_file_picker.clone(),
                t("help.shortcuts.toggle_file_picker"),
            ),
            (self.add_player.clone(), t("help.shortcuts.add_player")),
            (
                self.remove_player.clone(),
                t("help.shortcuts.remove_player"),
            ),
            (self.focus_next.clone(), t("help.shortcuts.focus_next")),
            (volume_display, t("help.shortcuts.adjust_volume")),
            (
                self.toggle_bookmarks.clone(),
                t("help.shortcuts.toggle_bookmarks"),
            ),
            (
                self.toggle_history.clone(),
                t("help.shortcuts.toggle_history"),
            ),
            (speed_or_bookmark_display, t("bookmarks.bookmark_current")),
            (
                self.toggle_scroll.clone(),
                t("help.shortcuts.toggle_scroll"),
            ),
            (self.toggle_flip.clone(), t("help.shortcuts.toggle_flip")),
            (
                self.toggle_layout.clone(),
                t("help.shortcuts.toggle_layout"),
            ),
            (
                self.toggle_subtitles.clone(),
                t("help.shortcuts.toggle_subtitles"),
            ),
            (
                self.toggle_transcript.clone(),
                t("help.shortcuts.toggle_transcript"),
            ),
            (frame_display, t("help.shortcuts.prev_next_frame")),
            (close_display, t("help.shortcuts.close_app")),
            ("Alt + Drag".to_string(), t("help.shortcuts.move_window")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_hint() {
        let kb = KeybindSettings::default();
        assert_eq!(kb.menu_hint(&kb.add_player), "N");
        assert_eq!(kb.menu_hint(&kb.search_videos), "J");
        assert_eq!(kb.menu_hint(&kb.toggle_layout), "L");
        assert_eq!(kb.menu_hint(&kb.toggle_file_picker), "H");
        assert_eq!(kb.menu_hint(&kb.toggle_transcript), "V");
        assert_eq!(kb.menu_hint(&kb.toggle_bookmarks), "B");
        assert_eq!(kb.menu_hint(&kb.toggle_history), "Y");
        assert_eq!(kb.menu_hint(&kb.close_app), "Alt+X");
    }

    #[test]
    fn test_help_shortcuts_generation() {
        let kb = KeybindSettings::default();
        let shortcuts = kb.help_shortcuts_with_descriptions(|key| key.to_string());
        assert_eq!(shortcuts.len(), 22);
        assert_eq!(
            shortcuts[0],
            (
                "j OR /".to_string(),
                "help.shortcuts.search_videos".to_string()
            )
        );
        assert_eq!(
            shortcuts[7],
            ("n".to_string(), "help.shortcuts.add_player".to_string())
        );
        assert_eq!(
            shortcuts[12],
            ("y".to_string(), "help.shortcuts.toggle_history".to_string())
        );
        assert_eq!(
            shortcuts[13],
            (
                "+ / =".to_string(),
                "bookmarks.bookmark_current".to_string()
            )
        );
        assert_eq!(
            shortcuts[20],
            (
                "Alt + X".to_string(),
                "help.shortcuts.close_app".to_string()
            )
        );
    }

    #[test]
    fn test_is_complete_json_and_reconcile_with_defaults() {
        // Incomplete json missing most keys
        let incomplete_json: serde_json::Value = serde_json::json!({
            "keybinds": {
                "add_player": "p"
            }
        });
        assert!(!KeybindSettings::is_complete_json(&incomplete_json));

        // Incomplete json with blank value
        let mut full_map = serde_json::Map::new();
        for &k in KeybindSettings::ALL_KEYS {
            full_map.insert(k.to_string(), serde_json::Value::String("x".to_string()));
        }
        full_map.insert(
            "add_player".to_string(),
            serde_json::Value::String("   ".to_string()),
        );
        let blank_json = serde_json::json!({ "keybinds": full_map });
        assert!(!KeybindSettings::is_complete_json(&blank_json));

        // Reconcile blank value
        let mut kb = KeybindSettings {
            add_player: "  ".to_string(),
            ..Default::default()
        };
        kb.reconcile_with_defaults();
        assert_eq!(kb.add_player, "n");

        // Complete json with non-empty values
        let complete_json = serde_json::to_value(crate::models::WazooSettings::default()).unwrap();
        assert!(KeybindSettings::is_complete_json(&complete_json));
    }
}
