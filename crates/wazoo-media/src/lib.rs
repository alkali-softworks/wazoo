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
    AudioTrack, BufferConfig, PlayerId, PlayerState, StartTime, SubtitleTrack, VideoHandle,
    build_alang_string, build_slang_string, build_slang_string_with_fallback,
    find_matching_audio_track, find_matching_subtitle_track, format_audio_track_label,
    format_subtitle_track_label, get_subtitle_track_preference_string,
    get_track_preference_string, is_forced_track, is_signs_or_songs_track, language_aliases,
    language_display_name, matches_alias_token, select_best_subtitle_track,
    select_best_subtitle_track_with_fallback, subtitle_track_matches_preference,
    track_matches_preference,
};
pub use scroll::{ScrollEngine, ScrollItem};
pub use subtitles::{
    SubtitleCue, load_subtitles, load_subtitles_for_stream, load_subtitles_for_stream_sync,
    load_subtitles_for_track, load_subtitles_for_track_details, load_subtitles_for_track_sync,
    load_subtitles_sync, parse_subtitles, run_ffmpeg_subtitle_extract_async,
};
