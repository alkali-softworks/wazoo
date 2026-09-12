/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Application Message Definitions
 * 
 * Enumerates all user interactions, playback commands, background scanner updates,
 * timer ticks, and window system events routed through the Iced application loop.
 */

use std::sync::{Arc, Mutex};
use std::time::Duration;
use iced::{keyboard::Key, Point, Size, window::Id, event::Status};
use wazoo_core::Bookmark;
use wazoo_media::{PlayerId, VideoHandle};
use wazoo_scanner::ScanProgress;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    // Window management & Events
    WindowIdReceived(Id),
    WindowResized(Id, Size),
    GainWindowFocus,
    PreloadedPlayerReady(Arc<Mutex<Option<Result<VideoHandle, String>>>>),
    CursorMoved(Id, Point),
    RightClickPressed(Id),
    KeyPressed(Key, Status),
    KeyReleased(Key),
    MinimizeWindow,
    MaximizeWindow,
    CloseApp,
    DragWindow,
    TitleBarPressed,
    LeftClickReleased,
    ToggleDropdownMenu,
    CloseDropdownMenu,
    OpenMenuModal,
    CloseMenuModal,
    ToggleFilePicker,
    ToggleFolderCollapse(String),
    FilePickerSearchChanged(String),
    PlayFileInFocused(String),
    SelectSearchFolder(String),
    SetWindowOpacity(f32),

    // Playback controls
    PlayerClicked(PlayerId),
    TogglePlay(PlayerId),
    TogglePlayFocused,
    NextVideo(PlayerId),
    NextVideoFocused,
    PrevVideoFocused,
    Seek(PlayerId, Duration),
    SeekRatio(PlayerId, f32),
    SeekRelativeFocused(f64),
    PlayerHovered(PlayerId),
    PlayerUnhovered(PlayerId),
    SetVolume(PlayerId, f64),
    AdjustVolumeFocused(f64),
    TogglePlayerMute(PlayerId),
    ToggleMuteFocused,
    ToggleGlobalMute,
    GlobalUnmute,
    ToggleShuffleMode,

    // Layout & Modes
    CycleLayout,
    SetPlayerCount(usize),
    ToggleScrollMode,
    ToggleFlipMode,
    SetScrollSpeed(f32),
    AdjustScrollSpeed(f32),
    AddNewPlayer,
    RemoveFocusedPlayer,
    CycleFocusedPlayer,
    SetFocusedPlayer(usize),

    // Modals & UI
    OpenSearchModal,
    CloseSearchModal,
    SearchInputChanged(String),
    PerformSearch,
    OpenSettingsModal,
    CloseSettingsModal,
    SetBufferDuration(u32),
    OpenHelpModal,
    CloseHelpModal,
    FolderInputChanged(String),
    PickFolders,
    FoldersSelected(Vec<String>),
    AddMediaFolder,
    RemoveMediaFolder(String),
    StartScan,
    ScanProgressUpdate(ScanProgress),
    ScanFinished(Result<usize, String>),
    ToggleSubtitles,
    ToggleTranscript,
    ToggleTranscriptForPlayer(PlayerId),
    CloseTranscript,
    TranscriptSearchChanged(String),
    TranscriptLoaded(String, Vec<wazoo_media::SubtitleCue>),
    SeekToSubtitle(f64),
    EscapePressed,

    // Bookmarks & wazoo-js shortcuts
    ToggleBookmarksModal,
    CloseBookmarksModal,
    AddBookmarkFocused,
    RemoveBookmarkFocused,
    RemoveBookmark(usize),
    JumpToBookmark(Bookmark),
    RandomSeekFocused,
    ShowTitleOverlayFocused,

    // Timers & Ticks
    AnimationTick,
    VideoFrameTick,
    WatchdogTick,
    FlipModeTick,
    DismissToast,
}
