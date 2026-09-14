/*!
 * ==============================================================================
 * TUTORIAL LESSON: Elm Architecture & Basic Iced Widgets
 * ==============================================================================
 *
 * This tab demonstrates the most essential building blocks of Iced:
 * 1. Buttons & Event Dispatching (`button`, `.on_press`)
 * 2. Text Input & Two-way Binding Simulation (`text_input`, `.on_input`)
 * 3. Numerical Sliders (`slider`)
 * 4. Checkboxes & Booleans (`checkbox`, `.on_toggle`)
 * 5. Layout Containers (`column!`, `row!`, `container`)
 *
 * RUST CONCEPT: Pure Declarative Views
 * In Iced, your `view` function does NOT mutate the UI directly.
 * There is no `document.getElementById` or `my_button.setText("...")`.
 * Instead, `view(&self)` is called whenever state changes, returning a lightweight
 * tree of `Element<'_, Message>`. Iced handles rendering this tree to the screen.
 */

use iced::{
    Alignment, Element, Length,
    widget::{
        button, checkbox, column, container, row, rule, scrollable, slider, text, text_input,
    },
};

use crate::app::TutorialApp;
use crate::message::Message;
use crate::style;

impl TutorialApp {
    /// Renders Tab 1: Counter & Interactive Widgets.
    ///
    /// RUST CONCEPT: Lifetime Elision in `Element<'_, Message>`
    /// Notice the return type: `Element<'_, Message>`.
    /// The `'_` is an anonymous lifetime. It tells the Rust compiler:
    /// "This Element contains references that borrow data from `&self` for at least
    /// as long as `self` is alive."
    pub fn view_counter_tab(&self) -> Element<'_, Message> {
        // ======================================================================
        // Section 1: The Classic Counter
        // ======================================================================
        let counter_display = text(format!("{}", self.counter_value))
            .size(42)
            .style(|_theme| text::Style {
                color: Some(style::COLOR_ACCENT),
            });

        // A horizontal row of buttons: [-], [Reset], [+]
        let counter_controls = row![
            button(text(" - ").size(20))
                .style(style::secondary_button_style)
                .on_press(Message::DecrementCounter)
                .padding([8, 20]),
            button(text("Reset").size(16))
                .style(style::secondary_button_style)
                .on_press(Message::ResetCounter)
                .padding([8, 16]),
            button(text(" + ").size(20))
                .style(style::primary_button_style)
                .on_press(Message::IncrementCounter)
                .padding([8, 20]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let counter_card = container(
            column![
                row![
                    text("Counter Demo").size(20),
                    container(text("TEA State").size(12).color(style::COLOR_ACCENT))
                        .style(style::badge_style)
                        .padding([2, 8]),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                text("Clicking buttons dispatches messages that mutate `self.counter_value` in `update()`.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                column![counter_display, counter_controls]
                    .spacing(16)
                    .align_x(Alignment::Center),
            ]
            .spacing(14),
        )
        .padding(20)
        .style(style::card_style)
        .width(Length::Fill);

        // ======================================================================
        // Section 2: Text Input & Form Controls
        // ======================================================================
        // Notice `text_input`:
        // - First parameter is placeholder text: "Type something..."
        // - Second parameter is a borrowed reference to state: `&self.text_input_value`
        // - `.on_input(Message::TextInputChanged)` turns keyboard keystrokes into Messages
        let input_box = text_input(
            "Type something to test two-way state...",
            &self.text_input_value,
        )
        .on_input(Message::TextInputChanged)
        .padding(10)
        .size(15);

        let echo_display = if self.text_input_value.is_empty() {
            text("(Waiting for input...)")
                .color(style::COLOR_TEXT_DIM)
                .size(14)
        } else {
            text(format!(
                "You typed: \"{}\" (length: {} chars)",
                self.text_input_value,
                self.text_input_value.len()
            ))
            .color(style::COLOR_ACCENT)
            .size(14)
        };

        // Numerical Slider
        let slider_widget = slider(0.0..=100.0, self.slider_value, Message::SliderChanged);
        let slider_label = text(format!("Slider Value: {:.1}%", self.slider_value))
            .size(14)
            .color(style::COLOR_TEXT_MUTED);

        // Checkbox widget (in Iced 0.14, checkbox takes a boolean `is_checked`)
        let checkbox_widget = checkbox(self.checkbox_value)
            .on_toggle(Message::CheckboxToggled)
            .size(18);

        let flag_status = if self.checkbox_value {
            text("Feature is currently: ACTIVE")
                .color(style::COLOR_ACCENT)
                .size(13)
        } else {
            text("Feature is currently: INACTIVE")
                .color(style::COLOR_TEXT_MUTED)
                .size(13)
        };

        let form_card = container(
            column![
                text("Form Controls & Reactive Inputs").size(20),
                text("Notice how widgets receive current state and produce Messages upon user action.")
                    .size(14)
                    .color(style::COLOR_TEXT_MUTED),
                rule::horizontal(1),
                text("1. Text Input:").size(15),
                input_box,
                echo_display,
                rule::horizontal(1),
                text("2. Numerical Slider:").size(15),
                slider_widget,
                slider_label,
                rule::horizontal(1),
                text("3. Checkbox:").size(15),
                row![
                    checkbox_widget,
                    text("Toggle Feature Flag (Boolean State)").size(14),
                    flag_status
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
        // Section 3: Rust Concept Explainer Box
        // ======================================================================
        let concept_box = container(
            column![
                text("💡 How This Works in Rust & Iced").size(16).color(style::COLOR_ACCENT),
                text("• Pure Functions: The `view` function has signature `fn view(&self) -> Element`. It cannot modify `self`!").size(13),
                text("• State Mutation: When a user clicks '+', Iced passes `Message::IncrementCounter` to `fn update(&mut self, message)`. Only `update` can modify data.").size(13),
                text("• Ownership & Borrowing: The text input borrows `&self.text_input_value`. No unnecessary heap copies are created during rendering!").size(13),
            ]
            .spacing(8),
        )
        .padding(16)
        .style(style::highlight_box_style)
        .width(Length::Fill);

        // Combine everything into a scrollable column
        let content = column![counter_card, form_card, concept_box,]
            .spacing(20)
            .padding(20);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
