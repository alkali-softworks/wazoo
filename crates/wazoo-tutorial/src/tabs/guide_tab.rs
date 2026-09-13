/*!
 * ==============================================================================
 * TUTORIAL LESSON: Rust & Iced In-App Cheat Sheet & Architecture Guide
 * ==============================================================================
 * 
 * This tab acts as an interactive, built-in study reference guide.
 * It synthesizes the key concepts you need to read, write, and understand
 * Rust applications like `wazoo-rs`.
 */

use iced::{
    widget::{column, container, rule, scrollable, text},
    Element, Length,
};

use crate::app::TutorialApp;
use crate::message::Message;
use crate::style;

impl TutorialApp {
    /// Renders Tab 5: Built-in Rust & Iced Reference Guide.
    pub fn view_guide_tab(&self) -> Element<'_, Message> {
        let title_card = container(
            column![
                text("📖 The Rust & Iced Survival Guide").size(24).color(style::COLOR_ACCENT),
                text("Keep this reference handy as you explore the code in `wazoo-tutorial` and `wazoo-rs`.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
            ]
            .spacing(6),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // Section 1: Ownership & Borrowing
        let section_ownership = container(
            column![
                text("1. Ownership & Borrowing (Rust's Superpower)").size(18).color(style::COLOR_ACCENT),
                rule::horizontal(1),
                text("Unlike C/C++ (manual malloc/free) or Java/Go/JS (garbage collection), Rust manages memory at compile time using ownership rules:").size(13),
                container(
                    column![
                        text("Rule 1: Each value in Rust has an owner (a variable).").size(13),
                        text("Rule 2: There can only be ONE owner at a time.").size(13),
                        text("Rule 3: When the owner goes out of scope, the value is automatically dropped.").size(13),
                    ]
                    .spacing(4),
                )
                .padding(10)
                .style(style::highlight_box_style),
                text("Borrowing allows functions to read or modify data without taking ownership:").size(13),
                text("• `&T`: An immutable reference (read-only). You can have many `&T` at once.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `&mut T`: A mutable reference (read & write). You can only have ONE `&mut T` at a time (preventing data races!).").size(12).color(style::COLOR_TEXT_MUTED),
            ]
            .spacing(10),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // Section 2: The Elm Architecture (TEA)
        let section_tea = container(
            column![
                text("2. The Elm Architecture in Iced").size(18).color(style::COLOR_ACCENT),
                rule::horizontal(1),
                text("Iced applications follow a strict unidirectional data flow:").size(13),
                container(
                    column![
                        text("┌─────────────────────────────────────────────────────────┐").size(12).color(style::COLOR_ACCENT),
                        text("│  1. VIEW (renders UI based on current State)            │").size(12),
                        text("│     └─► User interacts with widget (e.g. button click)  │").size(12),
                        text("│  2. MESSAGE (dispatched by widget)                      │").size(12),
                        text("│     └─► Delivered to `update(&mut self, message)`       │").size(12),
                        text("│  3. UPDATE (mutates State & optionally returns Task)    │").size(12),
                        text("│     └─► Triggers a new VIEW render cycle!               │").size(12),
                        text("└─────────────────────────────────────────────────────────┘").size(12).color(style::COLOR_ACCENT),
                    ]
                    .spacing(2),
                )
                .padding(10)
                .style(style::highlight_box_style),
                text("Key Takeaway: The View is pure and never directly mutates state. All mutations happen in `update`.").size(13),
            ]
            .spacing(10),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // Section 3: Error Handling & Enums
        let section_errors = container(
            column![
                text("3. Enums, Option, & Result").size(18).color(style::COLOR_ACCENT),
                rule::horizontal(1),
                text("Rust has NO null pointers and NO unhandled exceptions! Instead, it uses standard enums:").size(13),
                text("• `Option<T>`: Either `Some(T)` or `None`. Used when a value might be missing.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `Result<T, E>`: Either `Ok(T)` (success) or `Err(E)` (failure). Used for fallible operations.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• The `?` operator: Unwraps `Ok(val)` or returns early on `Err(e)`. Extremely clean error propagation!").size(12).color(style::COLOR_TEXT_MUTED),
            ]
            .spacing(10),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // Section 4: How Wazoo-RS Works
        let section_wazoo = container(
            column![
                text("4. How the Wazoo-RS Workspace is Structured").size(18).color(style::COLOR_ACCENT),
                rule::horizontal(1),
                text("`wazoo-rs` is a modular Cargo workspace divided into distinct crates:").size(13),
                text("• `wazoo-core`: Database persistence (`rusqlite`), configuration (`serde`), models, keybinds.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `wazoo-scanner`: Background file discovery (`walkdir`, regex, `tokio`) without blocking GUI.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `wazoo-media`: Hardware-accelerated video decoding, rendering buffers, OpenGL/Vulkan/wgpu.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `wazoo-app`: The top-level Iced GUI window, frameless layout, shortcuts, overlays.").size(12).color(style::COLOR_TEXT_MUTED),
                text("• `wazoo-tutorial` (this app!): Your hands-on sandbox to experiment and learn!").size(12).color(style::COLOR_ACCENT),
            ]
            .spacing(10),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        let content = column![
            title_card,
            section_ownership,
            section_tea,
            section_errors,
            section_wazoo,
        ]
        .spacing(20)
        .padding(20);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
