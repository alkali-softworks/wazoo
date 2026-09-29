/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Modal Dialogs Subsystem
 *
 * Modular modal components: search, settings, keyboard shortcut help,
 * bookmarks, context menu, and shared dismissable backdrop wrapper.
 */

pub mod bookmarks_modal;
pub mod help_modal;
pub mod menu_modal;
pub mod search_modal;
pub mod settings;

use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, mouse_area, row},
};

impl WazooApp {
    pub(crate) fn wrap_modal_with_backdrop<'a>(
        card: container::Container<'a, Message>,
        on_close: Message,
    ) -> Element<'a, Message> {
        let backdrop_top = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_bottom = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .on_press(on_close.clone());

        let backdrop_left = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close.clone());

        let backdrop_right = mouse_area(
            container(Space::new())
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        )
        .on_press(on_close);

        let card_area = mouse_area(card).on_press(Message::ModalCardClicked);

        let center_row = row![backdrop_left, card_area, backdrop_right,]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Shrink);

        let modal_layout = column![backdrop_top, center_row, backdrop_bottom,]
            .width(Length::Fill)
            .height(Length::Fill);

        container(modal_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::modal_backdrop_style)
            .into()
    }
}
