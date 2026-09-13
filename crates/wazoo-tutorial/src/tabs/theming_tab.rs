/*!
 * ==============================================================================
 * TUTORIAL LESSON: Theming & Custom Design Systems in Iced
 * ==============================================================================
 * 
 * One of the best features of Iced is its rich, built-in theming system.
 * Iced ships with dozens of popular developer themes:
 * - `Theme::Dark`
 * - `Theme::Light`
 * - `Theme::TokyoNight`
 * - `Theme::CatppuccinMocha`
 * - `Theme::Dracula`
 * - `Theme::Nord`
 * - `Theme::SolarizedDark`
 * - `Theme::SolarizedLight`
 * 
 * How dynamic theming works in Iced:
 * 1. Your application registers a theme function: `.theme(TutorialApp::theme)`
 * 2. When the user selects a different theme, `self.theme` updates.
 * 3. Iced re-renders the UI using the new theme's color palette!
 * 
 * In this tab, you can switch themes on the fly and see how standard widgets
 * and custom-styled widgets adapt automatically.
 */

use iced::{
    widget::{button, column, container, pick_list, row, rule, scrollable, text},
    Alignment, Element, Length, Theme,
};

use crate::app::TutorialApp;
use crate::message::Message;
use crate::style;

/// List of themes supported in our selector.
pub const AVAILABLE_THEMES: &[Theme] = &[
    Theme::Dark,
    Theme::TokyoNight,
    Theme::CatppuccinMocha,
    Theme::Dracula,
    Theme::Nord,
    Theme::SolarizedDark,
    Theme::Light,
];

impl TutorialApp {
    /// Renders Tab 4: Theming & Styling Playground.
    pub fn view_theming_tab(&self) -> Element<'_, Message> {
        // ======================================================================
        // Section 1: Theme Selector
        // ======================================================================
        // The `pick_list` widget takes:
        // 1. A slice of available options: `AVAILABLE_THEMES`
        // 2. The currently selected option: `Some(&self.current_theme)`
        // 3. A message constructor callback: `Message::ThemeSelected`
        let theme_selector = pick_list(
            AVAILABLE_THEMES,
            Some(&self.current_theme),
            Message::ThemeSelected,
        )
        .padding(8);

        let current_palette = self.current_theme.palette();

        let selector_card = container(
            column![
                row![
                    text("Dynamic Application Theming").size(20),
                    container(text("Live Hot-Swapping").size(11).color(style::COLOR_ACCENT))
                        .style(style::badge_style)
                        .padding([2, 8]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text("Switch the active Iced theme below. Watch all widgets update their palette instantly!")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                row![
                    text("Active Theme:").size(15),
                    theme_selector,
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 2: Palette Color Inspector
        // ======================================================================
        // Helper closure to render a color swatch tile
        let color_chip = |label: &'static str, color: iced::Color| {
            container(
                column![
                    container(text("").size(1))
                        .width(Length::Fixed(60.0))
                        .height(Length::Fixed(30.0))
                        .style(move |_theme| container::Style {
                            background: Some(iced::Background::Color(color)),
                            border: iced::Border {
                                radius: iced::border::Radius::from(4.0),
                                width: 1.0,
                                color: ColorChipBorder::color(),
                            },
                            ..Default::default()
                        }),
                    text(label).size(11).color(style::COLOR_TEXT_MUTED),
                ]
                .spacing(4)
                .align_x(Alignment::Center),
            )
        };

        let palette_row = row![
            color_chip("Background", current_palette.background),
            color_chip("Text", current_palette.text),
            color_chip("Primary", current_palette.primary),
            color_chip("Success", current_palette.success),
            color_chip("Danger", current_palette.danger),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let palette_card = container(
            column![
                text("Active Theme Palette Swatches").size(18),
                text("Every Iced theme provides a standardized palette used by built-in widgets.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                palette_row,
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 3: Component Showcase Under Current Theme
        // ======================================================================
        let showcase_card = container(
            column![
                text("Widget Showcase Under Selected Theme").size(18),
                text("Standard Iced buttons automatically inherit the active theme's colors when not overridden by custom styles.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                rule::horizontal(1),
                row![
                    button("Default Primary Button").padding([8, 16]),
                    button("Secondary / Outlined")
                        .style(button::secondary)
                        .padding([8, 16]),
                    button("Danger Button")
                        .style(button::danger)
                        .padding([8, 16]),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 4: Rust Concept Explainer Box
        // ======================================================================
        let concept_box = container(
            column![
                text("💡 How Wazoo-RS Styles Desktop Windows").size(16).color(style::COLOR_ACCENT),
                text("• Custom Closures: In `wazoo-app/src/theme.rs`, Wazoo uses custom styling closures like `|theme, status| button::Style { ... }` for a dark ambient look.").size(13),
                text("• Frameless Windows: Wazoo sets `.window(iced::window::Settings { decorations: false, transparent: true, .. })` for a borderless glass aesthetic.").size(13),
                text("• Zero Overhead: Iced renders directly using WebGPU (`wgpu`) or Vulkan/Metal/DirectX, with smooth 60+ FPS hardware acceleration!").size(13),
            ]
            .spacing(8),
        )
        .padding(16)
        .style(style::highlight_box_style)
        .width(Length::Fill);

        let content = column![
            selector_card,
            palette_card,
            showcase_card,
            concept_box,
        ]
        .spacing(20)
        .padding(20);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Helper struct for static border color
struct ColorChipBorder;
impl ColorChipBorder {
    fn color() -> iced::Color {
        iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2)
    }
}
