/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Root View Assembly
 * 
 * Assembles the primary UI layer stack, rendering video players, transient overlays,
 * sliding titlebars, floating toast notices, scan banners, and modal dialogs.
 */

pub mod file_picker;
pub mod modals;
pub mod player;
pub mod titlebar;
pub mod transcript;
pub mod welcome;

use iced::{
    widget::{button, column, container, mouse_area, row, text, Space, Stack},
    Alignment, Element, Length, Theme,
};
use wazoo_scanner::ScanStage;
use crate::app::WazooApp;
use crate::format;
use crate::message::Message;
use crate::theme;

impl WazooApp {
    pub fn view(&self) -> Element<'_, Message> {
        let main_content: Element<'_, Message> = if self.available_videos.is_empty() {
            self.view_welcome()
        } else if self.show_file_picker {
            row![
                container(self.view_players()).width(Length::Fill).height(Length::Fill),
                self.view_file_picker(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else if self.show_transcript {
            row![
                container(self.view_players()).width(Length::Fill).height(Length::Fill),
                self.view_transcript_drawer(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            row![container(self.view_players()).width(Length::Fill).height(Length::Fill)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        // If any modal is active, display only main content in the background and the modal layer.
        // This guarantees a stable 2-element stack where the modal is always child index 1.
        // Intermediate transient layers (toast, titlebar, scan banner) are never inserted/removed
        // while a modal is open, preventing iced::widget::Stack index shifts that destroy focus.
        let modal: Option<Element<'_, Message>> = if self.show_search_modal {
            Some(self.view_search_modal())
        } else if self.show_settings_modal {
            Some(self.view_settings_modal())
        } else if self.show_help_modal {
            Some(self.view_help_modal())
        } else if self.show_bookmarks_modal {
            Some(self.view_bookmarks_modal())
        } else if self.show_menu_modal {
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
            if self.show_dropdown_menu {
                root_stack_children.push(Element::from(
                    mouse_area(container(Space::new()).width(Length::Fill).height(Length::Fill))
                        .on_press(Message::CloseDropdownMenu),
                ));
            }

            // 2. Sliding Titlebar & Dropdown Menu Overlay
            if self.show_titlebar {
                root_stack_children.push(self.view_titlebar());
            }

            // 3. Floating Notice (Matches Electron Notice.vue)
            if let Some(ref toast) = self.toast_message {
                let toast_widget = container(
                    row![
                        text(toast).size(16).color(iced::Color::WHITE),
                        button(text("✕").size(12))
                            .style(theme::window_control_button_style)
                            .on_press(Message::DismissToast)
                            .padding(2),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding([6, 18])
                .style(theme::notice_pill_style);

                let toast_layer = container(
                    column![
                        Space::new().height(Length::Fixed(35.0)),
                        toast_widget,
                    ]
                    .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill);

                root_stack_children.push(Element::from(toast_layer));
            }

            // 4. Scan Toast Banner (Matches Electron scan toast across top)
            if self.is_scanning {
                let (label, status_text) = if let Some(ref progress) = self.scan_progress {
                    match progress.stage {
                        ScanStage::Listing => {
                            let name = if progress.current_name.is_empty() {
                                "Discovering files...".to_string()
                            } else {
                                format!("Listing: {}", progress.current_name)
                            };
                            let stat = format!("{}% ({} found)", progress.percent, format::format_number(progress.files_found));
                            (name, stat)
                        }
                        ScanStage::Indexing => {
                            let name = if progress.current_name.is_empty() {
                                "Indexing library...".to_string()
                            } else {
                                format!("Indexing: {}", progress.current_name)
                            };
                            let stat = format!("{}% ({} files)", progress.percent, format::format_number(progress.total));
                            (name, stat)
                        }
                    }
                } else {
                    ("Scanning media folders...".to_string(), "In progress".to_string())
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
            if self.is_alt_pressed {
                let alt_overlay = container(
                    column![
                        text(self.t("app.drag_to_move")).size(22).color(iced::Color::WHITE),
                        text(self.t("app.x_to_quit")).size(16).color(theme::COLOR_TEXT_DIM),
                    ]
                    .spacing(8)
                    .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                    ..Default::default()
                });

                root_stack_children.push(Element::from(mouse_area(alt_overlay).on_press(Message::DragWindow)));
            }

            Stack::with_children(root_stack_children)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        crate::cursor::WindowBorderResizer::new(root, Message::DragResize).into()
    }
}
