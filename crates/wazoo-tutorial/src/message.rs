/*!
 * ==============================================================================
 * TUTORIAL LESSON: The Elm Architecture & The `Message` Enum
 * ==============================================================================
 *
 * At the core of every Iced application is "The Elm Architecture" (TEA).
 * TEA organizes UI code into three fundamental parts:
 *
 * 1. MODEL (State): The data your application holds (defined in `app.rs`).
 * 2. VIEW: A pure function converting your State into visible UI widgets.
 * 3. UPDATE: A function handling user interactions or events to transition state.
 *
 * What connects the View to the Update function?
 * MESSAGES!
 *
 * In Rust, the `Message` type is almost always an `enum`.
 * Unlike enums in languages like C, Java, or TypeScript (which are just numbers
 * or string constants), Rust enums are "Algebraic Data Types" (sum types).
 * Each variant can hold completely different kinds of payload data!
 *
 * For example:
 * - `Increment` holds no data (unit variant).
 * - `InputChanged(String)` holds the new text typed into an input.
 * - `FilesScanned(Result<Vec<String>, String>)` holds the outcome of an async operation!
 */

use std::path::PathBuf;

/// The tabs available in our tutorial dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    CounterAndWidgets,
    AsyncMediaScanner,
    SqliteDatabase,
    ThemingPlayground,
    ReferenceGuide,
}

impl Tab {
    /// Returns a human-friendly display title for the tab header.
    pub fn title(&self) -> &'static str {
        match self {
            Tab::CounterAndWidgets => "1. Elm Architecture & Widgets",
            Tab::AsyncMediaScanner => "2. Async Scanner & RFD",
            Tab::SqliteDatabase => "3. SQLite Persistence",
            Tab::ThemingPlayground => "4. Theming & Styling",
            Tab::ReferenceGuide => "5. Rust & Iced Guide",
        }
    }

    /// Returns an icon or emoji for the tab button.
    pub fn icon(&self) -> &'static str {
        match self {
            Tab::CounterAndWidgets => "🎛️",
            Tab::AsyncMediaScanner => "🔍",
            Tab::SqliteDatabase => "💾",
            Tab::ThemingPlayground => "🎨",
            Tab::ReferenceGuide => "📖",
        }
    }
}

/// A scanned file result item.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannedItem {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub is_video: bool,
}

/// The unified `Message` enum for the entire tutorial application.
///
/// Note the derives:
/// - `#[derive(Debug)]`: Allows printing messages for debugging/logging.
/// - `#[derive(Clone)]`: Allows Iced to clone messages when dispatching events.
#[derive(Debug, Clone)]
pub enum Message {
    // --------------------------------------------------------------------------
    // Navigation Messages
    // --------------------------------------------------------------------------
    /// Switched active tab
    TabSelected(Tab),

    // --------------------------------------------------------------------------
    // Tab 1: Counter & Interactive Widgets
    // --------------------------------------------------------------------------
    /// User clicked the '+' button
    IncrementCounter,
    /// User clicked the '-' button
    DecrementCounter,
    /// User clicked the 'Reset' button
    ResetCounter,
    /// User typed into the text input box (payload is the current string)
    TextInputChanged(String),
    /// User moved the volume/progress slider (payload is f32 between 0.0 and 100.0)
    SliderChanged(f32),
    /// User toggled the checkbox (payload is the new boolean state)
    CheckboxToggled(bool),

    // --------------------------------------------------------------------------
    // Tab 2: Async Background Scanner & Native File Dialogs (like wazoo-scanner)
    // --------------------------------------------------------------------------
    /// User clicked "Pick Folder" -> triggers native OS folder picker via `rfd`
    PickFolderClicked,
    /// The async `rfd` file dialog finished and returned the chosen folder path (or None if cancelled)
    FolderChosen(Option<String>),
    /// User clicked "Start Scan" -> spawns async background file scanner task
    StartScanClicked,
    /// The background async scanner completed with either Ok(items) or Err(error_message)
    ScanFinished(Result<Vec<ScannedItem>, String>),
    /// User changed the search filter input on the scanned items list
    FilterQueryChanged(String),

    // --------------------------------------------------------------------------
    // Tab 3: Embedded SQLite Persistence (like wazoo-core)
    // --------------------------------------------------------------------------
    /// User changed the note title input field
    NoteTitleChanged(String),
    /// User changed the note body input field
    NoteContentChanged(String),
    /// User clicked "Save Note to SQLite"
    SaveNoteClicked,
    /// User clicked "Delete" on a specific note row (payload is the SQLite row ID)
    DeleteNoteClicked(i64),
    /// User clicked "Refresh Notes List"
    RefreshNotesClicked,

    // --------------------------------------------------------------------------
    // Tab 4: Theming & Custom Styling
    // --------------------------------------------------------------------------
    /// User selected a theme from the pick list
    ThemeSelected(iced::Theme),

    // --------------------------------------------------------------------------
    // Global App Events
    // --------------------------------------------------------------------------
    /// A 1-second periodic tick subscription event (used to update an in-app uptime counter)
    Tick,
}
