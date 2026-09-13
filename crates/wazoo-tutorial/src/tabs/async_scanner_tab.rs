/*!
 * ==============================================================================
 * TUTORIAL LESSON: Async Background Tasks, `rfd`, & `walkdir`
 * ==============================================================================
 * 
 * In desktop media applications like `wazoo-rs`, file operations (like scanning
 * a folder with thousands of videos) can take several seconds or minutes.
 * 
 * IF YOU RUN THIS ON THE MAIN THREAD:
 * The GUI will freeze completely, the window will become unresponsive, and the OS
 * might display "Application Not Responding".
 * 
 * THE ICED SOLUTION: `Task::perform`
 * Iced integrates seamlessly with the `tokio` async runtime.
 * You can spawn a background `async` future using `Task::perform(future, MessageMapper)`.
 * The future runs asynchronously on a background worker thread. When it finishes,
 * Iced automatically dispatches the mapped `Message` back into the main `update()` loop!
 * 
 * In this module, you will learn:
 * 1. Using `rfd::AsyncFileDialog` to show native OS folder pickers.
 * 2. Using `walkdir` to recursively search directories.
 * 3. Rust's `async`/`await` syntax and thread-safety bounds (`Send + 'static`).
 * 4. Filtering and displaying scanned items in an interactive UI list.
 */

use iced::{
    widget::{button, column, container, row, rule, scrollable, text, text_input},
    Alignment, Element, Length,
};

use crate::app::TutorialApp;
use crate::message::{Message, ScannedItem};
use crate::style;

impl TutorialApp {
    /// Renders Tab 2: Async Media Scanner & Native File Dialogs.
    pub fn view_async_scanner_tab(&self) -> Element<'_, Message> {
        // ======================================================================
        // Section 1: Folder Selection Header & Controls
        // ======================================================================
        let folder_label = match &self.selected_folder {
            Some(path) => text(format!("Selected Folder: {}", path))
                .size(14)
                .color(style::COLOR_ACCENT),
            None => text("No folder selected yet. Click 'Pick Folder' to choose one.")
                .size(14)
                .color(style::COLOR_TEXT_MUTED),
        };

        let pick_btn = button(
            row![
                text("📂").size(16),
                text("Pick Folder (rfd)").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(style::secondary_button_style)
        .on_press(Message::PickFolderClicked)
        .padding([8, 16]);

        let can_scan = self.selected_folder.is_some() && !self.is_scanning;
        let mut scan_btn = button(
            row![
                text(if self.is_scanning { "⏳" } else { "▶️" }).size(16),
                text(if self.is_scanning { "Scanning..." } else { "Start Scan" }).size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(style::primary_button_style)
        .padding([8, 20]);

        if can_scan {
            scan_btn = scan_btn.on_press(Message::StartScanClicked);
        }

        let controls_card = container(
            column![
                row![
                    text("Async Filesystem Scanner").size(20),
                    container(
                        text(if self.is_scanning { "RUNNING ASYNC TASK" } else { "READY" })
                            .size(11)
                            .color(if self.is_scanning { style::COLOR_DANGER } else { style::COLOR_ACCENT })
                    )
                    .style(style::badge_style)
                    .padding([2, 8]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text("Demonstrates `rfd::AsyncFileDialog` + `walkdir` recursive scanning on a background Tokio thread.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                folder_label,
                row![pick_btn, scan_btn]
                    .spacing(12)
                    .align_y(Alignment::Center),
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 2: Results Display & Search Filter
        // ======================================================================
        let filter_input = text_input(
            "Filter scanned results by name or extension...",
            &self.filter_query,
        )
        .on_input(Message::FilterQueryChanged)
        .padding(8)
        .size(14);

        // Filter the scanned items based on `self.filter_query`
        let query_lower = self.filter_query.to_lowercase();
        let filtered_items: Vec<&ScannedItem> = self
            .scanned_items
            .iter()
            .filter(|item| {
                if query_lower.is_empty() {
                    true
                } else {
                    item.filename.to_lowercase().contains(&query_lower)
                        || item.path.to_string_lossy().to_lowercase().contains(&query_lower)
                }
            })
            .collect();

        let count_text = format!(
            "Found {} total files ({} media videos) — Showing {} matching filter",
            self.scanned_items.len(),
            self.scanned_items.iter().filter(|i| i.is_video).count(),
            filtered_items.len()
        );

        let results_header = row![
            text(count_text).size(13).color(style::COLOR_TEXT_MUTED),
        ];

        let results_list: Element<'_, Message> = if self.scanned_items.is_empty() {
            container(
                text(if self.is_scanning {
                    "Scanning in progress on background worker thread... Please wait."
                } else {
                    "No files scanned yet. Pick a folder and click 'Start Scan'."
                })
                .size(14)
                .color(style::COLOR_TEXT_DIM),
            )
            .padding(20)
            .center_x(Length::Fill)
            .into()
        } else if filtered_items.is_empty() {
            container(
                text("No scanned items match the current filter query.")
                    .size(14)
                    .color(style::COLOR_TEXT_DIM),
            )
            .padding(20)
            .center_x(Length::Fill)
            .into()
        } else {
            let mut list_col = column![].spacing(6);

            for item in filtered_items.iter().take(50) {
                let icon = if item.is_video { "🎬" } else { "📄" };
                let size_mb = (item.size_bytes as f64) / (1024.0 * 1024.0);

                let item_row = container(
                    row![
                        text(icon).size(18),
                        column![
                            text(&item.filename)
                                .size(14)
                                .color(if item.is_video { style::COLOR_ACCENT } else { iced::Color::WHITE }),
                            text(item.path.to_string_lossy().to_string())
                                .size(11)
                                .color(style::COLOR_TEXT_DIM),
                        ]
                        .spacing(2)
                        .width(Length::Fill),
                        text(format!("{:.2} MB", size_mb))
                            .size(12)
                            .color(style::COLOR_TEXT_MUTED),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding([8, 12])
                .style(style::card_style)
                .width(Length::Fill);

                list_col = list_col.push(item_row);
            }

            if filtered_items.len() > 50 {
                list_col = list_col.push(
                    container(
                        text(format!("... and {} more items", filtered_items.len() - 50))
                            .size(12)
                            .color(style::COLOR_TEXT_MUTED),
                    )
                    .padding(8)
                    .center_x(Length::Fill),
                );
            }

            list_col.into()
        };

        let results_card = container(
            column![
                text("Scan Results").size(18),
                filter_input,
                results_header,
                rule::horizontal(1),
                results_list,
            ]
            .spacing(12),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 3: Rust Concept Explainer Box
        // ======================================================================
        let concept_box = container(
            column![
                text("💡 How Async Tasks Work in Wazoo & Iced").size(16).color(style::COLOR_ACCENT),
                text("• `Task::perform(future, Message::ScanFinished)`: Takes an async block and runs it on Tokio.").size(13),
                text("• Thread Safety: Rust's type system requires the future to be `Send + 'static`. That means data passed into the async closure must be owned (e.g. `String`, not `&str`).").size(13),
                text("• Zero UI Freezing: While thousands of files are being walked on the background thread, your 60 FPS GUI remains completely interactive!").size(13),
            ]
            .spacing(8),
        )
        .padding(16)
        .style(style::highlight_box_style)
        .width(Length::Fill);

        let content = column![
            controls_card,
            results_card,
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
