/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Media Server Integration Reducer
 */

use crate::app::{DEFAULT_TOAST_SECS, LONG_TOAST_SECS, WazooApp};
use crate::message::Message;
use iced::Task;
use iced::futures::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use wazoo_core::{Database, PlexClient};

impl WazooApp {
    /// Handles messages for Plex server configuration, connectivity testing,
    /// and asynchronous metadata synchronization.
    pub(crate) fn update_plex(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetPlexEnabled(enabled) => {
                self.settings.plex.enabled = enabled;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }

            Message::PlexServerUrlChanged(url) => {
                self.settings.plex.server_url = url;
                self.plex.test_result = None;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }

            Message::PlexTokenChanged(token) => {
                self.settings.plex.token = token;
                self.plex.test_result = None;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }

            Message::SetPlexAutoSync(auto_sync) => {
                self.settings.plex.auto_sync_on_scan = auto_sync;
                let _ = self.config_mgr.save_settings(&self.settings);
                Task::none()
            }

            Message::TestPlexConnection => {
                self.plex.is_testing = true;
                self.plex.test_result = None;

                let server_url = self.settings.plex.server_url.clone();
                let token = self.settings.plex.token.clone();

                Task::perform(
                    async move {
                        let client = PlexClient::new(server_url, token);
                        client.test_connection().await
                    },
                    Message::PlexConnectionTested,
                )
            }

            Message::PlexConnectionTested(res) => {
                self.plex.is_testing = false;
                self.plex.test_result = Some(res);
                Task::none()
            }

            Message::StartPlexSync => {
                if self.plex.is_syncing {
                    return Task::none();
                }

                self.plex.is_syncing = true;
                self.plex.sync_progress = None;
                self.plex.sync_result = None;
                self.plex.current_sync_id += 1;

                let server_url = self.settings.plex.server_url.clone();
                let token = self.settings.plex.token.clone();
                let db_path = self.config_mgr.database_path();

                Task::stream(iced::stream::channel(
                    100,
                    move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

                        let sync_handle = tokio::spawn(async move {
                            let db = match Database::open(&db_path) {
                                Ok(d) => Arc::new(Mutex::new(d)),
                                Err(e) => return Err(format!("Could not open database: {e}")),
                            };

                            let client = PlexClient::new(server_url, token);
                            wazoo_core::sync_plex_library(&client, db, Some(tx)).await
                        });

                        while let Some(progress) = rx.recv().await {
                            let _ = output.send(Message::PlexSyncProgressUpdate(progress)).await;
                        }

                        let res = match sync_handle.await {
                            Ok(inner_res) => inner_res,
                            Err(join_err) => Err(join_err.to_string()),
                        };

                        let _ = output.send(Message::PlexSyncFinished(res)).await;
                    },
                ))
            }

            Message::PlexSyncProgressUpdate(progress) => {
                self.plex.sync_progress = Some(progress);
                Task::none()
            }

            Message::PlexSyncFinished(res) => {
                self.plex.is_syncing = false;
                match &res {
                    Ok(summary) => {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        self.settings.plex.last_synced_at = Some(now);
                        let _ = self.config_mgr.save_settings(&self.settings);

                        // Reload in-memory metadata cache from SQLite
                        self.metadata_cache = self.db.get_all_metadata_map().unwrap_or_default();

                        let matched_str = summary.matched_and_updated.to_string();
                        self.overlay.toast_message = Some(format!(
                            "Plex sync complete: {} items matched with metadata",
                            matched_str
                        ));
                        self.overlay.toast_time_remaining = LONG_TOAST_SECS;
                    }
                    Err(err) => {
                        self.overlay.toast_message = Some(format!("Plex sync failed: {err}"));
                        self.overlay.toast_time_remaining = DEFAULT_TOAST_SECS;
                    }
                }
                self.plex.sync_result = Some(res);
                Task::none()
            }

            _ => Task::none(),
        }
    }
}

