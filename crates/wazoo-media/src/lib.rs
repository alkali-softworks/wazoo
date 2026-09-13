/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Media Subsystem Root
 * 
 * Exposes the high-level VideoHandle, playback engine, buffer configuration, and continuous
 * vertical scroll engine abstractions.
 */

pub mod mpv_ffi;
pub mod pipeline;
pub mod player;
pub mod scroll;
pub mod subtitles;

pub use player::{
    build_alang_string, find_matching_audio_track, format_audio_track_label, format_subtitle_track_label,
    get_track_preference_string, language_aliases, language_display_name, AudioTrack, BufferConfig,
    PlayerId, PlayerState, SubtitleTrack, VideoHandle,
};
pub use scroll::{ScrollEngine, ScrollItem};
pub use subtitles::{
    load_subtitles, load_subtitles_for_stream, load_subtitles_for_stream_sync, load_subtitles_for_track,
    load_subtitles_for_track_sync, load_subtitles_sync, parse_subtitles, SubtitleCue,
};
