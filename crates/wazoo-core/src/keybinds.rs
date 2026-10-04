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
    // --- 1. Playback & Seeking ---
    TogglePlayPause,
    SeekBackward,
    SeekForward,
    PrevFrame,
    NextFrame,
    PrevVideo,
    NextVideo,
    RandomSeek,
    TogglePlayMode,
    MarkIn,
    MarkOut,
    ClearMarkIn,
    ClearMarkOut,

    // --- 2. Panels & Drawers ---
    ToggleFilePicker,
    SearchVideos,
    ToggleBookmarks,
    SpeedOrBookmarkUp,
    SpeedOrBookmarkDown,
    ToggleHistory,
    ToggleTranscript,

    // --- 3. Grid & Multi-Player ---
    CycleLayout,
    AddNewPlayer,
    RemovePlayer,
    FocusNext,
    Player1,
    Player2,
    Player3,
    Player4,

    // --- 4. Visual Modes & Effects ---
    ToggleScroll,
    ToggleFlip,
    ToggleFilters,
    ToggleCube,
    ToggleCubeOverlay,

    // --- 5. Audio & Display ---
    VolumeDown,
    VolumeUp,
    ToggleMute,
    ToggleSubtitles,
    ShowTitleOverlay,

    // --- 6. System & Window ---
    ToggleAlwaysOnTop,
    FitWindow,
    ToggleFullscreen,
    CloseApp,
    OpenHelp,
    OpenSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindSettings {
    // --- 1. Playback & Seeking ---
    pub play_pause: String,
    pub seek_backward: String,
    pub seek_forward: String,
    pub prev_frame: String,
    pub next_frame: String,
    pub prev_video: String,
    pub next_video: String,
    pub random_seek: String,
    pub toggle_play_mode: String,
    pub mark_in: String,
    pub mark_out: String,
    pub clear_mark_in: String,
    pub clear_mark_out: String,

    // --- 2. Panels & Drawers ---
    pub toggle_file_picker: String,
    pub search_videos: String,
    pub toggle_bookmarks: String,
    pub speed_or_bookmark_up: String,
    pub speed_or_bookmark_down: String,
    pub toggle_history: String,
    pub toggle_transcript: String,

    // --- 3. Grid & Multi-Player ---
    pub toggle_layout: String,
    pub add_player: String,
    pub remove_player: String,
    pub focus_next: String,
    pub player_1: String,
    pub player_2: String,
    pub player_3: String,
    pub player_4: String,

    // --- 4. Visual Modes & Effects ---
    pub toggle_scroll: String,
    pub toggle_flip: String,
    #[serde(alias = "toggle_crt")]
    pub toggle_filters: String,
    pub toggle_cube: String,
    pub toggle_cube_overlay: String,

    // --- 5. Audio & Display ---
    pub volume_down: String,
    pub volume_up: String,
    pub toggle_mute: String,
    pub toggle_subtitles: String,
    pub show_title_overlay: String,

    // --- 6. System & Window ---
    pub toggle_pin: String,
    #[serde(alias = "resize_to_fit")]
    pub fit_window: String,
    pub toggle_fullscreen: String,
    pub close_app: String,
    pub open_help: String,
    pub open_settings: String,
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            // --- 1. Playback & Seeking ---
            play_pause: "Space".to_string(),
            seek_backward: "Left".to_string(),
            seek_forward: "Right".to_string(),
            prev_frame: ",".to_string(),
            next_frame: ".".to_string(),
            prev_video: "Down".to_string(),
            next_video: "Up".to_string(),
            random_seek: "r".to_string(),
            toggle_play_mode: "s".to_string(),
            mark_in: "i".to_string(),
            mark_out: "o".to_string(),
            clear_mark_in: "Alt+I".to_string(),
            clear_mark_out: "Alt+O".to_string(),

            // --- 2. Panels & Drawers ---
            toggle_file_picker: "h, f".to_string(),
            search_videos: "j, /".to_string(),
            toggle_bookmarks: "b".to_string(),
            speed_or_bookmark_up: "+, =".to_string(),
            speed_or_bookmark_down: "-".to_string(),
            toggle_history: "y".to_string(),
            toggle_transcript: "v".to_string(),

            // --- 3. Grid & Multi-Player ---
            toggle_layout: "l".to_string(),
            add_player: "n".to_string(),
            remove_player: "x".to_string(),
            focus_next: "Tab".to_string(),
            player_1: "1".to_string(),
            player_2: "2".to_string(),
            player_3: "3".to_string(),
            player_4: "4".to_string(),

            // --- 4. Visual Modes & Effects ---
            toggle_scroll: "5".to_string(),
            toggle_flip: "6".to_string(),
            toggle_filters: "7".to_string(),
            toggle_cube: "8".to_string(),
            toggle_cube_overlay: "9".to_string(),

            // --- 5. Audio & Display ---
            volume_down: "[".to_string(),
            volume_up: "]".to_string(),
            toggle_mute: "m".to_string(),
            toggle_subtitles: "c".to_string(),
            show_title_overlay: "t".to_string(),

            // --- 6. System & Window ---
            toggle_pin: "p, Alt+P".to_string(),
            fit_window: "w".to_string(),
            toggle_fullscreen: "F11".to_string(),
            close_app: "Alt+X".to_string(),
            open_help: "F1".to_string(),
            open_settings: "F2".to_string(),
        }
    }
}

impl KeybindSettings {
    pub const ALL_KEYS: &'static [&'static str] = &[
        // 1. Playback & Seeking
        "play_pause",
        "seek_backward",
        "seek_forward",
        "prev_frame",
        "next_frame",
        "prev_video",
        "next_video",
        "random_seek",
        "toggle_play_mode",
        "mark_in",
        "mark_out",
        "clear_mark_in",
        "clear_mark_out",
        // 2. Panels & Drawers
        "toggle_file_picker",
        "search_videos",
        "toggle_bookmarks",
        "speed_or_bookmark_up",
        "speed_or_bookmark_down",
        "toggle_history",
        "toggle_transcript",
        // 3. Grid & Multi-Player
        "toggle_layout",
        "add_player",
        "remove_player",
        "focus_next",
        "player_1",
        "player_2",
        "player_3",
        "player_4",
        // 4. Visual Modes & Effects
        "toggle_scroll",
        "toggle_flip",
        "toggle_filters",
        "toggle_cube",
        "toggle_cube_overlay",
        // 5. Audio & Display
        "volume_down",
        "volume_up",
        "toggle_mute",
        "toggle_subtitles",
        "show_title_overlay",
        // 6. System & Window
        "toggle_pin",
        "fit_window",
        "toggle_fullscreen",
        "close_app",
        "open_help",
        "open_settings",
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

        // 1. Playback & Seeking
        if self.play_pause.trim().is_empty() {
            self.play_pause = def.play_pause;
        }
        if self.seek_backward.trim().is_empty() {
            self.seek_backward = def.seek_backward;
        }
        if self.seek_forward.trim().is_empty() {
            self.seek_forward = def.seek_forward;
        }
        if self.prev_frame.trim().is_empty() {
            self.prev_frame = def.prev_frame;
        }
        if self.next_frame.trim().is_empty() {
            self.next_frame = def.next_frame;
        }
        if self.prev_video.trim().is_empty() {
            self.prev_video = def.prev_video;
        }
        if self.next_video.trim().is_empty() {
            self.next_video = def.next_video;
        }
        if self.random_seek.trim().is_empty() {
            self.random_seek = def.random_seek;
        }
        if self.toggle_play_mode.trim().is_empty() {
            self.toggle_play_mode = def.toggle_play_mode;
        }
        if self.mark_in.trim().is_empty() {
            self.mark_in = def.mark_in;
        }
        if self.mark_out.trim().is_empty() {
            self.mark_out = def.mark_out;
        }
        if self.clear_mark_in.trim().is_empty() {
            self.clear_mark_in = def.clear_mark_in;
        }
        if self.clear_mark_out.trim().is_empty() {
            self.clear_mark_out = def.clear_mark_out;
        }

        // 2. Panels & Drawers
        if self.toggle_file_picker.trim().is_empty() {
            self.toggle_file_picker = def.toggle_file_picker;
        }
        if self.search_videos.trim().is_empty() {
            self.search_videos = def.search_videos;
        }
        if self.toggle_bookmarks.trim().is_empty() {
            self.toggle_bookmarks = def.toggle_bookmarks;
        }
        if self.speed_or_bookmark_up.trim().is_empty() {
            self.speed_or_bookmark_up = def.speed_or_bookmark_up;
        }
        if self.speed_or_bookmark_down.trim().is_empty() {
            self.speed_or_bookmark_down = def.speed_or_bookmark_down;
        }
        if self.toggle_history.trim().is_empty() {
            self.toggle_history = def.toggle_history;
        }
        if self.toggle_transcript.trim().is_empty() {
            self.toggle_transcript = def.toggle_transcript;
        }

        // 3. Grid & Multi-Player
        if self.toggle_layout.trim().is_empty() {
            self.toggle_layout = def.toggle_layout;
        }
        if self.add_player.trim().is_empty() {
            self.add_player = def.add_player;
        }
        if self.remove_player.trim().is_empty() {
            self.remove_player = def.remove_player;
        }
        if self.focus_next.trim().is_empty() {
            self.focus_next = def.focus_next;
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

        // 4. Visual Modes & Effects
        if self.toggle_scroll.trim().is_empty() {
            self.toggle_scroll = def.toggle_scroll;
        }
        if self.toggle_flip.trim().is_empty() {
            self.toggle_flip = def.toggle_flip;
        }
        if self.toggle_filters.trim().is_empty() {
            self.toggle_filters = def.toggle_filters;
        }
        if self.toggle_cube.trim().is_empty() {
            self.toggle_cube = def.toggle_cube;
        }
        if self.toggle_cube_overlay.trim().is_empty() {
            self.toggle_cube_overlay = def.toggle_cube_overlay;
        }

        // 5. Audio & Display
        if self.volume_down.trim().is_empty() {
            self.volume_down = def.volume_down;
        }
        if self.volume_up.trim().is_empty() {
            self.volume_up = def.volume_up;
        }
        if self.toggle_mute.trim().is_empty() {
            self.toggle_mute = def.toggle_mute;
        }
        if self.toggle_subtitles.trim().is_empty() {
            self.toggle_subtitles = def.toggle_subtitles;
        }
        if self.show_title_overlay.trim().is_empty() {
            self.show_title_overlay = def.show_title_overlay;
        }

        // 6. System & Window
        if self.toggle_pin.trim().is_empty() {
            self.toggle_pin = def.toggle_pin;
        }
        if self.fit_window.trim().is_empty() {
            self.fit_window = def.fit_window;
        }
        if self.toggle_fullscreen.trim().is_empty() {
            self.toggle_fullscreen = def.toggle_fullscreen;
        }
        if self.close_app.trim().is_empty() {
            self.close_app = def.close_app;
        }
        if self.open_help.trim().is_empty() {
            self.open_help = def.open_help;
        }
        if self.open_settings.trim().is_empty() {
            self.open_settings = def.open_settings;
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

    /// Formats the complete shortcuts list with localized action descriptions.
    /// Derived directly from the systemic help categories to ensure consistency.
    pub fn help_shortcuts_with_descriptions(
        &self,
        t: impl Fn(&str) -> String,
    ) -> Vec<(String, String)> {
        self.help_categories(|k| t(k))
            .into_iter()
            .flat_map(|cat| {
                cat.shortcuts
                    .into_iter()
                    .map(|s| (s.key.to_display_string("OR"), s.description))
            })
            .collect()
    }

    /// Returns the systemic, categorized list of keyboard shortcuts for the help reference screen.
    pub fn help_categories(&self, t: impl Fn(&str) -> String) -> Vec<HelpCategory> {
        vec![
            // Category 0: Playback & Seeking
            HelpCategory {
                title: t("help.categories.playback"),
                icon: "🎬",
                shortcuts: vec![
                    HelpShortcut {
                        key: KeyDisplay::single(&self.play_pause),
                        description: t("help.shortcuts.play_pause"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::seek_pair(&self.seek_backward, &self.seek_forward),
                        description: t("help.shortcuts.seek_back_forward"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::frame_pair(&self.prev_frame, &self.next_frame),
                        description: t("help.shortcuts.prev_next_frame"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::video_pair(&self.prev_video, &self.next_video),
                        description: t("help.shortcuts.prev_next_video"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.random_seek),
                        description: t("help.shortcuts.random_seek"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_play_mode),
                        description: t("help.shortcuts.toggle_play_mode"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.mark_in),
                        description: t("help.shortcuts.mark_in"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.mark_out),
                        description: t("help.shortcuts.mark_out"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::Combo(vec!["Alt".to_string(), "I / O".to_string()]),
                        description: t("help.shortcuts.clear_marks"),
                    },
                ],
            },
            // Category 1: Panels & Drawers
            HelpCategory {
                title: t("help.categories.drawers"),
                icon: "📁",
                shortcuts: vec![
                    HelpShortcut {
                        key: KeyDisplay::alternatives_from_binding(&self.toggle_file_picker),
                        description: t("help.shortcuts.toggle_file_picker"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::alternatives_from_binding(&self.search_videos),
                        description: t("help.shortcuts.search_videos"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_bookmarks),
                        description: t("help.shortcuts.toggle_bookmarks"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::alternatives_from_binding(&self.speed_or_bookmark_up),
                        description: t("bookmarks.bookmark_current"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_history),
                        description: t("help.shortcuts.toggle_history"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_transcript),
                        description: t("help.shortcuts.toggle_transcript"),
                    },
                ],
            },
            // Category 2: Grid & Multi-Player
            HelpCategory {
                title: t("help.categories.layout"),
                icon: "⊞",
                shortcuts: vec![
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_layout),
                        description: t("help.shortcuts.toggle_layout"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.add_player),
                        description: t("help.shortcuts.add_player"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.remove_player),
                        description: t("help.shortcuts.remove_player"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.focus_next),
                        description: t("help.shortcuts.focus_next"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_scroll),
                        description: t("help.shortcuts.toggle_scroll"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_flip),
                        description: t("help.shortcuts.toggle_flip"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_filters),
                        description: t("help.shortcuts.toggle_filters"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_cube),
                        description: t("help.shortcuts.toggle_cube"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_cube_overlay),
                        description: t("help.shortcuts.toggle_cube_overlay"),
                    },
                ],
            },
            // Category 3: Audio & Display
            HelpCategory {
                title: t("help.categories.audio"),
                icon: "🔊",
                shortcuts: vec![
                    HelpShortcut {
                        key: KeyDisplay::volume_pair(&self.volume_down, &self.volume_up),
                        description: t("help.shortcuts.adjust_volume"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_mute),
                        description: t("help.shortcuts.toggle_mute"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_subtitles),
                        description: t("help.shortcuts.toggle_subtitles"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.show_title_overlay),
                        description: t("help.shortcuts.show_title_overlay"),
                    },
                ],
            },
            // Category 4: System & Window
            HelpCategory {
                title: t("help.categories.system"),
                icon: "⚙️",
                shortcuts: vec![
                    HelpShortcut {
                        key: KeyDisplay::single("P"),
                        description: t("help.shortcuts.toggle_pin"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::Combo(vec!["Alt".to_string(), "Drag".to_string()]),
                        description: t("help.shortcuts.move_window"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.fit_window),
                        description: t("help.shortcuts.fit_window"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::single(&self.toggle_fullscreen),
                        description: t("help.shortcuts.toggle_fullscreen"),
                    },
                    HelpShortcut {
                        key: KeyDisplay::combo_from_binding(&self.close_app),
                        description: t("help.shortcuts.close_app"),
                    },
                ],
            },
        ]
    }
}

/// Visual representation of keyboard shortcuts for UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyDisplay {
    /// Single keycap, e.g. "Space", "Tab", "M"
    Single(String),
    /// Alternative keys (press either), e.g. ["H", "F"], ["J", "/"]
    Alternatives(Vec<String>),
    /// Paired directional/step keys, e.g. ("←", "→"), ("[", "]")
    Pair(String, String),
    /// Key combination with modifier, e.g. ["Alt", "X"]
    Combo(Vec<String>),
}

impl KeyDisplay {
    /// Normalizes and formats a single key binding into a clean keycap label.
    pub fn single(key: &str) -> Self {
        let trimmed = key.trim();
        let formatted = if trimmed.eq_ignore_ascii_case("space") {
            "Space".to_string()
        } else if trimmed.eq_ignore_ascii_case("tab") {
            "Tab".to_string()
        } else if trimmed.to_lowercase().starts_with("alt+") {
            return Self::combo_from_binding(trimmed);
        } else {
            trimmed.to_uppercase()
        };
        Self::Single(formatted)
    }

    /// Creates a pair of complementary keys (e.g. directional arrows or bracket pairs).
    pub fn pair(first: impl Into<String>, second: impl Into<String>) -> Self {
        Self::Pair(first.into(), second.into())
    }

    /// Resolves seek backward/forward display (defaults to visual arrows "← →").
    pub fn seek_pair(backward: &str, forward: &str) -> Self {
        if backward.eq_ignore_ascii_case("left") && forward.eq_ignore_ascii_case("right") {
            Self::Pair("←".to_string(), "→".to_string())
        } else {
            Self::Pair(backward.to_uppercase(), forward.to_uppercase())
        }
    }

    /// Resolves previous/next video display (defaults to visual arrows "↓ ↑").
    pub fn video_pair(prev: &str, next: &str) -> Self {
        if (prev.eq_ignore_ascii_case("down") && next.eq_ignore_ascii_case("up"))
            || (prev.eq_ignore_ascii_case("up") && next.eq_ignore_ascii_case("down"))
        {
            Self::Pair("↓".to_string(), "↑".to_string())
        } else {
            Self::Pair(prev.to_uppercase(), next.to_uppercase())
        }
    }

    /// Resolves frame stepping display (defaults to "< >").
    pub fn frame_pair(prev: &str, next: &str) -> Self {
        if (prev == "," || prev == "<") && (next == "." || next == ">") {
            Self::Pair("<".to_string(), ">".to_string())
        } else {
            Self::Pair(prev.to_uppercase(), next.to_uppercase())
        }
    }

    /// Resolves volume display (defaults to "[ ]").
    pub fn volume_pair(down: &str, up: &str) -> Self {
        if down == "[" && up == "]" {
            Self::Pair("[".to_string(), "]".to_string())
        } else {
            Self::Pair(down.to_uppercase(), up.to_uppercase())
        }
    }

    /// Parses comma-separated alternative keys cleanly into an Alternatives display.
    pub fn alternatives_from_binding(binding: &str) -> Self {
        let trimmed = binding.trim();
        if trimmed == "," {
            return Self::Single(",".to_string());
        }
        let keys: Vec<String> = trimmed
            .split(',')
            .map(|s| {
                let t = s.trim();
                if t.eq_ignore_ascii_case("space") {
                    "Space".to_string()
                } else if t.to_lowercase().starts_with("alt+") {
                    let rest = &t[4..];
                    format!("Alt+{}", rest.to_uppercase())
                } else {
                    t.to_uppercase()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        if keys.len() == 1 {
            Self::Single(keys.into_iter().next().unwrap())
        } else {
            Self::Alternatives(keys)
        }
    }

    /// Parses a combination binding such as "Alt+X" into a Combo display.
    pub fn combo_from_binding(binding: &str) -> Self {
        let parts: Vec<String> = binding
            .split('+')
            .map(|s| {
                let t = s.trim();
                if t.eq_ignore_ascii_case("alt") {
                    "Alt".to_string()
                } else if t.eq_ignore_ascii_case("ctrl") {
                    "Ctrl".to_string()
                } else {
                    t.to_uppercase()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();
        Self::Combo(parts)
    }

    /// Formats the key display as a human-readable text string.
    pub fn to_display_string(&self, or_text: &str) -> String {
        match self {
            KeyDisplay::Single(k) => k.clone(),
            KeyDisplay::Pair(k1, k2) => format!("{} {}", k1, k2),
            KeyDisplay::Alternatives(keys) => keys.join(&format!(" {} ", or_text)),
            KeyDisplay::Combo(keys) => keys.join(" + "),
        }
    }
}

/// A localized keyboard shortcut entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpShortcut {
    pub key: KeyDisplay,
    pub description: String,
}

/// A functional grouping of related keyboard shortcuts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpCategory {
    pub title: String,
    pub icon: &'static str,
    pub shortcuts: Vec<HelpShortcut>,
}
