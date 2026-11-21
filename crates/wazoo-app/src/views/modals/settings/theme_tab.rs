/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Settings Modal - Theme Customization Tab
 *
 * Provides interactive HSV color pickers, hex inputs, quick swatches, one-click presets,
 * live titlebar/HUD preview, and geometry/opacity sliders for customizing Wazoo's palette.
 */

use super::badge::badge_with_reset;
use crate::app::WazooApp;
use crate::color_picker::{ColorSvPad, HueBar, hsv_to_rgb, rgb_to_hsv};
use crate::cursor;
use crate::message::Message;
use crate::state::{ModalState, ThemeColorTarget};
use crate::theme;
use crate::wrap::Wrap;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Shadow, Theme,
    widget::{Space, button, column, container, row, slider, text, text_input},
};
use wazoo_core::{
    DEFAULT_THEME_CORNER_RADIUS, DEFAULT_THEME_FOCUS_RING_WIDTH, DEFAULT_THEME_HUD_OPACITY,
    ThemePreset, parse_hex_rgb,
};

const QUICK_SWATCHES_ACCENT: &[&str] = &[
    "#42b883", "#89b4fa", "#008080", "#000080", "#fb2872", "#9fe22c", "#61dcf5", "#fbbd2d",
    "#ee9d2b", "#eb477e", "#ff2e97", "#33cc66",
];

const QUICK_SWATCHES_SURFACE: &[&str] = &[
    "#1f1f1f", "#141414", "#1e1e2e", "#181825", "#c0c0c0", "#20211c", "#25231d", "#3f3b31",
    "#f6e4ea", "#eed3dc", "#d1d5db", "#e5e7eb",
];

const QUICK_SWATCHES_TITLEBAR: &[&str] = &[
    "#1f092f", "#181825", "#000080", "#151613", "#1d1b16", "#25231d", "#e4c8d1", "#2b0f4a",
    "#1f1f1f", "#000000", "#312e81", "#e5e7eb",
];

const QUICK_SWATCHES_TEXT: &[&str] = &[
    "#ffffff", "#f7f7f2", "#cdd6f4", "#eadcb3", "#a99866", "#aaaa92", "#661a33", "#b6778c",
    "#334155", "#1f2937", "#111827", "#000000",
];

pub fn view_theme_tab<'a>(app: &'a WazooApp) -> Element<'a, Message> {
    let pal = app.theme_palette;

    // 1. Live Mini Preview (Titlebar + Focus Ring + HUD Pill)
    let preview_card = view_live_preview(app, pal);

    // 2. Presets Card
    let preset_items: Vec<(ThemePreset, &'static str, String)> = vec![
        (ThemePreset::Default, "🌿", app.t("settings.theme_preset_default")),
        (ThemePreset::Amber, "🔥", app.t("settings.theme_preset_amber")),
        (ThemePreset::Mocha, "☕", app.t("settings.theme_preset_mocha")),
        (ThemePreset::FrostLight, "☀️", app.t("settings.theme_preset_frost")),
        (ThemePreset::Monokai, "🎨", app.t("settings.theme_preset_monokai")),
        (ThemePreset::Turtle, "🐢", app.t("settings.theme_preset_turtle")),
        (ThemePreset::Strawberry, "🍓", app.t("settings.theme_preset_strawberry")),
    ];

    let preset_buttons: Vec<Element<'a, Message>> = preset_items
        .into_iter()
        .map(|(preset, icon, label)| {
            let is_active = app.settings.theme.matches_preset(preset);
            cursor::PointerCursor::new(
                button(text(format!("{icon} {label}")).size(11))
                    .style(theme::folder_chip_style(pal, is_active))
                    .on_press(Message::ApplyThemePreset(preset))
                    .padding([5, 10]),
            )
            .into()
        })
        .collect();

    let randomize_btn = cursor::PointerCursor::new(
        button(text(format!("🎲 {}", app.t("settings.theme_randomize"))).size(11))
            .style(theme::action_button_style(pal))
            .on_press(Message::RandomizeTheme)
            .padding([4, 10]),
    );

    let presets_card = container(
        column![
            row![
                text(app.t("settings.theme_presets"))
                    .size(11)
                    .font(theme::FONT_BOLD)
                    .color(pal.text_dim),
                Space::new().width(Length::Fill),
                randomize_btn,
            ]
            .align_y(Alignment::Center),
            Wrap::with_elements(preset_buttons)
                .spacing(6)
                .vertical_spacing(6)
                .align_items(Alignment::Center),
        ]
        .spacing(8),
    )
    .padding([10, 14])
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 3. Color Pickers Section
    let color_rows = column![
        text(format!("🎨 {}", app.t("settings.theme_section_colors")))
            .size(12)
            .font(theme::FONT_BOLD)
            .color(theme::readable_accent_on(pal, pal.modal_bg)),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::Accent,
            app.t("settings.theme_accent_color"),
            app.t("settings.theme_accent_color_desc"),
            QUICK_SWATCHES_ACCENT,
        ),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::Surface,
            app.t("settings.theme_surface_color"),
            app.t("settings.theme_surface_color_desc"),
            QUICK_SWATCHES_SURFACE,
        ),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::Titlebar,
            app.t("settings.theme_titlebar_color"),
            app.t("settings.theme_titlebar_color_desc"),
            QUICK_SWATCHES_TITLEBAR,
        ),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::TitlebarBadge,
            app.t("settings.theme_titlebar_badge_color"),
            app.t("settings.theme_titlebar_badge_color_desc"),
            QUICK_SWATCHES_TITLEBAR,
        ),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::Hud,
            app.t("settings.theme_hud_color"),
            app.t("settings.theme_hud_color_desc"),
            QUICK_SWATCHES_SURFACE,
        ),
        view_color_slot(
            app,
            pal,
            ThemeColorTarget::Text,
            app.t("settings.theme_text_color"),
            app.t("settings.theme_text_color_desc"),
            QUICK_SWATCHES_TEXT,
        ),
    ]
    .spacing(8);

    // 4. Appearance & Geometry Sliders
    let hud_pct = (app.settings.theme.hud_opacity * 100.0).round() as i32;
    let hud_diff = (app.settings.theme.hud_opacity - DEFAULT_THEME_HUD_OPACITY).abs() > 0.01;
    let hud_opacity_group = column![
        row![
            text(app.t("settings.theme_hud_opacity"))
                .size(13)
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                format!("{hud_pct}%"),
                hud_diff,
                Message::SetThemeHudOpacity(DEFAULT_THEME_HUD_OPACITY),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.25..=1.0,
                app.settings.theme.hud_opacity,
                Message::SetThemeHudOpacity,
            )
            .step(0.01_f32)
            .style(theme::settings_slider_style(pal))
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let radius_val = app.settings.theme.corner_radius;
    let radius_str = if radius_val < 0.05 {
        "Sharp".to_string()
    } else {
        format!("{:.2}x", radius_val)
    };
    let radius_diff = (radius_val - DEFAULT_THEME_CORNER_RADIUS).abs() > 0.02;
    let corner_radius_group = column![
        row![
            text(app.t("settings.theme_corner_radius"))
                .size(13)
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                radius_str,
                radius_diff,
                Message::SetThemeCornerRadius(DEFAULT_THEME_CORNER_RADIUS),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=2.0,
                app.settings.theme.corner_radius,
                Message::SetThemeCornerRadius,
            )
            .step(0.05_f32)
            .style(theme::settings_slider_style(pal))
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let ring_val = app.settings.theme.focus_ring_width;
    let ring_str = if ring_val < 0.1 {
        "Off".to_string()
    } else {
        format!("{:.1} px", ring_val)
    };
    let ring_diff = (ring_val - DEFAULT_THEME_FOCUS_RING_WIDTH).abs() > 0.1;
    let focus_ring_group = column![
        row![
            text(app.t("settings.theme_focus_ring_width"))
                .size(13)
                .color(pal.text_muted),
            Space::new().width(Length::Fill),
            badge_with_reset(
                pal,
                ring_str,
                ring_diff,
                Message::SetThemeFocusRingWidth(DEFAULT_THEME_FOCUS_RING_WIDTH),
            ),
        ]
        .align_y(Alignment::Center),
        cursor::PointerCursor::new(
            slider(
                0.0..=8.0,
                app.settings.theme.focus_ring_width,
                Message::SetThemeFocusRingWidth,
            )
            .step(0.5_f32)
            .style(theme::settings_slider_style(pal))
            .width(Length::Fill),
        ),
    ]
    .spacing(4);

    let appearance_box = container(
        column![
            text(format!("✨ {}", app.t("settings.theme_section_appearance")))
                .size(12)
                .font(theme::FONT_BOLD)
                .color(theme::readable_accent_on(pal, pal.card_bg)),
            hud_opacity_group,
            corner_radius_group,
            focus_ring_group,
        ]
        .spacing(10),
    )
    .padding(14)
    .style(theme::settings_card_style(pal))
    .width(Length::Fill);

    // 5. Reset All Button
    let reset_all_button = cursor::PointerCursor::new(
        button(
            row![
                text("↺").size(13),
                text(app.t("settings.theme_reset_all")).size(12),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .style(theme::settings_reset_button_style(pal))
        .on_press(Message::ResetThemeSettings)
        .padding([8, 16]),
    );

    let reset_row = row![Space::new().width(Length::Fill), reset_all_button]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    column![
        preview_card,
        presets_card,
        color_rows,
        appearance_box,
        reset_row,
    ]
    .spacing(12)
    .into()
}

fn view_live_preview<'a>(app: &'a WazooApp, pal: theme::ThemePalette) -> Element<'a, Message> {
    let mini_badge = container(
        text("Wazoo")
            .size(11)
            .font(theme::FONT_BOLD)
            .color(pal.badge_text),
    )
    .padding([2, 10])
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(pal.badge_bg)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: pal.radius(12.0),
                bottom_right: pal.radius(12.0),
                bottom_left: 0.0,
            },
            ..Default::default()
        },
        ..Default::default()
    });

    let mini_titlebar = container(
        row![
            mini_badge,
            Space::new().width(Length::Fixed(6.0)),
            text("ALKALI").size(9).font(theme::FONT_BOLD).color(pal.titlebar_brand_text),
            Space::new().width(Length::Fill),
            text("─   □   ✕")
                .size(10)
                .color(pal.titlebar_icon_color),
        ]
        .padding(iced::Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 0.0,
        })
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(22.0))
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(pal.titlebar_bg)),
        ..Default::default()
    });

    let mini_title_pill = container(
        text("Sample Title")
            .size(10)
            .font(theme::FONT_BOLD)
            .color(pal.hud_text),
    )
    .padding([3, 10])
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(theme::with_alpha(
            pal.hud_bg,
            pal.hud_opacity,
        ))),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: pal.radius(10.0),
                bottom_right: pal.radius(10.0),
                bottom_left: 0.0,
            },
            ..Default::default()
        },
        ..Default::default()
    });

    let mini_seekbar = container(
        row![
            container(Space::new().width(Length::FillPortion(6)).height(Length::Fixed(5.0)))
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(pal.primary)),
                    border: Border {
                        radius: pal.radius(2.5).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::new().width(Length::FillPortion(4)),
        ]
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fixed(5.0))
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(pal.track_bg)),
        border: Border {
            radius: pal.radius(2.5).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let mini_hud = container(
        column![
            row![
                text("CC")
                    .size(9)
                    .font(theme::FONT_BOLD)
                    .color(theme::readable_hud_accent_on(pal, pal.hud_bg)),
                Space::new().width(Length::Fill),
                text("▶  01:24 / 03:45").size(9).color(pal.hud_text),
            ]
            .align_y(Alignment::Center),
            mini_seekbar,
        ]
        .spacing(4),
    )
    .padding([5, 10])
    .width(Length::Fixed(220.0))
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(theme::with_alpha(
            pal.hud_bg,
            pal.hud_opacity,
        ))),
        border: Border {
            radius: pal.radius(10.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let mini_player = container(
        column![
            row![mini_title_pill, Space::new().width(Length::Fill)],
            Space::new().height(Length::Fixed(8.0)),
            container(mini_hud).width(Length::Fill).center_x(Length::Fill),
        ]
        .padding(iced::Padding {
            top: 6.0,
            right: 8.0,
            bottom: 8.0,
            left: 0.0,
        }),
    )
    .width(Length::Fill)
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(pal.empty_bg)),
        border: Border {
            radius: 0.0.into(),
            width: pal.focus_ring_width,
            color: pal.primary,
        },
        ..Default::default()
    });

    let preview_box = container(column![mini_titlebar, mini_player])
        .width(Length::Fill)
        .style(move |_: &Theme| container::Style {
            border: Border {
                radius: pal.radius(6.0).into(),
                width: 1.0,
                color: pal.border,
            },
            ..Default::default()
        });

    container(
        column![
            text(app.t("settings.theme_preview"))
                .size(11)
                .font(theme::FONT_BOLD)
                .color(pal.text_dim),
            preview_box,
        ]
        .spacing(6),
    )
    .padding([10, 14])
    .style(theme::settings_card_style(pal))
    .width(Length::Fill)
    .into()
}

fn view_color_slot<'a>(
    app: &'a WazooApp,
    pal: theme::ThemePalette,
    target: ThemeColorTarget,
    title: String,
    desc: String,
    quick_swatches: &'static [&'static str],
) -> Element<'a, Message> {
    let current_hex = target.get_hex(&app.settings.theme);
    let default_hex = target.default_hex();
    let is_modified = !current_hex.eq_ignore_ascii_case(default_hex);
    let is_expanded = app.modals.active_color_picker == Some(target);
    let (r, g, b) = parse_hex_rgb(current_hex).unwrap_or((128, 128, 128));
    let swatch_color = Color::from_rgb8(r, g, b);

    let toggle_target = if is_expanded { None } else { Some(target) };

    let swatch_btn = cursor::PointerCursor::new(
        button(Space::new().width(Length::Fixed(32.0)).height(Length::Fixed(20.0)))
            .style(move |_theme: &Theme, status| {
                let border_col = if is_expanded || matches!(status, button::Status::Hovered) {
                    pal.primary
                } else {
                    pal.border_hover
                };
                button::Style {
                    background: Some(Background::Color(swatch_color)),
                    border: Border {
                        radius: pal.radius(4.0).into(),
                        width: if is_expanded { 2.0 } else { 1.0 },
                        color: border_col,
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                }
            })
            .on_press(Message::SelectThemeColorTarget(toggle_target))
            .padding(0),
    );

    let draft_val = &app.modals.theme_hex_drafts[ModalState::target_index(target)];
    let hex_input = text_input("#rrggbb", draft_val)
        .on_input(move |s| Message::ThemeHexInputChanged(target, s))
        .style(theme::dark_input_style(pal))
        .padding([4, 8])
        .size(12)
        .width(Length::Fixed(76.0));

    let label_btn = cursor::PointerCursor::new(
        button(
            column![
                row![
                    text(title)
                        .size(13)
                        .font(theme::FONT_BOLD)
                        .color(if is_expanded { pal.primary } else { pal.text }),
                    Space::new().width(Length::Fixed(6.0)),
                    text(if is_expanded { "▾" } else { "▸" })
                        .size(11)
                        .color(if is_expanded {
                            pal.primary
                        } else {
                            pal.text_muted
                        }),
                ]
                .align_y(Alignment::Center),
                text(desc).size(11).color(pal.text_muted),
            ]
            .spacing(2),
        )
        .style(move |_theme: &Theme, _status| button::Style {
            background: None,
            text_color: pal.text,
            ..Default::default()
        })
        .on_press(Message::SelectThemeColorTarget(toggle_target))
        .padding(0)
        .width(Length::Fill),
    );

    let mut header_row = row![label_btn, swatch_btn, hex_input,]
        .spacing(8)
        .align_y(Alignment::Center);

    if is_modified {
        header_row = header_row.push(cursor::PointerCursor::new(
            button(text("↺").size(11))
                .style(theme::settings_reset_button_style(pal))
                .on_press(Message::ResetThemeColor(target))
                .padding([3, 6]),
        ));
    } else {
        header_row = header_row.push(Space::new().width(Length::Fixed(22.0)));
    }

    let mut card_col = column![header_row].spacing(10);

    if is_expanded {
        let (h, s, v) = app.modals.theme_picker_hsv;
        let (pr, pg, pb) = hsv_to_rgb(h, s, v);

        let sv_pad = ColorSvPad::new(h, s, v, move |new_s, new_v| {
            Message::SetThemeColorHsv(target, h, new_s, new_v)
        })
        .height(128.0)
        .border_color(pal.border)
        .radius(pal.radius(6.0))
        .on_commit(Message::CommitThemeColor);

        let hue_bar = HueBar::new(h, move |new_h| {
            Message::SetThemeColorHsv(target, new_h, s, v)
        })
        .height(18.0)
        .border_color(pal.border)
        .radius(pal.radius(6.0))
        .on_commit(Message::CommitThemeColor);

        let swatches: Vec<Element<'a, Message>> = quick_swatches
            .iter()
            .map(|&swatch_hex| {
                let (sr, sg, sb) = parse_hex_rgb(swatch_hex).unwrap_or((0, 0, 0));
                let sc = Color::from_rgb8(sr, sg, sb);
                let is_selected = current_hex.eq_ignore_ascii_case(swatch_hex);
                cursor::PointerCursor::new(
                    button(Space::new().width(Length::Fixed(20.0)).height(Length::Fixed(20.0)))
                        .style(move |_theme: &Theme, status| {
                            let border_col = if is_selected {
                                pal.primary
                            } else if matches!(status, button::Status::Hovered) {
                                pal.text
                            } else {
                                pal.border
                            };
                            button::Style {
                                background: Some(Background::Color(sc)),
                                border: Border {
                                    radius: pal.radius(4.0).into(),
                                    width: if is_selected { 2.0 } else { 1.0 },
                                    color: border_col,
                                },
                                ..Default::default()
                            }
                        })
                        .on_press(Message::ThemeHexInputChanged(
                            target,
                            swatch_hex.to_string(),
                        ))
                        .padding(0),
                )
                .into()
            })
            .collect();

        let (dh, ds, dv) = rgb_to_hsv(pr, pg, pb);
        let readout = row![
            text(format!(
                "H: {:.0}°   S: {:.0}%   V: {:.0}%",
                dh,
                ds * 100.0,
                dv * 100.0
            ))
            .size(11)
            .color(pal.text_muted),
            Space::new().width(Length::Fill),
            text(format!("RGB({pr}, {pg}, {pb})"))
                .size(11)
                .color(pal.text_dim),
        ]
        .align_y(Alignment::Center);

        let picker_body = column![
            sv_pad,
            hue_bar,
            readout,
            Wrap::with_elements(swatches)
                .spacing(6)
                .vertical_spacing(6)
                .align_items(Alignment::Center),
        ]
        .spacing(8);

        card_col = card_col.push(picker_body);
    }

    container(card_col)
        .padding([10, 12])
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(pal.card_bg)),
            border: Border {
                radius: pal.radius(8.0).into(),
                width: 1.0,
                color: if is_expanded { pal.primary } else { pal.border },
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}
