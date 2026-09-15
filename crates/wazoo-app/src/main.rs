/*!
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
use crate::app::WazooApp;
use crate::cli::parse_cli_args;
use iced::{Color, Theme};

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
            // SAFETY: Setting environment variables at startup before background worker threads run.
            unsafe {
                std::env::set_var("WGPU_POWER_PREF", "low");
            }
        }
    }

    let config_mgr = wazoo_core::ConfigManager::new();
    let initial_settings = config_mgr.load_settings();
    let win_w = (initial_settings.window_bounds.width as f32).clamp(200.0, 7680.0);
    let win_h = (initial_settings.window_bounds.height as f32).clamp(150.0, 4320.0);
    let win_pos = iced::window::Position::Specific(iced::Point::new(
        initial_settings.window_bounds.x as f32,
        initial_settings.window_bounds.y as f32,
    ));

    let initial_query = cli.query;
    let app_icon =
        iced::window::icon::from_file_data(include_bytes!("../resources/icon.png"), None).ok();

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
        size: iced::Size::new(win_w, win_h),
        position: win_pos,
        icon: app_icon,
        decorations: false,
        transparent: true,
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "wazoo".to_string(),
            override_redirect: false,
        },
        ..Default::default()
    })
    .run()
}
