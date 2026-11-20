/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Subsystem
 *
 * Modular player components: track representations and language parsing,
 * player state & buffer configuration, and the libmpv VideoHandle playback engine.
 */

mod equalizer;
pub mod handle;
mod mpv_client;
mod playback;
mod render;
pub mod state;
mod track_control;
pub mod tracks;

pub use handle::*;
pub use state::*;
pub use tracks::*;
