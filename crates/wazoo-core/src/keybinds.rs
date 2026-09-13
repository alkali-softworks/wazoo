/**
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
            toggle_mute: "m".to_string(),
            play_pause: "Space".to_string(),
            toggle_play_mode: "s".to_string(),
            prev_video: "Up".to_string(),
            next_video: "Down".to_string(),
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
    pub fn help_shortcuts_with_descriptions(&self, t: impl Fn(&str) -> String) -> Vec<(String, String)> {
        let search_display = if self.search_videos == "j, /" || self.search_videos == "j, /, f" {
            "j OR /".to_string()
        } else {
            self.search_videos.replace(',', " OR")
        };
        let prev_next_display = if self.prev_video.eq_ignore_ascii_case("up") && self.next_video.eq_ignore_ascii_case("down") {
            "↓ ↑".to_string()
        } else {
            format!("{} {}", self.prev_video, self.next_video)
        };
        let seek_display = if self.seek_backward.eq_ignore_ascii_case("left") && self.seek_forward.eq_ignore_ascii_case("right") {
            "← →".to_string()
        } else {
            format!("{} {}", self.seek_backward, self.seek_forward)
        };
        let volume_display = if self.volume_down == "[" && self.volume_up == "]" {
            "[ OR ]".to_string()
        } else {
            format!("{} OR {}", self.volume_down, self.volume_up)
        };
        let frame_display = if (self.prev_frame == "," || self.prev_frame == "<") && (self.next_frame == "." || self.next_frame == ">") {
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
            (self.toggle_play_mode.clone(), t("help.shortcuts.toggle_play_mode")),
            (self.toggle_mute.clone(), t("help.shortcuts.toggle_mute")),
            (play_pause_display, t("help.shortcuts.play_pause")),
            (self.toggle_file_picker.clone(), t("help.shortcuts.toggle_file_picker")),
            (self.add_player.clone(), t("help.shortcuts.add_player")),
            (self.remove_player.clone(), t("help.shortcuts.remove_player")),
            (self.focus_next.clone(), t("help.shortcuts.focus_next")),
            (volume_display, t("help.shortcuts.adjust_volume")),
            (self.toggle_bookmarks.clone(), t("help.shortcuts.toggle_bookmarks")),
            (speed_or_bookmark_display, t("bookmarks.bookmark_current")),
            (self.toggle_scroll.clone(), t("help.shortcuts.toggle_scroll")),
            (self.toggle_flip.clone(), t("help.shortcuts.toggle_flip")),
            (self.toggle_layout.clone(), t("help.shortcuts.toggle_layout")),
            (self.toggle_subtitles.clone(), t("help.shortcuts.toggle_subtitles")),
            (self.toggle_transcript.clone(), t("help.shortcuts.toggle_transcript")),
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
        assert_eq!(kb.menu_hint(&kb.close_app), "Alt+X");
    }

    #[test]
    fn test_help_shortcuts_generation() {
        let kb = KeybindSettings::default();
        let shortcuts = kb.help_shortcuts_with_descriptions(|key| key.to_string());
        assert_eq!(shortcuts.len(), 21);
        assert_eq!(shortcuts[0], ("j OR /".to_string(), "help.shortcuts.search_videos".to_string()));
        assert_eq!(shortcuts[7], ("n".to_string(), "help.shortcuts.add_player".to_string()));
        assert_eq!(shortcuts[12], ("+ / =".to_string(), "bookmarks.bookmark_current".to_string()));
        assert_eq!(shortcuts[19], ("Alt + X".to_string(), "help.shortcuts.close_app".to_string()));
    }
}
