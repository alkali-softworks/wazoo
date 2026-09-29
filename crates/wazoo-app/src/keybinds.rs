/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Keybind Matching & Resolution
 *
 * Maps incoming iced keyboard events to defined KeyActions using configured KeybindSettings.
 */

use iced::keyboard::{Key, key::Named};
use wazoo_core::{KeyAction, KeybindSettings};

/// Check whether an iced keyboard key matches a key binding string (e.g. "n", "j, /", ",", "Space", "Alt+X").
pub fn key_matches_binding(binding: &str, key: &Key, is_alt_pressed: bool) -> bool {
    let binding_trimmed = binding.trim();
    if binding_trimmed.is_empty() {
        return false;
    }

    // Special case: single comma as binding
    if binding_trimmed == "," {
        return !is_alt_pressed && match_single_key(",", key);
    }

    for token in parse_keybind_tokens(binding_trimmed) {
        let is_alt_binding = token.to_lowercase().starts_with("alt+");
        if is_alt_binding {
            if !is_alt_pressed {
                continue;
            }
            let key_part = token[4..].trim();
            if match_single_key(key_part, key) {
                return true;
            }
        } else {
            if is_alt_pressed {
                continue;
            }
            if match_single_key(&token, key) {
                return true;
            }
        }
    }

    false
}

fn parse_keybind_tokens(binding: &str) -> Vec<String> {
    let trimmed = binding.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed == "," {
        return vec![",".to_string()];
    }

    let mut tokens = Vec::new();
    let parts: Vec<&str> = trimmed.split(',').collect();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i].trim();
        if part.is_empty() {
            // An empty segment between commas or at start/end represents an escaped / literal comma key
            tokens.push(",".to_string());
            if i + 1 < parts.len() && parts[i + 1].trim().is_empty() {
                i += 1;
            }
        } else {
            tokens.push(part.to_string());
        }
        i += 1;
    }
    tokens
}

fn match_single_key(token: &str, key: &Key) -> bool {
    match key {
        Key::Named(named) => match named {
            Named::Space => token.eq_ignore_ascii_case("space") || token == " ",
            Named::ArrowUp => {
                token.eq_ignore_ascii_case("up")
                    || token.eq_ignore_ascii_case("arrowup")
                    || token == "↑"
            }
            Named::ArrowDown => {
                token.eq_ignore_ascii_case("down")
                    || token.eq_ignore_ascii_case("arrowdown")
                    || token == "↓"
            }
            Named::ArrowLeft => {
                token.eq_ignore_ascii_case("left")
                    || token.eq_ignore_ascii_case("arrowleft")
                    || token == "←"
            }
            Named::ArrowRight => {
                token.eq_ignore_ascii_case("right")
                    || token.eq_ignore_ascii_case("arrowright")
                    || token == "→"
            }
            Named::Tab => token.eq_ignore_ascii_case("tab"),
            _ => false,
        },
        Key::Character(s) => {
            if s == " " && token.eq_ignore_ascii_case("space") {
                return true;
            }
            if (s == "<" || s == ",") && (token == "<" || token == ",") {
                return true;
            }
            if (s == ">" || s == ".") && (token == ">" || token == ".") {
                return true;
            }
            s.eq_ignore_ascii_case(token)
        }
        _ => false,
    }
}

pub fn find_key_action(
    keybinds: &KeybindSettings,
    key: &Key,
    is_alt_pressed: bool,
) -> Option<KeyAction> {
    if is_alt_pressed {
        if key_matches_binding(&keybinds.close_app, key, true) {
            return Some(KeyAction::CloseApp);
        }
        if key_matches_binding(&keybinds.toggle_pin, key, true) {
            return Some(KeyAction::ToggleAlwaysOnTop);
        }
        if key_matches_binding(&keybinds.clear_mark_in, key, true) {
            return Some(KeyAction::ClearMarkIn);
        }
        if key_matches_binding(&keybinds.clear_mark_out, key, true) {
            return Some(KeyAction::ClearMarkOut);
        }
        return None;
    }

    if key_matches_binding(&keybinds.play_pause, key, false) {
        return Some(KeyAction::TogglePlayPause);
    }
    if key_matches_binding(&keybinds.prev_video, key, false) {
        return Some(KeyAction::PrevVideo);
    }
    if key_matches_binding(&keybinds.next_video, key, false) {
        return Some(KeyAction::NextVideo);
    }
    if key_matches_binding(&keybinds.seek_backward, key, false) {
        return Some(KeyAction::SeekBackward);
    }
    if key_matches_binding(&keybinds.seek_forward, key, false) {
        return Some(KeyAction::SeekForward);
    }
    if key_matches_binding(&keybinds.focus_next, key, false) {
        return Some(KeyAction::FocusNext);
    }

    if key_matches_binding(&keybinds.search_videos, key, false) {
        return Some(KeyAction::SearchVideos);
    }
    if key_matches_binding(&keybinds.add_player, key, false) {
        return Some(KeyAction::AddNewPlayer);
    }
    if key_matches_binding(&keybinds.remove_player, key, false) {
        return Some(KeyAction::RemovePlayer);
    }
    if key_matches_binding(&keybinds.toggle_layout, key, false) {
        return Some(KeyAction::CycleLayout);
    }
    if key_matches_binding(&keybinds.toggle_file_picker, key, false) {
        return Some(KeyAction::ToggleFilePicker);
    }
    if key_matches_binding(&keybinds.toggle_transcript, key, false) {
        return Some(KeyAction::ToggleTranscript);
    }
    if key_matches_binding(&keybinds.toggle_bookmarks, key, false) {
        return Some(KeyAction::ToggleBookmarks);
    }
    if key_matches_binding(&keybinds.toggle_history, key, false) {
        return Some(KeyAction::ToggleHistory);
    }
    if key_matches_binding(&keybinds.toggle_mute, key, false) {
        return Some(KeyAction::ToggleMute);
    }
    if key_matches_binding(&keybinds.toggle_play_mode, key, false) {
        return Some(KeyAction::TogglePlayMode);
    }
    if key_matches_binding(&keybinds.toggle_scroll, key, false) {
        return Some(KeyAction::ToggleScroll);
    }
    if key_matches_binding(&keybinds.toggle_flip, key, false) {
        return Some(KeyAction::ToggleFlip);
    }
    if key_matches_binding(&keybinds.toggle_subtitles, key, false) {
        return Some(KeyAction::ToggleSubtitles);
    }
    if key_matches_binding(&keybinds.prev_frame, key, false) {
        return Some(KeyAction::PrevFrame);
    }
    if key_matches_binding(&keybinds.next_frame, key, false) {
        return Some(KeyAction::NextFrame);
    }
    if key_matches_binding(&keybinds.volume_down, key, false) {
        return Some(KeyAction::VolumeDown);
    }
    if key_matches_binding(&keybinds.volume_up, key, false) {
        return Some(KeyAction::VolumeUp);
    }
    if key_matches_binding(&keybinds.speed_or_bookmark_down, key, false) {
        return Some(KeyAction::SpeedOrBookmarkDown);
    }
    if key_matches_binding(&keybinds.speed_or_bookmark_up, key, false) {
        return Some(KeyAction::SpeedOrBookmarkUp);
    }
    if key_matches_binding(&keybinds.random_seek, key, false) {
        return Some(KeyAction::RandomSeek);
    }
    if key_matches_binding(&keybinds.show_title_overlay, key, false) {
        return Some(KeyAction::ShowTitleOverlay);
    }
    if key_matches_binding(&keybinds.player_1, key, false) {
        return Some(KeyAction::Player1);
    }
    if key_matches_binding(&keybinds.player_2, key, false) {
        return Some(KeyAction::Player2);
    }
    if key_matches_binding(&keybinds.player_3, key, false) {
        return Some(KeyAction::Player3);
    }
    if key_matches_binding(&keybinds.player_4, key, false) {
        return Some(KeyAction::Player4);
    }
    if key_matches_binding(&keybinds.toggle_pin, key, false) {
        return Some(KeyAction::ToggleAlwaysOnTop);
    }
    if key_matches_binding(&keybinds.mark_in, key, false) {
        return Some(KeyAction::MarkIn);
    }
    if key_matches_binding(&keybinds.mark_out, key, false) {
        return Some(KeyAction::MarkOut);
    }

    None
}
