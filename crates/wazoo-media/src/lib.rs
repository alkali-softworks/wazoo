/**
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

pub use player::{BufferConfig, PlayerId, PlayerState, VideoHandle};
pub use scroll::{ScrollEngine, ScrollItem};
pub use subtitles::{load_subtitles, load_subtitles_sync, SubtitleCue};
