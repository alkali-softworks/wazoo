/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Menu Modal Component
 *
 * Compact context menu / quick actions modal.
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::theme;
use iced::{Element, Length, widget::container};

impl WazooApp {
    pub fn view_menu_modal(&self) -> Element<'_, Message> {
        let card = container(self.view_app_menu_list([8, 12]).spacing(4))
            .width(Length::Fixed(260.0))
            .padding(10)
            .style(theme::modal_card_style);

        Self::wrap_modal_with_backdrop(card, Message::CloseMenuModal)
    }
}
