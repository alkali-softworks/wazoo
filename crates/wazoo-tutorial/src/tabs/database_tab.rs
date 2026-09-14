/*!
 * ==============================================================================
 * TUTORIAL LESSON: SQLite Persistence UI & `rusqlite` CRUD
 * ==============================================================================
 *
 * This tab provides a live interactive interface for interacting with the
 * embedded SQLite database implemented in `src/db.rs`.
 *
 * In `wazoo-rs`:
 * - `wazoo-core/src/db.rs` stores indexed videos, bookmarks, and playback positions.
 * - This tutorial app provides a lightweight "Notes & Bookmarks Vault" to teach
 *   the exact same mechanics.
 *
 * Key Lessons:
 * 1. Building input forms that dispatch data to database transactions.
 * 2. Mapping dynamic SQLite query results into Iced widgets.
 * 3. Handling row deletion with row-specific identifiers (`DeleteNoteClicked(id)`).
 * 4. Graceful error handling (displaying error banners instead of crashing / panicking).
 */

use iced::{
    widget::{button, column, container, row, rule, scrollable, text, text_input},
    Alignment, Element, Length,
};

use crate::app::TutorialApp;
use crate::message::Message;
use crate::style;

impl TutorialApp {
    /// Renders Tab 3: Embedded SQLite Database CRUD.
    pub fn view_database_tab(&self) -> Element<'_, Message> {
        // ======================================================================
        // Section 1: Note Creation Form
        // ======================================================================
        let title_input = text_input(
            "Note title (e.g. 'Rust Borrow Checker Tips')...",
            &self.new_note_title,
        )
        .on_input(Message::NoteTitleChanged)
        .padding(10)
        .size(14);

        let content_input = text_input("Note content...", &self.new_note_content)
            .on_input(Message::NoteContentChanged)
            .padding(10)
            .size(14);

        let can_save = !self.new_note_title.trim().is_empty();
        let mut save_btn = button(
            row![text("💾").size(16), text("Save Note to SQLite").size(14),]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .style(style::primary_button_style)
        .padding([8, 16]);

        if can_save {
            save_btn = save_btn.on_press(Message::SaveNoteClicked);
        }

        let refresh_btn = button(
            row![text("🔄").size(16), text("Refresh List").size(14),]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .style(style::secondary_button_style)
        .on_press(Message::RefreshNotesClicked)
        .padding([8, 16]);

        let form_card = container(
            column![
                row![
                    text("Add Record to SQLite").size(20),
                    container(text("ACID Persistence").size(11).color(style::COLOR_ACCENT))
                        .style(style::badge_style)
                        .padding([2, 8]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text("Type a title and body to execute an `INSERT INTO notes` prepared statement.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                title_input,
                content_input,
                row![save_btn, refresh_btn]
                    .spacing(12)
                    .align_y(Alignment::Center),
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 2: Stored Notes List (SELECT Query Results)
        // ======================================================================
        let count = self.cached_notes.len();
        let list_header = text(format!("Stored Records in Database ({} total):", count)).size(16);

        let notes_view: Element<'_, Message> = if self.cached_notes.is_empty() {
            container(
                text("Database is currently empty. Add your first note above!")
                    .size(14)
                    .color(style::COLOR_TEXT_DIM),
            )
            .padding(20)
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list_col = column![].spacing(10);

            for note in &self.cached_notes {
                let note_id = note.id;

                let delete_btn = button(
                    row![text("🗑️").size(14), text("Delete").size(12),]
                        .spacing(6)
                        .align_y(Alignment::Center),
                )
                .style(style::danger_button_style)
                .on_press(Message::DeleteNoteClicked(note_id))
                .padding([4, 10]);

                let note_card = container(
                    row![
                        column![
                            row![
                                text(format!("#{}", note.id))
                                    .size(12)
                                    .color(style::COLOR_TEXT_DIM),
                                text(&note.title).size(15).color(style::COLOR_ACCENT),
                                text(format!("({})", note.created_at))
                                    .size(11)
                                    .color(style::COLOR_TEXT_DIM),
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                            text(&note.content).size(13).color(style::COLOR_TEXT_MUTED),
                        ]
                        .spacing(4)
                        .width(Length::Fill),
                        delete_btn,
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding([10, 14])
                .style(style::card_style)
                .width(Length::Fill);

                list_col = list_col.push(note_card);
            }

            list_col.into()
        };

        let list_card =
            container(column![list_header, rule::horizontal(1), notes_view,].spacing(12))
                .padding(20)
                .style(style::card_style)
                .width(Length::Fill);

        // ======================================================================
        // Section 3: Rust Concept Explainer Box
        // ======================================================================
        let concept_box = container(
            column![
                text("💡 How Wazoo-RS Uses Embedded SQLite").size(16).color(style::COLOR_ACCENT),
                text("• Zero Config: No need to install or run MySQL/PostgreSQL; SQLite runs inside the compiled process!").size(13),
                text("• Strongly Typed Queries: `rusqlite` maps SQLite columns directly to Rust structs (`NoteRecord`), catching type mismatches at runtime safely.").size(13),
                text("• ACID Guarantees: Changes written to SQLite are atomic, consistent, isolated, and durable, ensuring settings and bookmarks survive app crashes.").size(13),
            ]
            .spacing(8),
        )
        .padding(16)
        .style(style::highlight_box_style)
        .width(Length::Fill);

        let content = column![form_card, list_card, concept_box,]
            .spacing(20)
            .padding(20);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
