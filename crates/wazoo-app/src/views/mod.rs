/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Root View Assembly
 *
 * Assembles the primary UI layer stack, rendering video players, transient overlays,
 * sliding titlebars, floating toast notices, scan banners, and modal dialogs.
 */

pub mod cube;
pub mod file_picker;
pub mod history;
pub mod modals;
pub mod player;
pub mod titlebar;
pub mod transcript;
pub mod welcome;

use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length, Theme, mouse,
    widget::{Space, Stack, column, container, mouse_area, row, text},
};
use wazoo_scanner::ScanStage;

impl WazooApp {
    pub fn view(&self) -> Element<'_, Message> {
        // Cube Overlay Mode: hide all normal chrome and UI,
        // displaying only the 3D bouncing video cube directly over the transparent desktop
        if self.cube.desktop_overlay {
            let mut overlay_stack: Vec<Element<'_, Message>> = vec![
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
            ];
            if self.cube.is_present() {
                overlay_stack.push(self.view_cube_overlay());
            }
            if let Some(ref toast) = self.overlay.toast_message {
                let toast_widget = container(
                    text(toast).size(16).color(iced::Color::WHITE),
                )
                .padding([6, 18])
                .style(theme::notice_pill_style);

                let toast_layer = container(
                    column![Space::new().height(Length::Fixed(35.0)), toast_widget,]
                        .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill);

                overlay_stack.push(Element::from(toast_layer));
            }
            return Stack::with_children(overlay_stack)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let main_content: Element<'_, Message> = if self.available_videos.is_empty() {
            let total_library_videos = self.db.get_video_count().unwrap_or(0);
            let is_query_active = !self.search.active_query.trim().is_empty()
                || !self.search.active_folders.is_empty()
                || (!self.search.active_folder.is_empty()
                    && !self.is_all_folder(&self.search.active_folder));

            if total_library_videos > 0 || is_query_active {
                self.view_no_matches()
            } else {
                self.view_welcome()
            }
        } else if self.drawers.show_file_picker {
            row![
                container(self.view_players())
                    .width(Length::Fill)
                    .height(Length::Fill),
                self.view_file_picker(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.drawers.show_transcript {
            row![
                container(self.view_players())
                    .width(Length::Fill)
                    .height(Length::Fill),
                self.view_transcript_drawer(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.drawers.show_history_drawer {
            row![
                container(self.view_players())
                    .width(Length::Fill)
                    .height(Length::Fill),
                self.view_history_drawer(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            row![
                container(self.view_players())
                    .width(Length::Fill)
                    .height(Length::Fill)
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        // If any modal is active, display only main content in the background and the modal layer.
        // This guarantees a stable 2-element stack where the modal is always child index 1.
        // Intermediate transient layers (toast, titlebar, scan banner) are never inserted/removed
        // while a modal is open, preventing iced::widget::Stack index shifts that destroy focus.
        let modal: Option<Element<'_, Message>> = if self.modals.search {
            Some(self.view_search_modal())
        } else if self.modals.settings {
            Some(self.view_settings_modal())
        } else if self.modals.help {
            Some(self.view_help_modal())
        } else if self.modals.bookmarks {
            Some(self.view_bookmarks_modal())
        } else if self.modals.menu {
            Some(self.view_menu_modal())
        } else {
            None
        };

        let root: Element<'_, Message> = if let Some(modal_el) = modal {
            Stack::with_children(vec![main_content, modal_el])
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            let mut root_stack_children: Vec<Element<'_, Message>> = vec![main_content];

            // 1.5. Dropdown backdrop for dismissal when clicking outside
            if self.titlebar.show_dropdown_menu {
                root_stack_children.push(Element::from(
                    mouse_area(
                        container(Space::new())
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .interaction(mouse::Interaction::Idle)
                    .on_press(Message::CloseDropdownMenu),
                ));
            }

            // 2. Sliding Titlebar & Dropdown Menu Overlay (suppressed while resizing borders)
            if !self.window.is_resizing() && self.titlebar.show {
                root_stack_children.push(self.view_titlebar());
            }

            // 3. Floating Notice (Matches Electron Notice.vue)
            if let Some(ref toast) = self.overlay.toast_message {
                let toast_widget = container(
                    text(toast).size(16).color(iced::Color::WHITE),
                )
                .padding([6, 18])
                .style(theme::notice_pill_style);

                let toast_layer = container(
                    column![Space::new().height(Length::Fixed(35.0)), toast_widget,]
                        .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill);

                root_stack_children.push(Element::from(toast_layer));
            }

            // 4. Scan Toast Banner (Matches Electron scan toast across top)
            if self.scanner.is_scanning {
                let (label, status_text) = if let Some(ref progress) = self.scanner.scan_progress {
                    match progress.stage {
                        ScanStage::Listing => {
                            let name = if progress.current_name.is_empty() {
                                "Discovering files...".to_string()
                            } else {
                                format!("Listing: {}", progress.current_name)
                            };
                            let stat = format!(
                                "{}% ({} found)",
                                progress.percent,
                                format::format_number(progress.files_found)
                            );
                            (name, stat)
                        }
                        ScanStage::Indexing => {
                            let name = if progress.current_name.is_empty() {
                                "Indexing library...".to_string()
                            } else {
                                format!("Indexing: {}", progress.current_name)
                            };
                            let stat = format!(
                                "{}% ({} files)",
                                progress.percent,
                                format::format_number(progress.total)
                            );
                            (name, stat)
                        }
                    }
                } else {
                    (
                        "Scanning media folders...".to_string(),
                        "In progress".to_string(),
                    )
                };

                let scan_banner = container(
                    row![
                        text(label).size(13).color(iced::Color::WHITE),
                        Space::new().width(Length::Fill),
                        text(status_text).size(13).color(theme::COLOR_PRIMARY),
                    ]
                    .padding([4, 24])
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fixed(28.0))
                .style(theme::scan_toast_banner_style);

                root_stack_children.push(Element::from(scan_banner));
            }

            // 5. Alt Drag Overlay (Matches Electron Alt overlay)
            if self.window.is_alt_pressed {
                let drag_card = container(
                    column![
                        text(self.t("app.drag_to_move"))
                            .size(22)
                            .color(iced::Color::WHITE),
                        text(self.t("app.x_to_quit"))
                            .size(16)
                            .color(theme::COLOR_TEXT_DIM),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .padding([20, 36])
                .style(theme::drag_overlay_card_style);

                let alt_overlay = container(drag_card)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.0, 0.0, 0.0, 0.5,
                        ))),
                        ..Default::default()
                    });

                root_stack_children.push(Element::from(
                    mouse_area(alt_overlay).on_press(Message::DragWindow),
                ));
            }

            // 6. 3D Bouncing Video Cube (Overlay on top of everything)
            if self.cube.is_present() {
                root_stack_children.push(self.view_cube_overlay());
            }

            Stack::with_children(root_stack_children)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        crate::cursor::WindowBorderResizer::new(root, Message::DragResize).into()
    }
}
