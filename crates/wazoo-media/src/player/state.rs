/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Player State & Buffer Configuration
 */

use super::tracks::{AudioTrack, PlayerId, SubtitleTrack};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub path: String,
    pub name: String,
    pub duration: Duration,
    pub position: Duration,
    pub volume: f64,
    pub is_muted: bool,
    pub is_playing: bool,
    pub last_checked_pos: Duration,
    pub stuck_count: usize,
    pub audio_tracks: Vec<AudioTrack>,
    pub current_audio_track_id: Option<i64>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub current_subtitle_track_id: Option<i64>,
    pub mark_in: Option<Duration>,
    pub mark_out: Option<Duration>,
}

impl PlayerState {
    pub fn new(id: PlayerId, path: String, name: String) -> Self {
        Self {
            id,
            path,
            name,
            duration: Duration::ZERO,
            position: Duration::ZERO,
            volume: 1.0,
            is_muted: false,
            is_playing: true,
            last_checked_pos: Duration::ZERO,
            stuck_count: 0,
            audio_tracks: Vec::new(),
            current_audio_track_id: None,
            subtitle_tracks: Vec::new(),
            current_subtitle_track_id: None,
            mark_in: None,
            mark_out: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub duration_secs: u32,
    pub size_mb: u32,
    pub read_chunk_kb: u32,
    pub preferred_audio_language: Option<String>,
    pub gamma: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub saturation: f64,
    pub playback_speed: f64,
    pub crt_enabled: bool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            duration_secs: 5,
            size_mb: 16,
            read_chunk_kb: 512,
            preferred_audio_language: None,
            gamma: 0.0,
            contrast: 0.0,
            brightness: 0.0,
            saturation: 0.0,
            playback_speed: 1.0,
            crt_enabled: false,
        }
    }
}
