/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Explorer & Metadata Lookup
 *
 * Handles Plex metadata lookups, opening the Plex Explorer modal, live API fetching
 * for On Deck / Recently Added tabs, and SQLite/text filtering across items.
 */

use super::WazooApp;
use crate::message::Message;
use iced::Task;

impl WazooApp {
    /// Retrieves cached Plex metadata for a given video file path.
    pub fn get_metadata_for_path(&self, path: &str) -> Option<&wazoo_core::MediaMetadata> {
        let norm_target = path.replace('\\', "/");
        self.available_videos
            .iter()
            .find(|v| v.path == path || v.path.replace('\\', "/") == norm_target)
            .and_then(|v| self.metadata_cache.get(&v.id))
    }

    pub(crate) fn open_plex_explorer(&mut self) -> Task<Message> {
        let exit_task = if self.cube.desktop_overlay {
            self.exit_desktop_cube_overlay()
        } else {
            Task::none()
        };
        self.modals.close_all();
        self.drawers.close_all();
        self.titlebar.show_dropdown_menu = false;
        self.titlebar.show = false;
        self.overlay.clear_toast();
        self.hovered_player_id = None;
        self.overlay.ticks = 0;
        self.overlay.fade_in_ticks = 0;
        self.modals.plex_explorer = true;
        self.modals.plex_explorer_search.clear();
        self.modals.plex_explorer_genre = None;
        self.modals.plex_explorer_decade = None;
        self.modals.plex_explorer_random_seed = rand::random::<u64>();

        let tab = self.modals.plex_explorer_tab.clone();
        self.refresh_plex_explorer_items();
        let fetch_task = self.fetch_plex_live_items(tab);

        Task::batch([
            exit_task,
            fetch_task,
            iced::widget::operation::focus("plex_explorer_search_input"),
        ])
    }

    pub(crate) fn fetch_plex_live_items(
        &self,
        tab: wazoo_core::PlexExplorerTab,
    ) -> Task<Message> {
        if !self.settings.plex.enabled || self.settings.plex.token.trim().is_empty() {
            return Task::none();
        }

        match tab {
            wazoo_core::PlexExplorerTab::OnDeck => {
                let client = wazoo_core::PlexClient::new(
                    &self.settings.plex.server_url,
                    &self.settings.plex.token,
                );

                Task::perform(
                    async move { client.fetch_on_deck().await },
                    |res| match res {
                        Ok(items) => Message::PlexExplorerLiveLoaded(
                            wazoo_core::PlexExplorerTab::OnDeck,
                            items,
                        ),
                        Err(_) => Message::DismissToast,
                    },
                )
            }
            wazoo_core::PlexExplorerTab::RecentlyAdded => {
                let client = wazoo_core::PlexClient::new(
                    &self.settings.plex.server_url,
                    &self.settings.plex.token,
                );

                Task::perform(
                    async move { client.fetch_recently_added(60).await },
                    |res| match res {
                        Ok(items) => Message::PlexExplorerLiveLoaded(
                            wazoo_core::PlexExplorerTab::RecentlyAdded,
                            items,
                        ),
                        Err(_) => Message::DismissToast,
                    },
                )
            }
            _ => Task::none(),
        }
    }

    pub fn refresh_plex_explorer_items(&mut self) {
        let query = self.modals.plex_explorer_search.trim().to_lowercase();
        let query_words: Vec<String> = if query.is_empty() {
            Vec::new()
        } else {
            let clean = query.replace(['_', '-'], " ");
            clean.split_whitespace().map(|s| s.to_string()).collect()
        };

        let items = match &self.modals.plex_explorer_tab {
            wazoo_core::PlexExplorerTab::OnDeck => {
                let items = self
                    .plex
                    .live_items
                    .get(&wazoo_core::PlexExplorerTab::OnDeck)
                    .cloned()
                    .unwrap_or_default();
                if query_words.is_empty() {
                    items
                } else {
                    items
                        .into_iter()
                        .filter(|item| Self::plex_explorer_item_matches_words(item, &query_words))
                        .collect()
                }
            }
            wazoo_core::PlexExplorerTab::RecentlyAdded => {
                if let Some(items) = self
                    .plex
                    .live_items
                    .get(&wazoo_core::PlexExplorerTab::RecentlyAdded)
                {
                    if query_words.is_empty() {
                        items.clone()
                    } else {
                        items
                            .iter()
                            .filter(|item| Self::plex_explorer_item_matches_words(item, &query_words))
                            .cloned()
                            .collect()
                    }
                } else {
                    self.db
                        .query_plex_explorer(
                            &self.modals.plex_explorer_tab,
                            self.modals.plex_explorer_decade,
                            self.modals.plex_explorer_genre.as_deref(),
                            &self.modals.plex_explorer_search,
                        )
                        .unwrap_or_default()
                }
            }
            wazoo_core::PlexExplorerTab::Random => {
                let mut list = self
                    .db
                    .query_plex_explorer(
                        &self.modals.plex_explorer_tab,
                        self.modals.plex_explorer_decade,
                        self.modals.plex_explorer_genre.as_deref(),
                        &self.modals.plex_explorer_search,
                    )
                    .unwrap_or_default();
                use rand::SeedableRng;
                use rand::seq::SliceRandom;
                let seed = if self.modals.plex_explorer_random_seed == 0 {
                    1337
                } else {
                    self.modals.plex_explorer_random_seed
                };
                let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                list.shuffle(&mut rng);
                list.truncate(150);
                list
            }
            _ => self
                .db
                .query_plex_explorer(
                    &self.modals.plex_explorer_tab,
                    self.modals.plex_explorer_decade,
                    self.modals.plex_explorer_genre.as_deref(),
                    &self.modals.plex_explorer_search,
                )
                .unwrap_or_default(),
        };

        self.plex.explorer_items = items;
    }

    pub fn plex_explorer_items(&self) -> Vec<wazoo_core::PlexExplorerItem> {
        self.plex.explorer_items.clone()
    }

    fn plex_explorer_item_matches_words(
        item: &wazoo_core::PlexExplorerItem,
        words: &[String],
    ) -> bool {
        let title = item.title.to_lowercase();
        let subtitle = item.subtitle.as_deref().unwrap_or("").to_lowercase();
        let summary = item.summary.as_deref().unwrap_or("").to_lowercase();
        let directors = item.directors.join(" ").to_lowercase();
        let actors = item.actors.join(" ").to_lowercase();
        let mut genres = item.genres.join(" ").to_lowercase();
        if genres.contains("science fiction") || genres.contains("sci-fi") {
            genres.push_str(" scifi sci-fi science fiction");
        }
        if genres.contains("anime") || genres.contains("animation") {
            genres.push_str(" anime animation");
        }
        let year_str = match item.year {
            Some(y) => {
                let dec_full = format!("{}s", (y / 10) * 10);
                let dec_short = format!("{:02}s", ((y / 10) * 10) % 100);
                format!("{} {} {}", y, dec_full, dec_short)
            }
            None => String::new(),
        };

        words.iter().all(|w| {
            title.contains(w)
                || subtitle.contains(w)
                || summary.contains(w)
                || directors.contains(w)
                || actors.contains(w)
                || genres.contains(w)
                || year_str.contains(w)
        })
    }
}
