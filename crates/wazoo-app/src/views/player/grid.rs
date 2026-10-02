/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Multi-Player Grid & Scroll Stream Layouts
 */

use crate::app::WazooApp;
use crate::message::Message;
use crate::scroll_view;
use crate::state::AppPlayer;
use crate::theme;
use iced::{
    Element, Length,
    widget::{column, container, row, text},
};
use wazoo_core::{LayoutMode, PlaybackMode};

impl WazooApp {
    pub(crate) fn view_players(&self) -> Element<'_, Message> {
        let grid_players: Vec<&AppPlayer> = self.players.iter().filter(|p| !p.is_cube).collect();
        if grid_players.is_empty() {
            return container(
                text(self.t("wazoo.no_players"))
                    .size(18)
                    .color(theme::COLOR_TEXT_MUTED),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        }

        if self.settings.playback_mode == PlaybackMode::Scroll {
            return self.view_scroll_stream();
        }

        match self.settings.layout {
            LayoutMode::Row => {
                let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                for p in &grid_players {
                    r = r.push(self.view_single_player(p));
                }
                r.into()
            }
            LayoutMode::Column => {
                let mut c = column![]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill);
                for p in &grid_players {
                    c = c.push(self.view_single_player(p));
                }
                c.into()
            }
            LayoutMode::Grid => {
                let count = grid_players.len();
                if count == 1 {
                    self.view_single_player(grid_players[0])
                } else if count == 2 {
                    row![
                        self.view_single_player(grid_players[0]),
                        self.view_single_player(grid_players[1]),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else if count == 3 {
                    // Electron 3-player special layout: top player spans full width, bottom row has 2
                    column![
                        container(self.view_single_player(grid_players[0]))
                            .width(Length::Fill)
                            .height(Length::FillPortion(1)),
                        row![
                            self.view_single_player(grid_players[1]),
                            self.view_single_player(grid_players[2]),
                        ]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::FillPortion(1)),
                    ]
                    .spacing(0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
                } else {
                    let cols = if count <= 4 {
                        2
                    } else if count <= 9 {
                        3
                    } else {
                        4
                    };
                    let mut rows = column![]
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::Fill);
                    for chunk in grid_players.chunks(cols) {
                        let mut r = row![].spacing(0).width(Length::Fill).height(Length::Fill);
                        for p in chunk {
                            r = r.push(self.view_single_player(p));
                        }
                        rows = rows.push(r);
                    }
                    rows.into()
                }
            }
        }
    }

    pub(crate) fn view_scroll_stream(&self) -> Element<'_, Message> {
        let mut stream = scroll_view::ScrollStream::new();
        let mut scroll_items: Vec<(&AppPlayer, &wazoo_media::ScrollItem)> = self
            .players
            .iter()
            .filter_map(|p| self.scroll_engine.items.get(&p.id).map(|item| (p, item)))
            .collect();
        scroll_items.sort_by(|a, b| {
            a.1.y_pos
                .partial_cmp(&b.1.y_pos)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (p, item) in scroll_items {
            stream = stream.push(self.view_scroll_player(p), item.y_pos, item.height);
        }

        stream.into()
    }
}
