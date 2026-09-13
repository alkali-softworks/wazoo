/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Application Entry Point
 * 
 * Initializes logging, platform-specific cursor environments, and parses CLI arguments.
 * Boots the Iced application runtime with global styles, window configuration, and lifecycle hooks.
 */

mod app;
mod assets;
mod cli;
mod cursor;
mod format;
mod keybinds;
mod message;
mod platform;
mod scroll_view;
mod theme;
mod update;
mod views;

use iced::{Color, Theme};
use crate::app::WazooApp;
use crate::cli::parse_cli_args;

pub fn main() -> iced::Result {
    let cli = parse_cli_args();
    env_logger::init();
    #[cfg(target_os = "linux")]
    crate::platform::init_linux_cursor_env();

    #[cfg(target_os = "linux")]
    {
        // On Linux hybrid graphics laptops (e.g. Intel iGPU + NVIDIA dGPU), defaulting to
        // the integrated GPU avoids cross-GPU DRI3 PRIME swapchain presentation failure /
        // VK_ERROR_DEVICE_LOST when windows are occluded or behind other windows.
        // On desktop PCs, monitors are plugged directly into the dGPU,
        // so the dedicated GPU is preferred without PRIME offload sync issues.
        if std::env::var("WGPU_POWER_PREF").is_err() && crate::platform::is_hybrid_laptop() {
            std::env::set_var("WGPU_POWER_PREF", "low");
        }
    }

    let initial_query = cli.query;
    iced::application(
        move || WazooApp::new(initial_query.clone()),
        WazooApp::update,
        WazooApp::view,
    )
    .title(WazooApp::title)
    .subscription(WazooApp::subscription)
    .theme(WazooApp::theme)
    .style(|app: &WazooApp, _theme: &Theme| iced::theme::Style {
        background_color: if app.available_videos.is_empty() {
            Color::from_rgba(0.05, 0.05, 0.05, app.current_opacity())
        } else {
            Color::TRANSPARENT
        },
        text_color: Color::WHITE,
    })
    .window(iced::window::Settings {
        size: iced::Size::new(1280.0, 720.0),
        decorations: false,
        transparent: true,
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "wazoo".to_string(),
            override_redirect: false,
        },
        ..Default::default()
    })
    .run()
}
