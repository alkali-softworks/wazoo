/*!
 * ==============================================================================
 * WAZOO-TUTORIAL: A Step-by-Step Learning Sandbox for Rust & Iced
 * ==============================================================================
 *
 * Welcome to `wazoo-tutorial`!
 *
 * This application was crafted to help you learn Rust, Iced (0.14), and the
 * technologies that power `wazoo-rs` (such as `rusqlite`, `tokio`, `rfd`, and `walkdir`).
 *
 * HOW AN ICED APPLICATION RUNS (THE BIG PICTURE):
 * -----------------------------------------------
 * When you call `iced::application(...)` and chain `.run()`, Iced:
 * 1. Initializes the native OS window using `winit` (the standard Rust windowing library).
 * 2. Initializes the graphics backend using `wgpu` (WebGPU for native Vulkan/Metal/DirectX).
 * 3. Spins up an asynchronous event loop powered by `tokio`.
 * 4. Calls your `TutorialApp::new()` to construct the initial state.
 * 5. Calls `TutorialApp::view(&state)` to generate the widget tree.
 * 6. Waits for user interactions (clicks, keyboard inputs, timers) and passes them to
 *    `TutorialApp::update(&mut state, message)`.
 * 7. If `update` changes the state, Iced re-calls `view(&state)` and renders the new frame.
 *
 * HOW TO RUN THIS APPLICATION:
 * ----------------------------
 * In your terminal, run:
 *   cargo run -p wazoo-tutorial
 *
 * Or with debug logging enabled:
 *   RUST_LOG=info cargo run -p wazoo-tutorial
 */

// ==============================================================================
// STEP 1: MODULE DECLARATIONS (`mod`)
// ==============================================================================
// In Rust, files on your hard drive are NOT automatically compiled or autoloaded!
// Writing `mod app;` tells the compiler:
// "Go find `src/app.rs`, compile it, and attach it as a child module named `app`."
use wazoo_tutorial::app::TutorialApp;

/// The entry point of the executable.
///
/// RUST CONCEPT 1: `pub fn main() -> iced::Result`
/// - `pub`: Public visibility (standard in Rust binaries).
/// - `fn`: Function keyword.
/// - `-> iced::Result`: The return type!
///   In C, `main` returns an `int` (0 for success).
///   In Rust, `main` can return `Result<(), iced::Error>`.
///   - If everything succeeds, it returns `Ok(())` (where `()` is the "unit type", like void).
///   - If window creation fails (e.g. no display server / Wayland / X11 error),
///     it returns `Err(...)` and Rust cleanly prints the error to stderr!
pub fn main() -> iced::Result {
    // Initialize logging from environment variables (e.g. RUST_LOG=info cargo run).
    // If you don't set RUST_LOG, logging stays quiet.
    env_logger::init();

    log::info!("Starting Wazoo Rust & Iced Tutorial Application...");

    // ==========================================================================
    // STEP 3: WINDOW CONFIGURATION (Structs & Default Syntax)
    // ==========================================================================
    // RUST CONCEPT 2: Struct Update Syntax `..Default::default()`
    // `iced::window::Settings` has dozens of fields (icons, transparency, min/max size).
    // Instead of specifying all 20+ fields manually, Rust lets us specify the ones
    // we care about, then use `..Default::default()` to fill in the rest with defaults!
    let app_icon = iced::window::icon::from_file_data(
        include_bytes!("../../wazoo-app/resources/icon.png"),
        None,
    )
    .ok();

    let window_settings = iced::window::Settings {
        size: iced::Size::new(1020.0, 760.0), // Default width & height in points
        position: iced::window::Position::Centered, // Center the window on the active monitor
        min_size: Some(iced::Size::new(720.0, 500.0)), // Prevent users from making it too tiny
        resizable: true,                      // Allow window edge dragging
        decorations: true, // Enable standard OS titlebar, minimize, and [X] close buttons
        icon: app_icon,
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "wazoo-tutorial".to_string(),
            override_redirect: false,
        },
        ..Default::default() // Fill remaining fields (icons, platform settings) with defaults
    };

    // ==========================================================================
    // STEP 4: THE ICED APPLICATION BUILDER (Method Chaining & Function Pointers)
    // ==========================================================================
    // RUST CONCEPT 3: Passing Function Pointers (Notice NO parentheses!)
    // Notice we pass `TutorialApp::new`, NOT `TutorialApp::new()`.
    // - With parentheses `TutorialApp::new()` = "Call this function RIGHT NOW".
    // - WITHOUT parentheses `TutorialApp::new` = "Here is the address/pointer to this function.
    //   Iced, YOU call it when you are ready!"
    //
    // The three core functions passed here define The Elm Architecture (TEA):
    // 1. `TutorialApp::new`:    Constructs the initial state struct.
    // 2. `TutorialApp::update`: The state-mutation function: handles every `Message`.
    // 3. `TutorialApp::view`:   The pure UI layout function: converts state -> widgets.
    iced::application(TutorialApp::new, TutorialApp::update, TutorialApp::view)
        // RUST CONCEPT 4: The Builder Pattern
        // Each method takes `self` by value, configures a hook, and returns the modified builder.
        //
        // `.title(...)`: Callback returning the dynamic window title string.
        .title(TutorialApp::title)
        // `.subscription(...)`: Hook for listening to external event streams (e.g. 1-sec timer ticks).
        .subscription(TutorialApp::subscription)
        // `.theme(...)`: Callback returning the active color theme (Dark, Light, TokyoNight, etc.).
        .theme(TutorialApp::theme)
        // `.window(...)`: Attaches the window geometry and decoration settings configured above.
        .window(window_settings)
        // ==========================================================================
        // STEP 5: `.run()` — WHERE DOES EXECUTION GO?
        // ==========================================================================
        // Calling `.run()` BLOCKS the main thread and starts the desktop Event Loop.
        //
        // Here is what happens under the hood right now:
        // 1. Iced calls `TutorialApp::new()` to initialize memory and open SQLite.
        // 2. Iced creates the OS window via `winit` and attaches `wgpu` for GPU rendering.
        // 3. Iced calls `TutorialApp::view()` to render Frame 1 to the screen.
        // 4. Execution enters an infinite loop:
        //    - Sleeps until an OS event occurs (click, typing, window resize, timer tick).
        //    - Maps the event into a `Message`.
        //    - Calls `TutorialApp::update(&mut state, message)`.
        //    - If state changed, calls `TutorialApp::view(&state)` to repaint at 60+ FPS.
        // 5. When the user closes the window, `.run()` breaks the loop, frees all resources,
        //    and returns `Ok(())`!
        .run()
}
