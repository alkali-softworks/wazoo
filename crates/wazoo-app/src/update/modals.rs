/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modals & Settings Dialogs Reducer
 *
 * Handles lifecycle, tab selection, and setting updates for Search, Settings, Help, Bookmarks,
 * and Menu modals.
 */

use crate::app::WazooApp;
use crate::message::Message;
use iced::Task;

impl WazooApp {
    /// Handles messages for modal dialog visibility (Settings, Search, Help, Bookmarks, Menu),
    /// tab switching, and user preference persistence to disk.
    pub(crate) fn update_modals(&mut self, message: Message) -> Task<Message> {
        let exit_task = if matches!(
            message,
            Message::OpenMenuModal
                | Message::OpenSearchModal
                | Message::OpenSettingsModal
                | Message::OpenPlexSettingsModal
                | Message::OpenHelpModal
                | Message::ToggleBookmarksModal
        ) && self.cube.desktop_overlay
        {
            self.exit_desktop_cube_overlay()
        } else {
            Task::none()
        };

        match message {
            Message::OpenMenuModal => {
                self.modals.close_all();
                self.modals.menu = true;
                self.titlebar.show_dropdown_menu = false;
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                exit_task
            }
            Message::CloseMenuModal => {
                self.modals.menu = false;
                Task::none()
            }
            Message::OpenSearchModal => {
                self.modals.close_all();
                self.modals.search = true;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                self.search.tags = self
                    .search
                    .active_query
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.search.selected_folders = self.search.active_folders.clone();
                self.search.selected_folder = self.search.active_folder.clone();
                self.search.input.clear();
                Task::batch([exit_task, iced::widget::operation::focus("search_input")])
            }
            Message::CloseSearchModal => {
                self.modals.search = false;
                Task::none()
            }
            Message::OpenSettingsModal => {
                self.modals.close_all();
                self.modals.settings = true;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                exit_task
            }
            Message::CloseSettingsModal => {
                if self.modals.playback_settings_debounce_ticks > 0 {
                    self.apply_playback_settings();
                }
                self.modals.settings = false;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetSettingsTab(tab) => {
                if self.modals.playback_settings_debounce_ticks > 0 {
                    self.apply_playback_settings();
                }
                self.modals.settings_tab = tab;
                Task::none()
            }
            Message::SetLanguage(lang) => {
                let was_all_active = self.search.active_folders.is_empty()
                    || self.is_all_folder(&self.search.active_folder);
                let was_all_selected = self.search.selected_folders.is_empty()
                    || self.is_all_folder(&self.search.selected_folder);
                self.settings.language = lang;
                if was_all_active {
                    self.search.active_folder = self.t("common.all");
                }
                if was_all_selected {
                    self.search.selected_folder = self.t("common.all");
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                for p in &mut self.players {
                    p.set_i18n_language(Some(self.settings.language.clone()));
                }
                Task::none()
            }
            Message::SetGamma(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.gamma = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetContrast(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.contrast = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetBrightness(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.brightness = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetSaturation(val) => {
                let clamped = val.clamp(-100.0, 100.0);
                self.settings.saturation = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::SetPlaybackSpeed(val) => {
                let clamped = val.clamp(0.25, 3.0);
                self.settings.playback_speed = clamped;
                self.modals.playback_settings_debounce_ticks =
                    crate::app::PLAYBACK_SETTINGS_DEBOUNCE_TICKS;
                Task::none()
            }
            Message::ToggleFilters => {
                let new_state = !self.settings.filters_enabled;
                self.settings.filters_enabled = new_state;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                let key = self.settings.keybinds.toggle_filters.to_uppercase();
                let msg = if new_state {
                    self.t_with("toast.filters_enabled", &[("key", &key)])
                } else {
                    self.t("toast.filters_disabled")
                };
                self.overlay.show_toast(msg);
                Task::none()
            }
            Message::SetFiltersEnabled(enabled) => {
                self.settings.filters_enabled = enabled;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleCrtFilter => {
                let new_state = !self.settings.filter_crt;
                self.settings.filter_crt = new_state;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtFilter(enabled) => {
                self.settings.filter_crt = enabled;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtScanlineDarkness(val) => {
                self.settings.crt.scanline_darkness = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtScanlineDensity(val) => {
                self.settings.crt.scanline_density = val.clamp(50.0, 2000.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtScanlineSpeed(val) => {
                self.settings.crt.scanline_speed = val.clamp(0.0, 20.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtCurvature(val) => {
                self.settings.crt.curvature = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtGamma(val) => {
                self.settings.crt.gamma = val.clamp(0.2, 3.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtSaturation(val) => {
                self.settings.crt.saturation = val.clamp(0.0, 3.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtBrightness(val) => {
                self.settings.crt.brightness = val.clamp(0.2, 3.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtVignette(val) => {
                self.settings.crt.vignette = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtChromaticAberration(val) => {
                self.settings.crt.chromatic_aberration = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtNoise(val) => {
                self.settings.crt.noise = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtCornerRounding(val) => {
                self.settings.crt.corner_rounding = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtGlitchIntensity(val) => {
                self.settings.crt.glitch_intensity = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetCrtGlitchFrequency(val) => {
                self.settings.crt.glitch_frequency = val.clamp(0.0, 1.0);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ApplyCrtPreset(preset) => {
                self.settings.crt.apply_preset(preset);
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ResetCrtSettings => {
                self.settings.crt = wazoo_core::CrtSettings::default();
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SelectThemeColorTarget(target) => {
                self.modals.active_color_picker = target;
                self.modals.sync_picker_from_theme(&self.settings.theme);
                Task::none()
            }
            Message::SetThemeColorHsv(target, h, s, v) => {
                let h = h.rem_euclid(360.0);
                let s = s.clamp(0.0, 1.0);
                let v = v.clamp(0.0, 1.0);
                self.modals.theme_picker_hsv = (h, s, v);
                let (r, g, b) = crate::color_picker::hsv_to_rgb(h, s, v);
                let hex = wazoo_core::format_hex_rgb(r, g, b);
                target.set_hex(&mut self.settings.theme, hex.clone());
                self.modals.theme_hex_drafts[crate::state::ModalState::target_index(target)] = hex;
                self.sync_theme_palette();
                Task::none()
            }
            Message::CommitThemeColor => {
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ThemeHexInputChanged(target, input) => {
                let idx = crate::state::ModalState::target_index(target);
                // Keep user's in-progress text in the draft box (capped at 7 chars like #rrggbb)
                let cleaned: String = input.chars().take(7).collect();
                self.modals.theme_hex_drafts[idx] = cleaned.clone();
                if let Some((r, g, b)) = wazoo_core::parse_hex_rgb(&cleaned) {
                    let canonical = wazoo_core::format_hex_rgb(r, g, b);
                    target.set_hex(&mut self.settings.theme, canonical);
                    if self.modals.active_color_picker == Some(target) {
                        self.modals.theme_picker_hsv = crate::color_picker::rgb_to_hsv(r, g, b);
                    }
                    self.sync_theme_palette();
                    let _ = self.config_mgr.save_settings(&self.settings);
                }
                Task::none()
            }
            Message::ResetThemeColor(target) => {
                target.set_hex(&mut self.settings.theme, target.default_hex().to_string());
                self.sync_theme_palette();
                self.modals.sync_picker_from_theme(&self.settings.theme);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetThemeHudOpacity(val) => {
                self.settings.theme.hud_opacity = val.clamp(0.25, 1.0);
                self.sync_theme_palette();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetThemeCornerRadius(val) => {
                self.settings.theme.corner_radius = val.clamp(0.0, 2.0);
                self.sync_theme_palette();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetThemeFocusRingWidth(val) => {
                self.settings.theme.focus_ring_width = val.clamp(0.0, 8.0);
                self.sync_theme_palette();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ApplyThemePreset(preset) => {
                self.settings.theme.apply_preset(preset);
                self.sync_theme_palette();
                self.modals.sync_picker_from_theme(&self.settings.theme);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::RandomizeTheme => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let base_hue = rng.gen_range(0.0_f32..360.0);
                let accent_rgb = crate::color_picker::hsv_to_rgb(
                    base_hue,
                    rng.gen_range(0.55_f32..0.90),
                    rng.gen_range(0.75_f32..0.96),
                );
                let titlebar_hue = (base_hue + rng.gen_range(90.0_f32..240.0)).rem_euclid(360.0);
                let titlebar_rgb = crate::color_picker::hsv_to_rgb(
                    titlebar_hue,
                    rng.gen_range(0.45_f32..0.85),
                    rng.gen_range(0.14_f32..0.28),
                );
                let surface_rgb = crate::color_picker::hsv_to_rgb(
                    base_hue,
                    rng.gen_range(0.04_f32..0.22),
                    rng.gen_range(0.09_f32..0.18),
                );
                let badge_rgb = crate::color_picker::hsv_to_rgb(
                    titlebar_hue,
                    rng.gen_range(0.30_f32..0.70),
                    rng.gen_range(0.03_f32..0.10),
                );
                let hud_rgb = crate::color_picker::hsv_to_rgb(
                    base_hue,
                    rng.gen_range(0.08_f32..0.25),
                    rng.gen_range(0.06_f32..0.13),
                );
                self.settings.theme.accent_color =
                    wazoo_core::format_hex_rgb(accent_rgb.0, accent_rgb.1, accent_rgb.2);
                self.settings.theme.surface_color =
                    wazoo_core::format_hex_rgb(surface_rgb.0, surface_rgb.1, surface_rgb.2);
                self.settings.theme.titlebar_color =
                    wazoo_core::format_hex_rgb(titlebar_rgb.0, titlebar_rgb.1, titlebar_rgb.2);
                self.settings.theme.titlebar_badge_color =
                    wazoo_core::format_hex_rgb(badge_rgb.0, badge_rgb.1, badge_rgb.2);
                self.settings.theme.hud_color =
                    wazoo_core::format_hex_rgb(hud_rgb.0, hud_rgb.1, hud_rgb.2);
                self.settings.theme.text_color =
                    wazoo_core::DEFAULT_THEME_TEXT_COLOR.to_string();
                self.sync_theme_palette();
                self.modals.sync_picker_from_theme(&self.settings.theme);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ResetThemeSettings => {
                self.settings.theme = wazoo_core::ThemeSettings::default();
                self.sync_theme_palette();
                self.modals.sync_picker_from_theme(&self.settings.theme);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleWavyFilter => {
                let new_state = !self.settings.filter_wavy;
                self.settings.filter_wavy = new_state;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetWavyFilter(enabled) => {
                self.settings.filter_wavy = enabled;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleFogFilter => {
                let new_state = !self.settings.filter_fog;
                self.settings.filter_fog = new_state;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetFogFilter(enabled) => {
                self.settings.filter_fog = enabled;
                self.sync_player_filters();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetLoadingIndicator(indicator) => {
                self.settings.loading_indicator = indicator;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetBufferDuration(secs) => {
                self.settings.buffer_duration_secs = secs.clamp(2, 60);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetBufferSize(mb) => {
                self.settings.buffer_size_mb = mb.clamp(8, 1024);
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::SetFlipInterval(secs) => {
                let clamped = secs.clamp(1, 3600);
                self.settings.flip_interval_secs = clamped;
                self.stagger_flip_countdowns();
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ResetPlaybackOptions => {
                self.modals.playback_settings_debounce_ticks = 0;
                self.settings.gamma = 0.0;
                self.settings.contrast = 0.0;
                self.settings.brightness = 0.0;
                self.settings.saturation = 0.0;
                self.settings.playback_speed = 1.0;
                self.settings.filters_enabled = false;
                self.settings.filter_crt = true;
                self.settings.filter_wavy = false;
                self.settings.filter_fog = false;
                self.sync_player_filters();
                for player in &mut self.players {
                    player.set_equalizer(0.0, 0.0, 0.0, 0.0);
                    player.set_speed(1.0);
                }
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }
            Message::ToggleDefaultPlayer => {
                self.settings.is_default_player = !self.settings.is_default_player;
                let _ = self.config_mgr.save_settings(&self.settings);
                #[cfg(target_os = "linux")]
                {
                    if self.settings.is_default_player {
                        let _ = crate::platform::set_as_default_video_player();
                    } else {
                        let _ = crate::platform::unset_as_default_video_player();
                    }
                }
                Task::none()
            }
            Message::ToggleStandbyPlayer => {
                self.settings.use_standby_player = !self.settings.use_standby_player;
                let _ = self.config_mgr.save_settings(&self.settings);
                if !self.settings.use_standby_player {
                    if let Ok(mut guard) = self.standby_player.lock() {
                        if let Some(handle) = guard.take() {
                            std::thread::spawn(move || drop(handle));
                        }
                    }
                } else {
                    self.replenish_standby_player();
                }
                Task::none()
            }
            Message::OpenAlkaliWebsite => {
                crate::platform::open_url("https://alkalisoftworks.com/");
                Task::none()
            }
            Message::OpenHelpModal => {
                self.modals.close_all();
                self.modals.help = true;
                self.modals.help_search.clear();
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                Task::batch([
                    exit_task,
                    iced::widget::operation::focus("help_search_input"),
                ])
            }
            Message::CloseHelpModal => {
                self.modals.help = false;
                self.modals.help_search.clear();
                Task::none()
            }
            Message::HelpSearchChanged(val) => {
                self.modals.help_search = val;
                Task::none()
            }
            Message::ClearHelpSearch => {
                self.modals.help_search.clear();
                iced::widget::operation::focus("help_search_input")
            }
            Message::ToggleBookmarksModal => {
                self.modals.bookmarks = !self.modals.bookmarks;
                if self.modals.bookmarks {
                    self.titlebar.show_dropdown_menu = false;
                    self.modals.menu = false;
                    self.modals.search = false;
                    self.modals.settings = false;
                    self.modals.help = false;
                    self.titlebar.show = false;
                    self.overlay.clear_toast();
                    self.hovered_player_id = None;
                    self.overlay.ticks = 0;
                    self.overlay.fade_in_ticks = 0;
                }
                exit_task
            }
            Message::CloseBookmarksModal => {
                self.modals.bookmarks = false;
                Task::none()
            }
            Message::ToggleQuickPickerModal => {
                if self.modals.quick_picker {
                    self.modals.quick_picker = false;
                    self.modals.quick_picker_search.clear();
                    self.modals.quick_picker_cache = None;
                    Task::none()
                } else {
                    self.open_quick_picker()
                }
            }
            Message::OpenQuickPickerModal => self.open_quick_picker(),
            Message::CloseQuickPickerModal => {
                self.modals.quick_picker = false;
                self.modals.quick_picker_search.clear();
                self.modals.quick_picker_cache = None;
                Task::none()
            }
            Message::QuickPickerSearchChanged(val) => {
                self.modals.quick_picker_search = val;
                Task::none()
            }
            Message::ClearQuickPickerSearch => {
                self.modals.quick_picker_search.clear();
                iced::widget::operation::focus("quick_picker_search_input")
            }
            Message::QuickPickerSubmit => {
                let first_match = if let Some(cache) = self.modals.quick_picker_cache.as_ref() {
                    crate::views::modals::quick_picker_modal::filter_quick_picker_entries(
                        &cache.entries,
                        &self.modals.quick_picker_search,
                    )
                    .first()
                    .map(|e| e.path.clone())
                } else {
                    let cache = self.build_quick_picker_cache();
                    crate::views::modals::quick_picker_modal::filter_quick_picker_entries(
                        &cache.entries,
                        &self.modals.quick_picker_search,
                    )
                    .first()
                    .map(|e| e.path.clone())
                };
                if let Some(p) = first_match {
                    return self.update(Message::PlayQuickPickerFile(p));
                }
                Task::none()
            }
            Message::PlayQuickPickerFile(path) => {
                self.modals.quick_picker = false;
                self.modals.quick_picker_search.clear();
                self.modals.quick_picker_cache = None;
                self.update(Message::PlayFileInFocused(path))
            }
            Message::QuickPickerDiskSiblingsLoaded {
                source_path,
                folder_name,
                candidates,
            } => {
                self.merge_quick_picker_disk_siblings(&source_path, folder_name, candidates);
                Task::none()
            }
            Message::TogglePlexExplorerModal => {
                if self.modals.plex_explorer {
                    self.modals.plex_explorer = false;
                    self.modals.plex_explorer_search.clear();
                    self.modals.plex_explorer_genre = None;
                    self.modals.plex_explorer_decade = None;
                    Task::none()
                } else {
                    self.open_plex_explorer()
                }
            }
            Message::OpenPlexExplorerModal => self.open_plex_explorer(),
            Message::OpenPlexSettingsModal => {
                if self.modals.playback_settings_debounce_ticks > 0 {
                    self.apply_playback_settings();
                }
                self.modals.close_all();
                self.modals.settings = true;
                self.modals.settings_tab = crate::app::SettingsTab::Plex;
                self.titlebar.show_dropdown_menu = false;
                self.titlebar.show = false;
                self.overlay.clear_toast();
                self.hovered_player_id = None;
                self.overlay.ticks = 0;
                self.overlay.fade_in_ticks = 0;
                exit_task
            }
            Message::ClosePlexExplorerModal => {
                self.modals.plex_explorer = false;
                self.modals.plex_explorer_search.clear();
                self.modals.plex_explorer_genre = None;
                self.modals.plex_explorer_decade = None;
                Task::none()
            }
            Message::PlexExplorerSearchChanged(val) => {
                self.modals.plex_explorer_search = val;
                self.refresh_plex_explorer_items();
                Task::none()
            }
            Message::ClearPlexExplorerSearch => {
                self.modals.plex_explorer_search.clear();
                self.refresh_plex_explorer_items();
                iced::widget::operation::focus("plex_explorer_search_input")
            }
            Message::PlexExplorerSelectTab(tab) => {
                let task = match tab {
                    wazoo_core::PlexExplorerTab::Decade(dec) => {
                        self.modals.plex_explorer_tab = wazoo_core::PlexExplorerTab::All;
                        self.modals.plex_explorer_decade = Some(dec);
                        self.modals.plex_explorer_genre = None;
                        Task::none()
                    }
                    wazoo_core::PlexExplorerTab::Genre(genre) => {
                        self.modals.plex_explorer_tab = wazoo_core::PlexExplorerTab::All;
                        self.modals.plex_explorer_genre = Some(genre);
                        self.modals.plex_explorer_decade = None;
                        Task::none()
                    }
                    wazoo_core::PlexExplorerTab::Random => {
                        self.modals.plex_explorer_tab = wazoo_core::PlexExplorerTab::Random;
                        self.modals.plex_explorer_random_seed = rand::random::<u64>();
                        Task::none()
                    }
                    _ => {
                        self.modals.plex_explorer_tab = tab.clone();
                        self.fetch_plex_live_items(tab)
                    }
                };
                self.refresh_plex_explorer_items();
                task
            }
            Message::PlexExplorerToggleGenre(genre) => {
                if genre.is_none() || self.modals.plex_explorer_genre == genre {
                    self.modals.plex_explorer_genre = None;
                } else {
                    self.modals.plex_explorer_genre = genre;
                }
                self.refresh_plex_explorer_items();
                Task::none()
            }
            Message::PlexExplorerToggleDecade(decade) => {
                if decade.is_none() || self.modals.plex_explorer_decade == decade {
                    self.modals.plex_explorer_decade = None;
                } else {
                    self.modals.plex_explorer_decade = decade;
                }
                self.refresh_plex_explorer_items();
                Task::none()
            }
            Message::PlexExplorerSubmit => {
                let items = self.plex_explorer_items();
                if let Some(first) = items.first() {
                    let p = first.path.clone();
                    let offset = first.view_offset_ms;
                    return self.update(Message::PlayPlexExplorerItem(p, offset));
                }
                Task::none()
            }
            Message::PlayPlexExplorerItem(path, view_offset_ms) => {
                self.modals.plex_explorer = false;
                self.modals.plex_explorer_search.clear();
                self.modals.plex_explorer_genre = None;
                self.modals.plex_explorer_decade = None;
                let play_task = self.update(Message::PlayFileInFocused(path));
                if let (Some(offset_ms), Some(pid)) = (view_offset_ms, self.focused_player_id()) {
                    if offset_ms > 0 {
                        return Task::batch([
                            play_task,
                            self.update(Message::Seek(
                                pid,
                                std::time::Duration::from_millis(offset_ms),
                            )),
                        ]);
                    }
                }
                play_task
            }
            Message::PlexExplorerLiveLoaded(tab, raw_items) => {
                let items = self.db.match_plex_raw_items(&raw_items).unwrap_or_default();
                self.plex.live_items.insert(tab, items);
                self.refresh_plex_explorer_items();
                Task::none()
            }
            Message::EscapePressed => {
                if self.cube.desktop_overlay {
                    return self.update(Message::ToggleDesktopCubeScreensaver);
                }
                if self.drawers.show_transcript_menu {
                    self.drawers.show_transcript_menu = false;
                    return Task::none();
                }
                if self.open_audio_menu_id.is_some() {
                    self.open_audio_menu_id = None;
                    return Task::none();
                }
                if self.is_any_modal_open()
                    || self.titlebar.show_dropdown_menu
                    || self.drawers.is_any_open()
                {
                    if self.modals.playback_settings_debounce_ticks > 0 {
                        self.apply_playback_settings();
                    }
                    self.modals.close_all();
                    self.titlebar.show_dropdown_menu = false;
                    self.close_file_picker();
                    self.drawers.close_all();
                } else if self.window.is_fullscreen {
                    return self.update(Message::ToggleFullscreen);
                } else {
                    self.modals.menu = true;
                    self.titlebar.show_dropdown_menu = false;
                }
                Task::none()
            }
            Message::DismissToast => {
                self.overlay.clear_toast();
                Task::none()
            }
            Message::ModalCardClicked => Task::none(),
            _ => Task::none(),
        }
    }

    /// Applies pending debounced equalizer and playback speed settings across all active video players
    /// and commits the settings to persistent storage.
    pub fn apply_playback_settings(&mut self) {
        self.modals.playback_settings_debounce_ticks = 0;
        let gamma = self.settings.gamma as f64;
        let contrast = self.settings.contrast as f64;
        let brightness = self.settings.brightness as f64;
        let saturation = self.settings.saturation as f64;
        let speed = self.settings.playback_speed as f64;

        for player in &mut self.players {
            player.set_equalizer(gamma, contrast, brightness, saturation);
            player.set_speed(speed);
        }
        let _ = self.config_mgr.save_settings(&self.settings);
    }

    /// Synchronizes active GPU shader filters (CRT, Wavy, Fog) across all active video players
    /// based on the master filters_enabled switch and individual filter checkboxes.
    pub fn sync_player_filters(&mut self) {
        let master = self.settings.filters_enabled;
        let crt = master && self.settings.filter_crt;
        let wavy = master && self.settings.filter_wavy;
        let fog = master && self.settings.filter_fog;
        self.settings.crt_enabled = crt;
        for player in &mut self.players {
            player.set_filters(crt, wavy, fog);
            player.set_crt_settings(self.settings.crt);
        }
    }
}
