/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Application Player Entity & Collection Extensions
 *
 * Encapsulates the media engine handle together with player-specific UI state:
 * per-player shuffle mode, navigation undo/redo history, and loading indicators.
 */

use crate::app::PlayerNavHistory;
use std::ops::{Deref, DerefMut};
use wazoo_media::{PlayerId, VideoHandle};

/// Encapsulates all state pertaining to a single active player in the application.
#[derive(Debug)]
pub struct AppPlayer {
    /// The underlying media engine handle (mpv backend, frame rendering, tracks)
    pub handle: VideoHandle,
    /// Per-player shuffle mode (random vs sequential video selection)
    pub shuffle: bool,
    /// Per-player navigation history (back and forward stacks)
    pub nav_history: PlayerNavHistory,
    /// Whether this player is currently buffering or loading a new video
    pub is_loading: bool,
    /// Ticks elapsed while loading (used to auto-dismiss stuck loading states)
    pub loading_ticks: usize,
}

pub type Player = AppPlayer;

impl AppPlayer {
    /// Creates a new `AppPlayer` wrapping a `VideoHandle` with the specified shuffle setting.
    pub fn new(handle: VideoHandle, shuffle: bool) -> Self {
        Self {
            handle,
            shuffle,
            nav_history: PlayerNavHistory::default(),
            is_loading: false,
            loading_ticks: 0,
        }
    }

    /// Marks this player as loading.
    #[inline]
    pub fn start_loading(&mut self) {
        self.is_loading = true;
        self.loading_ticks = 0;
    }

    /// Clears the loading state on this player.
    #[inline]
    pub fn stop_loading(&mut self) {
        self.is_loading = false;
        self.loading_ticks = 0;
    }

    /// Returns whether this player is currently loading.
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
}

impl Deref for AppPlayer {
    type Target = VideoHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for AppPlayer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

/// A collection of active players in the application.
///
/// Wraps `Vec<AppPlayer>` and provides inherent methods for player lookup
/// by `PlayerId`, index finding, and element access without requiring extension traits.
#[derive(Debug, Default)]
pub struct PlayerList(pub Vec<AppPlayer>);

impl PlayerList {
    /// Creates a new empty `PlayerList`.
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns a reference to the player with the specified ID, if it exists.
    #[inline]
    pub fn player(&self, id: PlayerId) -> Option<&AppPlayer> {
        self.0.iter().find(|p| p.id == id)
    }

    /// Returns a mutable reference to the player with the specified ID, if it exists.
    #[inline]
    pub fn player_mut(&mut self, id: PlayerId) -> Option<&mut AppPlayer> {
        self.0.iter_mut().find(|p| p.id == id)
    }

    /// Returns the slice index of the player with the specified ID, if it exists.
    #[inline]
    pub fn player_index(&self, id: PlayerId) -> Option<usize> {
        self.0.iter().position(|p| p.id == id)
    }

    /// Returns whether a player with the specified ID is present in the collection.
    #[inline]
    pub fn has_player(&self, id: PlayerId) -> bool {
        self.0.iter().any(|p| p.id == id)
    }
}

impl Deref for PlayerList {
    type Target = Vec<AppPlayer>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlayerList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Index<usize> for PlayerList {
    type Output = AppPlayer;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for PlayerList {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<'a> IntoIterator for &'a PlayerList {
    type Item = &'a AppPlayer;
    type IntoIter = std::slice::Iter<'a, AppPlayer>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut PlayerList {
    type Item = &'a mut AppPlayer;
    type IntoIter = std::slice::IterMut<'a, AppPlayer>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl IntoIterator for PlayerList {
    type Item = AppPlayer;
    type IntoIter = std::vec::IntoIter<AppPlayer>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<AppPlayer>> for PlayerList {
    #[inline]
    fn from(vec: Vec<AppPlayer>) -> Self {
        Self(vec)
    }
}

impl From<PlayerList> for Vec<AppPlayer> {
    #[inline]
    fn from(list: PlayerList) -> Self {
        list.0
    }
}

impl AsRef<[AppPlayer]> for PlayerList {
    #[inline]
    fn as_ref(&self) -> &[AppPlayer] {
        &self.0
    }
}

impl AsMut<[AppPlayer]> for PlayerList {
    #[inline]
    fn as_mut(&mut self) -> &mut [AppPlayer] {
        &mut self.0
    }
}
