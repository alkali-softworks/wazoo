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

mod app;
mod db;
mod message;
mod style;
mod tabs;

use app::TutorialApp;

/// The entry point of the executable.
/// 
/// RUST CONCEPT: `iced::Result` in `main()`
/// In C/C++, `main` returns an `int` (0 for success).
/// In Rust, `main` can return `()` or a `Result<(), E>`.
/// `iced::Result` is an alias for `Result<(), iced::Error>`.
/// If the application exits normally, it returns `Ok(())`.
/// If window creation fails (e.g. no display server / Wayland / X11 error),
/// the error is cleanly printed to stderr!
pub fn main() -> iced::Result {
    // Initialize logging from environment variables (e.g. RUST_LOG=info)
    env_logger::init();

    log::info!("Starting Wazoo Rust & Iced Tutorial Application...");

    // Configure the main application window
    let window_settings = iced::window::Settings {
        size: iced::Size::new(1020.0, 760.0),
        position: iced::window::Position::Centered,
        min_size: Some(iced::Size::new(720.0, 500.0)),
        resizable: true,
        decorations: true, // Native window title bar & border for ease of use
        ..Default::default()
    };

    // Boot the Iced application runtime
    iced::application(
        TutorialApp::new,
        TutorialApp::update,
        TutorialApp::view,
    )
    .title(TutorialApp::title)
    .subscription(TutorialApp::subscription)
    .theme(TutorialApp::theme)
    .window(window_settings)
    .run()
}
