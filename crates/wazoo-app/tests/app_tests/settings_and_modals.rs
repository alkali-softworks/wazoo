use super::common::new_backend_app;
use wazoo_app::app::{
    DROPDOWN_MENU_SLIDE_TICKS, PLAYBACK_SETTINGS_DEBOUNCE_TICKS, PLAYER_OVERLAY_FADE_TICKS,
    SettingsTab, TITLEBAR_FADE_TICKS, TITLEBAR_SHOW_DELAY_TICKS, TITLEBAR_SLIDE_TICKS,
    new_test_app,
};
use wazoo_app::message::Message;
use wazoo_app::state::ThemeColorTarget;
use wazoo_app::theme;

#[test]
fn test_settings_toggle_default_player() {
    let (mut app, _) = new_test_app();
    assert!(!app.settings.is_default_player);

    let _ = app.update(Message::OpenSettingsModal);
    assert!(app.modals.settings);

    // Render settings modal with toggle button
    {
        let _view = app.view_settings_modal();
    }

    // Toggle default player ON
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(app.settings.is_default_player);

    // Render settings modal with active toggle button
    {
        let _view_active = app.view_settings_modal();
    }

    // Toggle default player OFF
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(!app.settings.is_default_player);
}

#[test]
fn test_settings_modal_tabs_and_playback_options() {
    let (mut app, _) = new_test_app();
    assert_eq!(app.modals.settings_tab, SettingsTab::General);

    // Open settings modal
    let _ = app.update(Message::OpenSettingsModal);
    assert!(app.modals.settings);
    assert_eq!(app.modals.settings_tab, SettingsTab::General);

    // Render General tab
    {
        let _view_general = app.view_settings_modal();
    }

    // Switch to Playback tab
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Playback));
    assert_eq!(app.modals.settings_tab, SettingsTab::Playback);

    // Render Playback tab
    {
        let _view_playback = app.view_settings_modal();
    }

    // Adjust playback options
    let _ = app.update(Message::SetGamma(25.0));
    assert_eq!(app.settings.gamma, 25.0);

    let _ = app.update(Message::SetContrast(-15.0));
    assert_eq!(app.settings.contrast, -15.0);

    let _ = app.update(Message::SetBrightness(10.0));
    assert_eq!(app.settings.brightness, 10.0);

    let _ = app.update(Message::SetSaturation(40.0));
    assert_eq!(app.settings.saturation, 40.0);

    let _ = app.update(Message::SetPlaybackSpeed(1.5));
    assert_eq!(app.settings.playback_speed, 1.5);

    let _ = app.update(Message::SetBufferDuration(15));
    assert_eq!(app.settings.buffer_duration_secs, 15);

    let _ = app.update(Message::SetBufferSize(64));
    assert_eq!(app.settings.buffer_size_mb, 64);

    // Re-render Playback tab with adjusted values
    {
        let _view_playback_adjusted = app.view_settings_modal();
    }

    // Test ResetPlaybackOptions
    let _ = app.update(Message::ResetPlaybackOptions);
    assert_eq!(app.settings.gamma, 0.0);
    assert_eq!(app.settings.contrast, 0.0);
    assert_eq!(app.settings.brightness, 0.0);
    assert_eq!(app.settings.saturation, 0.0);
    // Test individual slider reset (the reset button next to each value badge)
    let _ = app.update(Message::SetGamma(40.0));
    let _ = app.update(Message::SetContrast(100.0));
    assert_eq!(app.settings.gamma, 40.0);
    assert_eq!(app.settings.contrast, 100.0);
    {
        let _view_with_reset_buttons = app.view_settings_modal();
    }
    // Reset only gamma via individual reset button
    let _ = app.update(Message::SetGamma(0.0));
    assert_eq!(app.settings.gamma, 0.0);
    assert_eq!(app.settings.contrast, 100.0);
    // Reset only contrast via individual reset button
    let _ = app.update(Message::SetContrast(0.0));
    assert_eq!(app.settings.contrast, 0.0);

    // Switch to Filters tab (Shader Filters)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Filters));
    assert_eq!(app.modals.settings_tab, SettingsTab::Filters);
    {
        let _view_filters = app.view_settings_modal();
    }
    // Master filter toggle enables all checked filters (CRT is checked by default)
    let _ = app.update(Message::ToggleFilters);
    assert!(app.settings.filters_enabled);
    assert!(app.settings.crt_enabled);
    {
        let _view_filters_active = app.view_settings_modal();
    }
    // Individual filter toggles work
    let _ = app.update(Message::ToggleWavyFilter);
    assert!(app.settings.filter_wavy);
    let _ = app.update(Message::ToggleFogFilter);
    assert!(app.settings.filter_fog);

    let _ = app.update(Message::ToggleFilters);
    assert!(!app.settings.filters_enabled);
    assert!(!app.settings.crt_enabled);

    // Switch to CRT tab (Configurable CRT Shader Parameters)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Crt));
    assert_eq!(app.modals.settings_tab, SettingsTab::Crt);
    {
        let _view_crt = app.view_settings_modal();
    }

    // Configure CRT parameters
    let _ = app.update(Message::SetCrtScanlineDarkness(0.40));
    assert_eq!(app.settings.crt.scanline_darkness, 0.40);
    let _ = app.update(Message::SetCrtScanlineDensity(750.0));
    assert_eq!(app.settings.crt.scanline_density, 750.0);
    let _ = app.update(Message::SetCrtScanlineSpeed(2.5));
    assert_eq!(app.settings.crt.scanline_speed, 2.5);

    // CRT Gamma & Saturation compensation
    let _ = app.update(Message::SetCrtGamma(1.35));
    assert_eq!(app.settings.crt.gamma, 1.35);
    let _ = app.update(Message::SetCrtSaturation(1.40));
    assert_eq!(app.settings.crt.saturation, 1.40);
    let _ = app.update(Message::SetCrtBrightness(1.10));
    assert_eq!(app.settings.crt.brightness, 1.10);

    // Geometry & artifacts
    let _ = app.update(Message::SetCrtCurvature(0.25));
    assert_eq!(app.settings.crt.curvature, 0.25);
    let _ = app.update(Message::SetCrtCornerRounding(0.60));
    assert_eq!(app.settings.crt.corner_rounding, 0.60);
    let _ = app.update(Message::SetCrtVignette(0.30));
    assert_eq!(app.settings.crt.vignette, 0.30);
    let _ = app.update(Message::SetCrtChromaticAberration(0.35));
    assert_eq!(app.settings.crt.chromatic_aberration, 0.35);
    let _ = app.update(Message::SetCrtNoise(0.15));
    assert_eq!(app.settings.crt.noise, 0.15);
    let _ = app.update(Message::SetCrtGlitchIntensity(0.70));
    assert_eq!(app.settings.crt.glitch_intensity, 0.70);
    let _ = app.update(Message::SetCrtGlitchFrequency(0.45));
    assert_eq!(app.settings.crt.glitch_frequency, 0.45);

    {
        let _view_crt_configured = app.view_settings_modal();
    }

    // Apply preset
    let _ = app.update(Message::ApplyCrtPreset(wazoo_core::CrtPreset::WarmPhosphor));
    assert_eq!(app.settings.crt.gamma, 1.25);
    assert_eq!(app.settings.crt.saturation, 1.20);
    assert_eq!(app.settings.crt.scanline_darkness, 0.15);
    assert_eq!(app.settings.crt.glitch_intensity, 0.15);
    assert_eq!(app.settings.crt.glitch_frequency, 0.10);
    {
        let _view_crt_preset = app.view_settings_modal();
    }

    // Reset CRT settings
    let _ = app.update(Message::ResetCrtSettings);
    assert_eq!(app.settings.crt.scanline_darkness, 0.20);
    assert_eq!(app.settings.crt.gamma, 1.00);
    assert_eq!(app.settings.crt.saturation, 1.00);
    assert_eq!(app.settings.crt.glitch_intensity, 0.25);
    assert_eq!(app.settings.crt.glitch_frequency, 0.16);

    // Switch to System tab (Window Opacity & Default Player)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::System));
    assert_eq!(app.modals.settings_tab, SettingsTab::System);
    {
        let _view_system = app.view_settings_modal();
    }

    // Adjust window opacity on System tab
    let _ = app.update(Message::SetWindowOpacity(0.75));
    assert_eq!(app.settings.window_opacity, 0.75);

    // Toggle default player on System tab
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(app.settings.is_default_player);
    {
        let _view_system_active = app.view_settings_modal();
    }
    let _ = app.update(Message::ToggleDefaultPlayer);
    assert!(!app.settings.is_default_player);

    // Toggle standby player on System tab
    assert!(!app.settings.use_standby_player);
    let _ = app.update(Message::ToggleStandbyPlayer);
    assert!(app.settings.use_standby_player);
    {
        let _view_system_standby_on = app.view_settings_modal();
    }
    let _ = app.update(Message::ToggleStandbyPlayer);
    assert!(!app.settings.use_standby_player);

    // Adjust flip interval on System tab
    assert_eq!(app.settings.flip_interval_secs, 45);
    let _ = app.update(Message::SetFlipInterval(90));
    assert_eq!(app.settings.flip_interval_secs, 90);
    {
        let _view_system_flip_active = app.view_settings_modal();
    }
    // Reset flip interval via reset button
    let _ = app.update(Message::SetFlipInterval(
        wazoo_core::models::DEFAULT_FLIP_INTERVAL_SECS,
    ));
    assert_eq!(app.settings.flip_interval_secs, 45);

    // Test flip interval clamping
    let _ = app.update(Message::SetFlipInterval(0));
    assert_eq!(app.settings.flip_interval_secs, 1);
    let _ = app.update(Message::SetFlipInterval(99999));
    assert_eq!(app.settings.flip_interval_secs, 3600);
    let _ = app.update(Message::SetFlipInterval(45));

    // Switch to Cube tab (3D Cube Screensaver settings)
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Cube));
    assert_eq!(app.modals.settings_tab, SettingsTab::Cube);
    {
        let _view_cube = app.view_settings_modal();
    }

    // Toggle cube screensaver, adjust speed & size, spawn additional cube
    let _ = app.update(Message::ToggleCubeScreensaver);
    assert!(app.cube.enabled);
    assert_eq!(app.cube.cubes.len(), 1);
    {
        let _view_cube_active = app.view_settings_modal();
    }
    let _ = app.update(Message::SetCubeSpeed(1.5));
    assert_eq!(app.cube.speed_multiplier, 1.5);
    let _ = app.update(Message::SetCubeSize(1.8));
    assert_eq!(app.cube.size_multiplier, 1.8);
    let _ = app.update(Message::SpawnCube);
    assert_eq!(app.cube.cubes.len(), 2);
    let _ = app.update(Message::ClearCubes);
    assert_eq!(app.cube.cubes.len(), 0);

    // Switch back to General tab
    let _ = app.update(Message::SetSettingsTab(SettingsTab::General));
    assert_eq!(app.modals.settings_tab, SettingsTab::General);
    {
        let _view_general_back = app.view_settings_modal();
    }

    // Close settings modal
    let _ = app.update(Message::CloseSettingsModal);
    assert!(!app.modals.settings);
}

#[test]
fn test_menu_modal_open_close_and_render() {
    let (mut app, _) = new_test_app();
    assert!(!app.modals.menu);

    let _ = app.update(Message::OpenMenuModal);
    assert!(app.modals.menu);
    {
        let _menu_view = app.view_menu_modal();
    }

    let _ = app.update(Message::CloseMenuModal);
    assert!(!app.modals.menu);
}

#[test]
fn test_search_and_bookmarks_modal_render() {
    let (mut app, _) = new_test_app();
    let _ = app.update(Message::OpenSearchModal);
    assert!(app.modals.search);
    {
        let _search_view = app.view_search_modal();
    }
    let _ = app.update(Message::CloseSearchModal);
    assert!(!app.modals.search);

    let _ = app.update(Message::ToggleBookmarksModal);
    assert!(app.modals.bookmarks);
    {
        let _bookmarks_view = app.view_bookmarks_modal();
    }
    let _ = app.update(Message::CloseBookmarksModal);
    assert!(!app.modals.bookmarks);
}

#[test]
fn test_top_menu_slide_down_animation() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: top menu is hidden
    assert!(!app.titlebar.show);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Move cursor into top menu trigger zone (y < 35.0)
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(300.0, 10.0)));

    // Tick through hover delay (8 ticks)
    for _ in 0..TITLEBAR_SHOW_DELAY_TICKS {
        let _ = app.update(Message::Tick);
    }

    // After hover delay passes, titlebar begins showing and starts sliding down from 0.0
    assert!(app.titlebar.show);
    assert_eq!(app.titlebar.slide_ticks, 0);
    assert_eq!(app.titlebar_slide_progress(), 0.0);

    // Initial view rendering at start of slide down
    {
        let _view_start = app.view_titlebar();
    }

    // Animate ticks and verify slide down progress increases monotonically to 1.0
    let mut prev_progress = app.titlebar_slide_progress();
    for _ in 1..=TITLEBAR_SLIDE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_progress = app.titlebar_slide_progress();
        assert!(
            cur_progress >= prev_progress,
            "Slide progress must increase monotonically: prev={}, cur={}",
            prev_progress,
            cur_progress
        );
        prev_progress = cur_progress;
    }

    // Titlebar has fully slid down into view
    assert_eq!(app.titlebar_slide_progress(), 1.0);
    {
        let _view_full = app.view_titlebar();
    }

    // Now move cursor away to trigger dismissal
    let _ = app.update(Message::CursorMoved(win_id, iced::Point::new(300.0, 200.0)));

    // While fading out and sliding back up, progress smoothly decreases to 0.0
    for _ in 0..TITLEBAR_FADE_TICKS {
        let _ = app.update(Message::Tick);
    }

    // Dismissal complete
    assert!(!app.titlebar.show);
    assert_eq!(app.titlebar_slide_progress(), 0.0);
}

#[test]
fn test_dropdown_menu_slide_down_animation() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: dropdown menu is closed
    assert!(!app.titlebar.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // Toggle dropdown menu open
    let _ = app.update(Message::ToggleDropdownMenu);
    assert!(app.titlebar.show_dropdown_menu);
    assert!(app.titlebar.show);
    assert_eq!(app.titlebar.dropdown_menu_slide_ticks, 0);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);

    // View rendered at initial opening state
    {
        let _view_open_start = app.view_titlebar();
    }

    // Animate frames through DROPDOWN_MENU_SLIDE_TICKS
    let mut prev_progress = app.dropdown_menu_slide_progress();
    for _ in 1..=DROPDOWN_MENU_SLIDE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_progress = app.dropdown_menu_slide_progress();
        assert!(
            cur_progress >= prev_progress,
            "Dropdown slide progress must increase monotonically: prev={}, cur={}",
            prev_progress,
            cur_progress
        );
        prev_progress = cur_progress;
    }

    // Dropdown has fully slid down
    assert_eq!(app.dropdown_menu_slide_progress(), 1.0);
    {
        let _view_open_full = app.view_titlebar();
    }

    // Close dropdown menu
    let _ = app.update(Message::CloseDropdownMenu);
    assert!(!app.titlebar.show_dropdown_menu);
    assert_eq!(app.dropdown_menu_slide_progress(), 0.0);
}

#[test]
fn test_player_osd_fade_in_and_fade_out_animation() {
    let (mut app, _) = new_test_app();
    let win_id = iced::window::Id::unique();
    app.window.id = Some(win_id);

    // Initial state: player OSD is completely hidden
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);

    // Hover over a player to trigger OSD entrance
    let _ = app.update(Message::PlayerHovered(1));
    assert_eq!(app.hovered_player_id, Some(1));

    // On initial trigger, fade-in starts at alpha 0.0 (smooth entrance instead of popping to 1.0)
    assert_eq!(app.overlay.fade_in_ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);

    // Over PLAYER_OVERLAY_FADE_TICKS (12 ticks), alpha monotonically fades in to 1.0
    let mut prev_alpha = app.player_overlay_alpha();
    for _ in 1..=PLAYER_OVERLAY_FADE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_alpha = app.player_overlay_alpha();
        assert!(
            cur_alpha >= prev_alpha,
            "Fade-in alpha must increase monotonically: prev={}, cur={}",
            prev_alpha,
            cur_alpha
        );
        prev_alpha = cur_alpha;
    }

    // Fully faded in
    assert_eq!(app.player_overlay_alpha(), 1.0);

    // Now unhover the player to trigger fade out
    let _ = app.update(Message::PlayerUnhovered(1));

    // Over PLAYER_OVERLAY_FADE_TICKS frames, alpha monotonically fades out from 1.0 to 0.0
    let mut prev_alpha = app.player_overlay_alpha();
    for _ in 1..=PLAYER_OVERLAY_FADE_TICKS {
        let _ = app.update(Message::Tick);
        let cur_alpha = app.player_overlay_alpha();
        assert!(
            cur_alpha <= prev_alpha,
            "Fade-out alpha must decrease monotonically: prev={}, cur={}",
            prev_alpha,
            cur_alpha
        );
        prev_alpha = cur_alpha;
    }

    // Overlay completely dismissed
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);
    assert_eq!(app.hovered_player_id, None);

    // Test interrupted entrance: unhovering while still fading in does not jump
    let _ = app.update(Message::PlayerHovered(1));
    for _ in 0..4 {
        let _ = app.update(Message::Tick);
    }
    let mid_alpha = app.player_overlay_alpha();
    assert!((mid_alpha - (4.0 / PLAYER_OVERLAY_FADE_TICKS as f32)).abs() < 1e-4);

    // Unhover while mid-entrance: fade-out smoothly reverses from current alpha (4/12)
    let _ = app.update(Message::PlayerUnhovered(1));
    assert!(app.overlay.ticks <= 4);

    for _ in 0..4 {
        let _ = app.update(Message::Tick);
    }
    assert_eq!(app.overlay.ticks, 0);
    assert_eq!(app.player_overlay_alpha(), 0.0);
}

#[test]
fn test_playback_settings_sliders_debounced() {
    let (mut app, _) = new_test_app();

    // Open settings modal
    let _ = app.update(Message::OpenSettingsModal);
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Playback));
    assert!(app.modals.settings);
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);

    // 1. Moving sliders updates app.settings immediately for responsive UI
    let _ = app.update(Message::SetGamma(35.0));
    assert_eq!(app.settings.gamma, 35.0);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        PLAYBACK_SETTINGS_DEBOUNCE_TICKS
    );

    let _ = app.update(Message::SetSaturation(-20.0));
    assert_eq!(app.settings.saturation, -20.0);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        PLAYBACK_SETTINGS_DEBOUNCE_TICKS
    );

    // 2. Debounce ticks count down with Tick
    let _ = app.update(Message::Tick);
    assert_eq!(
        app.modals.playback_settings_debounce_ticks,
        PLAYBACK_SETTINGS_DEBOUNCE_TICKS - 1
    );

    // 3. Advancing all remaining ticks flushes the debounce
    while app.modals.playback_settings_debounce_ticks > 0 {
        let _ = app.update(Message::Tick);
    }
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);

    // 4. Closing the settings modal immediately flushes any pending debounced settings
    let _ = app.update(Message::SetBrightness(15.0));
    assert!(app.modals.playback_settings_debounce_ticks > 0);
    let _ = app.update(Message::CloseSettingsModal);
    assert_eq!(app.modals.playback_settings_debounce_ticks, 0);
    assert_eq!(app.settings.brightness, 15.0);
}

#[test]
fn test_theme_settings_tab_color_pickers_presets_and_persistence() {
    let (mut app, temp_dir) = new_backend_app("wazoo_theme_app");

    // Default palette matches original hardcoded constants
    assert!((app.theme_palette.primary.r - theme::COLOR_PRIMARY.r).abs() < 0.01);
    assert!((app.theme_palette.primary.g - theme::COLOR_PRIMARY.g).abs() < 0.01);
    assert!((app.theme_palette.primary.b - theme::COLOR_PRIMARY.b).abs() < 0.01);
    assert!((app.theme_palette.titlebar_bg.r - theme::COLOR_TITLEBAR_BG.r).abs() < 0.01);
    assert!((app.theme_palette.modal_bg.r - theme::COLOR_MODAL_BG.r).abs() < 0.01);

    // Open Settings -> Theme tab (all color pickers start collapsed until clicked)
    let _ = app.update(Message::OpenSettingsModal);
    let _ = app.update(Message::SetSettingsTab(SettingsTab::Theme));
    assert_eq!(app.modals.settings_tab, SettingsTab::Theme);
    assert_eq!(app.modals.active_color_picker, None);
    {
        let _view_theme = app.view_settings_modal();
    }

    // Select each color picker target and render expanded picker
    for &target in ThemeColorTarget::ALL {
        let _ = app.update(Message::SelectThemeColorTarget(Some(target)));
        assert_eq!(app.modals.active_color_picker, Some(target));
        let _view_target = app.view_settings_modal();
    }

    // Partial/invalid hex draft does not corrupt saved hex, valid hex updates palette immediately
    let _ = app.update(Message::ThemeHexInputChanged(
        ThemeColorTarget::Accent,
        "#ff".to_string(),
    ));
    assert_eq!(
        app.settings.theme.accent_color,
        wazoo_core::DEFAULT_THEME_ACCENT_COLOR
    );
    let _ = app.update(Message::ThemeHexInputChanged(
        ThemeColorTarget::Accent,
        "#FF5500".to_string(),
    ));
    assert_eq!(app.settings.theme.accent_color, "#ff5500");
    assert!((app.theme_palette.primary.r - 1.0).abs() < 0.01);
    assert!((app.theme_palette.primary.b - 0.0).abs() < 0.01);

    // Override UI grey (Surface) and purple titlebar via HSV & Hex
    let _ = app.update(Message::SetThemeColorHsv(
        ThemeColorTarget::Surface,
        210.0,
        0.4,
        0.15,
    ));
    assert_ne!(
        app.settings.theme.surface_color,
        wazoo_core::DEFAULT_THEME_SURFACE_COLOR
    );
    let _ = app.update(Message::ThemeHexInputChanged(
        ThemeColorTarget::Titlebar,
        "#0d3b66".to_string(),
    ));
    assert_eq!(app.settings.theme.titlebar_color, "#0d3b66");
    assert!((app.theme_palette.titlebar_bg.b - (0x66 as f32 / 255.0)).abs() < 0.01);

    // Adjust extra appearance sliders
    let _ = app.update(Message::SetThemeHudOpacity(0.65));
    assert_eq!(app.settings.theme.hud_opacity, 0.65);
    let _ = app.update(Message::SetThemeCornerRadius(1.75));
    assert_eq!(app.settings.theme.corner_radius, 1.75);
    let _ = app.update(Message::SetThemeFocusRingWidth(5.5));
    assert_eq!(app.settings.theme.focus_ring_width, 5.5);
    let _ = app.update(Message::CommitThemeColor);

    // Verify titlebar & modal render with customized theme
    {
        let _tb = app.view_titlebar();
        let _modal = app.view_settings_modal();
    }

    // Verify persistence across reload
    let reloaded_mgr = wazoo_core::ConfigManager::with_dirs(
        temp_dir.path().to_path_buf(),
        temp_dir.path().to_path_buf(),
    );
    let reloaded = reloaded_mgr.load_settings();
    assert_eq!(reloaded.theme.accent_color, "#ff5500");
    assert_eq!(reloaded.theme.titlebar_color, "#0d3b66");
    assert_eq!(reloaded.theme.hud_opacity, 0.65);
    assert_eq!(reloaded.theme.corner_radius, 1.75);
    assert_eq!(reloaded.theme.focus_ring_width, 5.5);

    // Reset single color
    let _ = app.update(Message::ResetThemeColor(ThemeColorTarget::Titlebar));
    assert_eq!(
        app.settings.theme.titlebar_color,
        wazoo_core::DEFAULT_THEME_TITLEBAR_COLOR
    );

    // Apply light preset & check light surface shifts darker instead of clipping white
    let _ = app.update(Message::ApplyThemePreset(wazoo_core::ThemePreset::FrostLight));
    assert!(app.settings.theme.matches_preset(wazoo_core::ThemePreset::FrostLight));
    assert!(app.theme_palette.card_bg.r < app.theme_palette.modal_bg.r);
    {
        let _view_light = app.view_settings_modal();
    }

    // Randomize & Reset all
    let _ = app.update(Message::RandomizeTheme);
    let _ = app.update(Message::ResetThemeSettings);
    assert_eq!(app.settings.theme, wazoo_core::ThemeSettings::default());
    assert!(app.settings.theme.matches_preset(wazoo_core::ThemePreset::Default));

    let _ = app.update(Message::CloseSettingsModal);
}
