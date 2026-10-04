/*!
 * ==============================================================================
 * TUTORIAL LESSON: The Central Application State & Lifecycle
 * ==============================================================================
 *
 * This file contains the primary `TutorialApp` state struct and the core functions
 * required by the `iced::application` runtime:
 *
 * 1. `new()`: Constructs initial state and returns an initial async `Task`.
 * 2. `update()`: Mutates state in response to `Message` events.
 * 3. `view()`: Purely renders the UI based on current state.
 * 4. `subscription()`: Listens to external/runtime event streams (e.g. timers, keyboard).
 * 5. `theme()`: Informs Iced of the active color scheme.
 *
 * RUST CONCEPT: The Single Source of Truth
 * Unlike frameworks that scatter state across many individual UI components
 * (leading to sync bugs and race conditions), TEA stores the ENTIRE application
 * state in a single, coherent struct (`TutorialApp`).
 */

use iced::{
    Alignment, Element, Length, Subscription, Task, Theme, time,
    widget::{button, column, container, row, rule, text},
};
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

use crate::db::{NoteRecord, TutorialDatabase};
use crate::message::{Message, ScannedItem, Tab};
use crate::style;

/// Video file extensions commonly indexed in media players like `wazoo-rs`.
const MEDIA_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm", "avi", "mov", "flv", "wmv", "m4v"];

/// Helper function to check if a file path is a video file.
fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

// ==============================================================================
// STEP 1: THE DATA SHAPE (`struct`)
// ==============================================================================
// RUST CONCEPT: Rust has NO `class` keyword!
// In JS, classes bundle data and functions together.
// In Rust, we define the DATA separately in a `struct` (like a database schema or interface),
// and then attach methods to it later inside an `impl` block below.
//
// Every field here is `pub` (public) so our tab modules can read them.
/// The master application struct holding all state for our tutorial app.
pub struct TutorialApp {
    // Navigation
    pub active_tab: Tab,

    // Tab 1: Counter & Interactive Widgets State
    pub counter_value: i64,
    pub text_input_value: String,
    pub slider_value: f32,
    pub checkbox_value: bool,

    // Tab 2: Async Background Scanner State
    pub selected_folder: Option<String>,
    pub is_scanning: bool,
    pub scanned_items: Vec<ScannedItem>,
    pub filter_query: String,

    // Tab 3: SQLite Persistence State
    pub db: TutorialDatabase,
    pub cached_notes: Vec<NoteRecord>,
    pub new_note_title: String,
    pub new_note_content: String,

    // Tab 4: Theming State
    pub current_theme: Theme,

    // Global Status & Uptime Ticker
    pub uptime_seconds: u64,
    pub status_banner: Option<String>,
}

// ==============================================================================
// STEP 2: THE BEHAVIOR (`impl`)
// ==============================================================================
// `impl` stands for "Implementation".
// This is where we attach functions and methods to the `TutorialApp` struct.
//
// Notice the difference:
// - `Self` (capital S): Refers to the TYPE (`TutorialApp`).
// - `self` (lowercase s): Refers to the INSTANCE (like `this` in JS).
impl TutorialApp {
    /// Constructs the initial application state.
    ///
    /// RUST CONCEPT: Static Constructor Functions
    /// Notice this function has NO `self` in its arguments! That makes it a static
    /// function (called as `TutorialApp::new()`), exactly like `static function create()` in PHP.
    ///
    /// RUST CONCEPT: Tuple Return Type `(Self, Task<Message>)`
    /// A tuple `(A, B)` bundles multiple values together without making a new struct.
    /// Here, we return:
    /// 1. `Self`: The newly created `TutorialApp` instance.
    /// 2. `Task<Message>`: An optional async task to run on boot (or `Task::none()`).
    pub fn new() -> (Self, Task<Message>) {
        // Initialize an in-memory SQLite database for the tutorial
        let db = TutorialDatabase::open_in_memory().expect("Failed to initialize SQLite database");

        // Seed the database with a couple of helpful tutorial notes
        let _ = db.add_note(
            "Welcome to Rust & Iced!",
            "You are running a native GUI compiled directly to machine code with zero Electron overhead.",
        );
        let _ = db.add_note(
            "The Elm Architecture",
            "State -> View -> Message -> Update. Unidirectional, predictable, and thread-safe.",
        );

        let initial_notes = db.get_all_notes().unwrap_or_default();

        // ======================================================================
        // RUST CONCEPT: The "Constructor Factory" Pattern (`let app = Self { ... }`)
        // ======================================================================
        // Since Rust has no classes, there is no magic `__construct()` or `constructor()`.
        // Instead, Rust uses the "Static Factory Method" pattern:
        //
        // 1. FACTORY ROLE: `TutorialApp::new()` acts as a factory. It performs setup
        //    (opens SQLite, seeds sample notes), constructs the instance, and returns it.
        // 2. `Self` (Capital S) is just a clean alias for `TutorialApp` (the struct we are inside).
        //    Writing `Self { ... }` is 100% IDENTICAL to writing `TutorialApp { ... }`.
        // 3. NO `new` KEYWORD: Rust has no `new` operator. You instantiate structs using
        //    the "Struct Literal" syntax: `StructName { field1: val1, field2: val2 }`.
        // 4. JS EQUIVALENT:
        //    `static create() { const app = new TutorialApp(); ... return [app, null]; }`
        //
        // We build the instance here, store it in the local variable `app`, and return it below!
        let app = Self {
            active_tab: Tab::CounterAndWidgets,
            counter_value: 0,
            text_input_value: String::new(),
            slider_value: 50.0,
            checkbox_value: true,
            selected_folder: None,
            is_scanning: false,
            scanned_items: Vec::new(),
            filter_query: String::new(),
            db,
            cached_notes: initial_notes,
            new_note_title: String::new(),
            new_note_content: String::new(),
            current_theme: Theme::Dark,
            uptime_seconds: 0,
            status_banner: Some("Ready! Select any tab above to explore.".to_string()),
        };

        // RUST CONCEPT: Returning a Tuple `(app, task)`
        // Notice there is no `return` keyword and no semicolon `;` on the last line!
        // In Rust, the final expression in a function without a semicolon IS the return value.
        // We return `app` (our new state) and `Task::none()` (no initial async task).
        (app, Task::none())
    }

    /// Returns the window title bar string.
    pub fn title(&self) -> String {
        format!(
            "Wazoo Tutorial — {} (Uptime: {}s)",
            self.active_tab.title(),
            self.uptime_seconds
        )
    }

    /// Returns the currently active Iced theme.
    pub fn theme(&self) -> Theme {
        self.current_theme.clone()
    }

    /// Subscribes to runtime event streams.
    ///
    /// RUST CONCEPT: Subscriptions & Periodic Timers
    /// Subscriptions let you listen to external events like keyboard keypresses,
    /// mouse motion, window resizing, or periodic time intervals.
    ///
    /// Here, we emit a `Message::Tick` every 1 second to increment `uptime_seconds`.
    ///
    /// RUST CONCEPT: Closures `|_|` and `.map()`
    /// - `time::every(...)` emits a timestamp (`Instant`) every 1 second.
    /// - `|_|` is an anonymous closure (like `(_) => ...` in JS).
    ///   The pipes `| |` enclose the parameters.
    /// - `_` means: "Ignore this timestamp argument; we don't need it."
    /// - `.map(...)` transforms the stream from `Subscription<Instant>` into `Subscription<Message>`!
    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }

    /// The state transition engine: handles incoming messages and mutates state.
    ///
    /// RUST CONCEPT: `&mut self` and Exhaustive `match`
    /// Notice `&mut self`. Rust's borrow checker guarantees that while `update` is running,
    /// no other thread or function can read or write to `TutorialApp`.
    ///
    /// Furthermore, the `match message` statement is EXHAUSTIVE: if we add a new variant
    /// to `enum Message`, the compiler will refuse to build until we handle it here!
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // ------------------------------------------------------------------
            // Global & Navigation
            // ------------------------------------------------------------------
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.status_banner = Some(format!("Switched to: {}", tab.title()));
                Task::none()
            }

            // ==================================================================
            // RUST CONCEPT: Subscriptions, Unit Variants & `Task::none()`
            // ==================================================================
            // 1. WHERE DID THIS COME FROM?
            //    In JS, you would write an imperative `setInterval(() => { ... }, 1000)`.
            //    In Iced/Elm, you DECLARE a subscription in `subscription()` above:
            //    `time::every(1s).map(|_| Message::Tick)`.
            //    Every 1 second, the runtime pushes `Message::Tick` into this loop.
            //
            // 2. UNIT VARIANT (No Parentheses!):
            //    Notice `TabSelected(tab)` had data attached to it.
            //    `Message::Tick` carries NO payload data. In Rust, this is called a
            //    "Unit Variant" (like a simple flag or signal).
            //
            // 3. WHAT IS `Task::none()`?
            //    The `update()` function MUST return a `Task<Message>`.
            //    A `Task` is an instruction telling Iced to do background async work
            //    (like making an API request, opening a file picker, or running a query).
            //    Returning `Task::none()` means: "State is updated. No background work needed!"
            Message::Tick => {
                self.uptime_seconds += 1;
                Task::none()
            }

            // ------------------------------------------------------------------
            // Tab 1: Counter & Interactive Widgets
            // ------------------------------------------------------------------
            Message::IncrementCounter => {
                self.counter_value += 1;
                self.status_banner = Some(format!("Counter incremented to {}", self.counter_value));
                Task::none()
            }

            Message::DecrementCounter => {
                self.counter_value -= 1;
                self.status_banner = Some(format!("Counter decremented to {}", self.counter_value));
                Task::none()
            }

            Message::ResetCounter => {
                self.counter_value = 0;
                self.status_banner = Some("Counter reset to 0".to_string());
                Task::none()
            }

            Message::TextInputChanged(text) => {
                self.text_input_value = text;
                Task::none()
            }

            Message::SliderChanged(val) => {
                self.slider_value = val;
                Task::none()
            }

            Message::CheckboxToggled(checked) => {
                self.checkbox_value = checked;
                self.status_banner = Some(format!(
                    "Feature flag is now: {}",
                    if checked { "ENABLED" } else { "DISABLED" }
                ));
                Task::none()
            }

            // ------------------------------------------------------------------
            // Tab 2: Async Background Scanner & Native RFD File Picker
            // ------------------------------------------------------------------
            Message::PickFolderClicked => {
                // RUST CONCEPT: Spawning an Async File Dialog via `Task::perform`
                // `rfd::AsyncFileDialog` opens the native operating system folder chooser.
                // We wrap it in `Task::perform` so the window doesn't freeze while the user
                // is picking a folder!
                Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_title("Select a Directory to Scan (Wazoo Scanner Demo)")
                            .pick_folder()
                            .await;

                        handle.map(|h| h.path().to_string_lossy().to_string())
                    },
                    Message::FolderChosen,
                )
            }

            Message::FolderChosen(opt_path) => {
                if let Some(path) = opt_path {
                    self.selected_folder = Some(path.clone());
                    self.status_banner = Some(format!("Selected folder: {}", path));
                }
                Task::none()
            }

            Message::StartScanClicked => {
                let folder = match &self.selected_folder {
                    Some(f) => f.clone(),
                    None => return Task::none(),
                };

                self.is_scanning = true;
                self.scanned_items.clear();
                self.status_banner = Some(format!("Scanning '{}' on background thread...", folder));

                // RUST CONCEPT: Offloading Heavy Work to a Background Thread
                // We use `tokio::task::spawn_blocking` inside `Task::perform`.
                // `walkdir` is synchronous filesystem I/O, so `spawn_blocking` ensures
                // it runs on Tokio's blocking threadpool, leaving the async UI runtime 100% responsive.
                Task::perform(
                    async move {
                        let scan_result = tokio::task::spawn_blocking(move || {
                            let mut results = Vec::new();
                            for entry in WalkDir::new(&folder)
                                .max_depth(5)
                                .into_iter()
                                .filter_map(|e| e.ok())
                            {
                                if entry.file_type().is_file() {
                                    let path = entry.path().to_path_buf();
                                    let filename = entry.file_name().to_string_lossy().to_string();
                                    let metadata = entry.metadata().ok();
                                    let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);
                                    let is_video = is_media_file(&path);

                                    results.push(ScannedItem {
                                        path,
                                        filename,
                                        size_bytes,
                                        is_video,
                                    });
                                }
                            }
                            results
                        })
                        .await;

                        match scan_result {
                            Ok(items) => Ok(items),
                            Err(e) => Err(format!("Scan worker panicked or failed: {}", e)),
                        }
                    },
                    Message::ScanFinished,
                )
            }

            Message::ScanFinished(result) => {
                self.is_scanning = false;
                match result {
                    Ok(items) => {
                        let count = items.len();
                        let media_count = items.iter().filter(|i| i.is_video).count();
                        self.scanned_items = items;
                        self.status_banner = Some(format!(
                            "Scan complete! Discovered {} files ({} media videos).",
                            count, media_count
                        ));
                    }
                    Err(err) => {
                        self.status_banner = Some(format!("Scan error: {}", err));
                    }
                }
                Task::none()
            }

            Message::FilterQueryChanged(query) => {
                self.filter_query = query;
                Task::none()
            }

            // ------------------------------------------------------------------
            // Tab 3: Embedded SQLite Persistence
            // ------------------------------------------------------------------
            Message::NoteTitleChanged(title) => {
                self.new_note_title = title;
                Task::none()
            }

            Message::NoteContentChanged(content) => {
                self.new_note_content = content;
                Task::none()
            }

            Message::SaveNoteClicked => {
                let title = self.new_note_title.trim();
                let content = self.new_note_content.trim();

                if title.is_empty() {
                    self.status_banner = Some("Cannot save note with empty title!".to_string());
                    return Task::none();
                }

                match self.db.add_note(title, content) {
                    Ok(id) => {
                        self.new_note_title.clear();
                        self.new_note_content.clear();
                        self.cached_notes = self.db.get_all_notes().unwrap_or_default();
                        self.status_banner =
                            Some(format!("Saved note #{} to SQLite database!", id));
                    }
                    Err(err) => {
                        self.status_banner = Some(format!("SQLite error: {}", err));
                    }
                }
                Task::none()
            }

            Message::DeleteNoteClicked(id) => {
                match self.db.delete_note(id) {
                    Ok(_) => {
                        self.cached_notes = self.db.get_all_notes().unwrap_or_default();
                        self.status_banner =
                            Some(format!("Deleted note #{} from SQLite database.", id));
                    }
                    Err(err) => {
                        self.status_banner =
                            Some(format!("Failed to delete note #{}: {}", id, err));
                    }
                }
                Task::none()
            }

            Message::RefreshNotesClicked => {
                self.cached_notes = self.db.get_all_notes().unwrap_or_default();
                self.status_banner = Some(format!(
                    "Refreshed notes from SQLite ({} records).",
                    self.cached_notes.len()
                ));
                Task::none()
            }

            // ------------------------------------------------------------------
            // Tab 4: Theming & Custom Styling
            // ------------------------------------------------------------------
            Message::ThemeSelected(theme) => {
                self.status_banner = Some(format!("Applied theme: {:?}", theme));
                self.current_theme = theme;
                Task::none()
            }
        }
    }

    /// Renders the entire application UI layout.
    ///
    /// RUST CONCEPT: Declarative Layout Composition
    /// In Iced, layouts are composed like Russian nesting dolls:
    /// `column![ header, tab_bar, active_tab_view, footer ]`
    pub fn view(&self) -> Element<'_, Message> {
        // ======================================================================
        // Top Application Header
        // ======================================================================
        let logo_badge = container(
            row![
                text("⚡").size(20),
                text("WAZOO TUTORIAL").size(16).color(style::COLOR_ACCENT),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        let subtitle = text("A hands-on interactive playground for mastering Rust & Iced")
            .size(13)
            .color(style::COLOR_TEXT_MUTED);

        let app_header = container(
            row![
                column![logo_badge, subtitle].spacing(2),
                container(
                    text(format!("Uptime: {}s", self.uptime_seconds))
                        .size(12)
                        .color(style::COLOR_TEXT_DIM)
                )
                .style(style::badge_style)
                .padding([4, 10]),
            ]
            .spacing(20)
            .align_y(Alignment::Center),
        )
        .padding([12, 20])
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Tab Navigation Bar
        // ======================================================================
        let tabs = [
            Tab::CounterAndWidgets,
            Tab::AsyncMediaScanner,
            Tab::SqliteDatabase,
            Tab::ThemingPlayground,
            Tab::ReferenceGuide,
        ];

        let mut tab_buttons = row![].spacing(8);
        for tab in tabs {
            let is_active = self.active_tab == tab;
            let tab_btn = button(
                row![text(tab.icon()).size(15), text(tab.title()).size(13),]
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .style(style::tab_button_style(is_active))
            .on_press(Message::TabSelected(tab))
            .padding([8, 14]);

            tab_buttons = tab_buttons.push(tab_btn);
        }

        let tab_bar = container(tab_buttons).padding([4, 20]).width(Length::Fill);

        // ======================================================================
        // Active Tab View Content
        // ======================================================================
        let tab_content: Element<'_, Message> = match self.active_tab {
            Tab::CounterAndWidgets => self.view_counter_tab(),
            Tab::AsyncMediaScanner => self.view_async_scanner_tab(),
            Tab::SqliteDatabase => self.view_database_tab(),
            Tab::ThemingPlayground => self.view_theming_tab(),
            Tab::ReferenceGuide => self.view_guide_tab(),
        };

        // ======================================================================
        // Bottom Status Bar
        // ======================================================================
        let status_text = self.status_banner.as_deref().unwrap_or("Ready.");

        let status_bar = container(
            row![
                text("ℹ️").size(13),
                text(status_text)
                    .size(12)
                    .color(style::COLOR_ACCENT)
                    .width(Length::Fill),
                text("Rust 2024 • Iced 0.14 • Tokio • SQLite")
                    .size(11)
                    .color(style::COLOR_TEXT_DIM),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([8, 20])
        .style(style::card_style)
        .width(Length::Fill);

        // Wrap everything into the root application column
        column![
            app_header,
            rule::horizontal(1),
            tab_bar,
            container(tab_content)
                .width(Length::Fill)
                .height(Length::Fill),
            rule::horizontal(1),
            status_bar,
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
