/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Side Drawers State (File Picker, Transcript, History)
 */

use crate::app::PlayHistoryItem;
use crate::views::file_picker::{FilePickerGroup, PrecomputedVideoMeta};
use std::collections::HashSet;
use wazoo_media::SubtitleCue;

#[derive(Debug, Clone, Default)]
pub struct DrawerState {
    // File Picker
    pub show_file_picker: bool,
    pub file_picker_search: String,
    pub file_picker_debounce_ticks: usize,
    pub(crate) file_picker_entries: Vec<PrecomputedVideoMeta>,
    pub(crate) file_picker_groups: Vec<FilePickerGroup>,
    pub expanded_folders: HashSet<String>,

    // Subtitles Transcript
    pub show_transcript: bool,
    pub transcript_cues: Vec<SubtitleCue>,
    pub transcript_search: String,
    pub transcript_loading: bool,
    pub transcript_video_path: Option<String>,
    pub transcript_track_index: usize,
    pub show_transcript_menu: bool,

    // Play History
    pub show_history_drawer: bool,
    pub play_history: Vec<PlayHistoryItem>,
    pub history_search: String,
}

impl DrawerState {
    pub fn is_any_open(&self) -> bool {
        self.show_file_picker || self.show_transcript || self.show_history_drawer
    }

    pub fn close_all(&mut self) {
        self.show_file_picker = false;
        self.show_transcript = false;
        self.show_transcript_menu = false;
        self.show_history_drawer = false;
    }
}
