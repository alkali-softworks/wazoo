/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Player Subsystem
 *
 * Modular player components: track representations and language parsing,
 * player state & buffer configuration, and the libmpv VideoHandle playback engine.
 */

pub mod handle;
pub mod state;
pub mod tracks;

pub use handle::*;
pub use state::*;
pub use tracks::*;
