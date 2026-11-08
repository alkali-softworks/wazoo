/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Media Server REST Integration & Path Matcher
 *
 * Communicates with Plex Media Server over REST/JSON, retrieves movie and television
 * metadata, and normalizes remote NAS file paths to local mount points.
 */

use crate::db::Database;
use crate::models::MediaMetadata;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlexServerInfo {
    pub machine_identifier: String,
    pub version: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlexSection {
    pub key: String,
    pub title: String,
    pub section_type: String, // "movie", "show"
    pub locations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlexRawItem {
    pub rating_key: String,
    pub title: String,
    pub series_title: Option<String>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub year: Option<u32>,
    pub tagline: Option<String>,
    pub summary: Option<String>,
    pub content_rating: Option<String>,
    pub duration_ms: Option<u64>,
    pub rating: Option<f32>,
    pub studio: Option<String>,
    pub genres: Vec<String>,
    pub directors: Vec<String>,
    pub actors: Vec<String>,
    pub thumb_url: Option<String>,
    pub art_url: Option<String>,
    pub file_paths: Vec<String>,
    pub view_offset_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PlexSyncProgress {
    pub stage: String,
    pub processed_items: usize,
    pub total_items: usize,
    pub matched_items: usize,
    pub current_library: String,
}

#[derive(Debug, Clone, Default)]
pub struct PlexSyncSummary {
    pub libraries_scanned: usize,
    pub total_plex_items: usize,
    pub matched_and_updated: usize,
}

#[derive(Clone)]
pub struct PlexClient {
    http: reqwest::Client,
    server_url: String,
    token: String,
}

impl PlexClient {
    pub fn new(server_url: impl Into<String>, token: impl Into<String>) -> Self {
        let trimmed_url = server_url.into().trim().trim_end_matches('/').to_string();
        let trimmed_token = token.into().trim().to_string();

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            "X-Plex-Client-Identifier",
            HeaderValue::from_static("wazoo-desktop-client"),
        );
        headers.insert(
            "X-Plex-Product",
            HeaderValue::from_static("Wazoo Media Engine"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http,
            server_url: trimmed_url,
            token: trimmed_token,
        }
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    /// Verifies connectivity and token authentication against Plex Media Server.
    pub async fn test_connection(&self) -> Result<PlexServerInfo, String> {
        if self.server_url.is_empty() {
            return Err("Plex server URL is required".to_string());
        }

        let identity_url = format!("{}/identity", self.server_url);
        let resp = self
            .http
            .get(&identity_url)
            .send()
            .await
            .map_err(|e| format!("Could not connect to Plex server: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Plex identity returned HTTP {}", resp.status()));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;

        // Extract version and machine identifier from identity XML
        let version = extract_xml_attr(&body, "version")
            .unwrap_or_else(|| "Unknown".to_string());
        let machine_identifier = extract_xml_attr(&body, "machineIdentifier")
            .unwrap_or_else(|| "Unknown".to_string());

        // Validate token by hitting /library/sections
        let sections_url = format!("{}/library/sections", self.server_url);
        let sec_resp = self
            .http
            .get(&sections_url)
            .header("X-Plex-Token", &self.token)
            .send()
            .await
            .map_err(|e| format!("Error validating token: {e}"))?;

        if sec_resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err("Invalid Plex token: 401 Unauthorized".to_string());
        }
        if !sec_resp.status().is_success() {
            return Err(format!("Plex sections returned HTTP {}", sec_resp.status()));
        }

        let sec_text = sec_resp.text().await.unwrap_or_default();
        let server_name = extract_json_title1(&sec_text);

        Ok(PlexServerInfo {
            machine_identifier,
            version,
            name: server_name,
        })
    }

    /// Fetches all movie and TV show library sections from the Plex server.
    pub async fn fetch_sections(&self) -> Result<Vec<PlexSection>, String> {
        let url = format!("{}/library/sections", self.server_url);
        let resp = self
            .http
            .get(&url)
            .header("X-Plex-Token", &self.token)
            .send()
            .await
            .map_err(|e| format!("Failed to reach Plex server: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Plex returned HTTP {}", resp.status()));
        }

        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse sections JSON: {e}"))?;

        let mut sections = Vec::new();
        if let Some(dirs) = val["MediaContainer"]["Directory"].as_array() {
            for dir in dirs {
                let sec_type = dir["type"].as_str().unwrap_or("").to_string();
                if sec_type != "movie" && sec_type != "show" {
                    continue;
                }

                let key = dir["key"].as_str().unwrap_or("").to_string();
                let title = dir["title"].as_str().unwrap_or("").to_string();

                let mut locations = Vec::new();
                if let Some(locs) = dir["Location"].as_array() {
                    for loc in locs {
                        if let Some(p) = loc["path"].as_str() {
                            locations.push(p.to_string());
                        }
                    }
                }

                if !key.is_empty() {
                    sections.push(PlexSection {
                        key,
                        title,
                        section_type: sec_type,
                        locations,
                    });
                }
            }
        }

        Ok(sections)
    }

    /// Fetches all media items for a section (movies or flat TV episodes).
    pub async fn fetch_section_items(
        &self,
        section_key: &str,
        section_type: &str,
    ) -> Result<Vec<PlexRawItem>, String> {
        self.fetch_section_items_with_title(section_key, section_type, None)
            .await
    }

    /// Fetches all media items for a section with optional library title context.
    ///
    /// For TV show libraries (`section_type == "show"`), Plex stores genre classifications
    /// at the series (grandparent) level rather than on individual episodes. This method
    /// automatically fetches the series metadata list and inherits show genres, release year,
    /// and audience rating onto each child episode so that genre filters and explorer tabs
    /// work seamlessly across television libraries.
    pub async fn fetch_section_items_with_title(
        &self,
        section_key: &str,
        section_type: &str,
        section_title: Option<&str>,
    ) -> Result<Vec<PlexRawItem>, String> {
        let is_anime_lib = section_title
            .map(|t| t.to_lowercase().contains("anime"))
            .unwrap_or(false);

        let mut show_info_by_key: HashMap<String, PlexShowInfo> = HashMap::new();
        let mut show_info_by_title: HashMap<String, PlexShowInfo> = HashMap::new();

        if section_type == "show" {
            // Fetch series list to collect series-level metadata (genres, year, rating)
            let show_url = format!(
                "{}/library/sections/{}/all?includeGuids=1",
                self.server_url, section_key
            );
            if let Ok(show_resp) = self
                .http
                .get(&show_url)
                .header("X-Plex-Token", &self.token)
                .send()
                .await
            {
                if show_resp.status().is_success() {
                    if let Ok(show_val) = show_resp.json::<serde_json::Value>().await {
                        if let Some(show_arr) = show_val["MediaContainer"]["Metadata"].as_array() {
                            for show in show_arr {
                                let mut genres = Vec::new();
                                if let Some(genre_arr) = show["Genre"].as_array() {
                                    for g in genre_arr {
                                        if let Some(tag) = g["tag"].as_str() {
                                            let trimmed = tag.trim();
                                            if !trimmed.is_empty() {
                                                genres.push(trimmed.to_string());
                                            }
                                        }
                                    }
                                }

                                if is_anime_lib {
                                    if !genres.iter().any(|g| g.eq_ignore_ascii_case("animation")) {
                                        genres.push("Animation".to_string());
                                    }
                                    if !genres.iter().any(|g| g.eq_ignore_ascii_case("anime")) {
                                        genres.push("Anime".to_string());
                                    }
                                } else if genres.iter().any(|g| g.eq_ignore_ascii_case("anime")) {
                                    if !genres.iter().any(|g| g.eq_ignore_ascii_case("animation")) {
                                        genres.push("Animation".to_string());
                                    }
                                }

                                let year = show["year"].as_u64().map(|y| y as u32).or_else(|| {
                                    show["originallyAvailableAt"]
                                        .as_str()
                                        .and_then(|d| d.split('-').next())
                                        .and_then(|y| y.parse::<u32>().ok())
                                });

                                let rating = show["rating"]
                                    .as_f64()
                                    .or_else(|| show["audienceRating"].as_f64())
                                    .map(|r| r as f32);

                                let info = PlexShowInfo {
                                    genres,
                                    year,
                                    rating,
                                };

                                if let Some(rk) = show["ratingKey"].as_str() {
                                    show_info_by_key.insert(rk.to_string(), info.clone());
                                } else if let Some(rk_num) = show["ratingKey"].as_u64() {
                                    show_info_by_key.insert(rk_num.to_string(), info.clone());
                                }

                                if let Some(title) = show["title"].as_str() {
                                    let clean_title = title.trim().to_lowercase();
                                    if !clean_title.is_empty() {
                                        show_info_by_title.insert(clean_title, info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let url = if section_type == "show" {
            // type=4 retrieves all episodes across all seasons in a single flat call
            format!(
                "{}/library/sections/{}/all?type=4&includeGuids=1",
                self.server_url, section_key
            )
        } else {
            format!(
                "{}/library/sections/{}/all?includeGuids=1",
                self.server_url, section_key
            )
        };

        let resp = self
            .http
            .get(&url)
            .header("X-Plex-Token", &self.token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch items for section {section_key}: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Section {section_key} returned HTTP {}", resp.status()));
        }

        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse section {section_key} JSON: {e}"))?;

        let mut items = Vec::new();
        if let Some(meta_arr) = val["MediaContainer"]["Metadata"].as_array() {
            for item in meta_arr {
                if let Some(raw) = parse_plex_metadata_item(
                    item,
                    section_type,
                    Some(&show_info_by_key),
                    Some(&show_info_by_title),
                    is_anime_lib,
                ) {
                    items.push(raw);
                }
            }
        }

        Ok(items)
    }

    /// Fetches recently added items directly from Plex across all libraries.
    pub async fn fetch_recently_added(&self, limit: usize) -> Result<Vec<PlexRawItem>, String> {
        let url = format!(
            "{}/library/recentlyAdded?X-Plex-Container-Start=0&X-Plex-Container-Size={limit}",
            self.server_url
        );
        let resp = self
            .http
            .get(&url)
            .header("X-Plex-Token", &self.token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch recently added: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("recentlyAdded returned HTTP {}", resp.status()));
        }

        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse recently added JSON: {e}"))?;

        let mut items = Vec::new();
        if let Some(meta_arr) = val["MediaContainer"]["Metadata"].as_array() {
            for item in meta_arr {
                let st = item["type"].as_str().unwrap_or("movie");
                if let Some(raw) = parse_plex_metadata_item(item, st, None, None, false) {
                    items.push(raw);
                }
            }
        }

        Ok(items)
    }

    /// Fetches continue watching / on-deck items directly from Plex.
    pub async fn fetch_on_deck(&self) -> Result<Vec<PlexRawItem>, String> {
        let url = format!("{}/library/onDeck", self.server_url);
        let resp = self
            .http
            .get(&url)
            .header("X-Plex-Token", &self.token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch on-deck items: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("onDeck returned HTTP {}", resp.status()));
        }

        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse on-deck JSON: {e}"))?;

        let mut items = Vec::new();
        if let Some(meta_arr) = val["MediaContainer"]["Metadata"].as_array() {
            for item in meta_arr {
                let st = item["type"].as_str().unwrap_or("movie");
                if let Some(raw) = parse_plex_metadata_item(item, st, None, None, false) {
                    items.push(raw);
                }
            }
        }

        Ok(items)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlexShowInfo {
    pub genres: Vec<String>,
    pub year: Option<u32>,
    pub rating: Option<f32>,
}

/// Parses an individual JSON metadata entry from Plex into a `PlexRawItem`.
fn parse_plex_metadata_item(
    item: &serde_json::Value,
    section_type: &str,
    show_info_by_key: Option<&HashMap<String, PlexShowInfo>>,
    show_info_by_title: Option<&HashMap<String, PlexShowInfo>>,
    is_anime_lib: bool,
) -> Option<PlexRawItem> {
    let rating_key = item["ratingKey"].as_str()?.to_string();
    let title = item["title"].as_str().unwrap_or("").to_string();

    let mut file_paths = Vec::new();
    if let Some(media_arr) = item["Media"].as_array() {
        for m in media_arr {
            if let Some(part_arr) = m["Part"].as_array() {
                for p in part_arr {
                    if let Some(f) = p["file"].as_str() {
                        if !f.is_empty() {
                            file_paths.push(f.to_string());
                        }
                    }
                }
            }
        }
    }

    if file_paths.is_empty() {
        return None;
    }

    let is_episode = section_type == "show"
        || section_type == "episode"
        || item["type"].as_str() == Some("episode");
    let series_title = if is_episode {
        item["grandparentTitle"].as_str().map(|s| s.to_string())
    } else {
        None
    };

    let season_number = if is_episode {
        item["parentIndex"].as_u64().map(|n| n as u32)
    } else {
        None
    };

    let episode_number = if is_episode {
        item["index"].as_u64().map(|n| n as u32)
    } else {
        None
    };

    // Lookup matching show info if available
    let parent_show_info = if is_episode {
        let from_key = if let Some(by_key) = show_info_by_key {
            if let Some(gp_key) = item["grandparentRatingKey"].as_str() {
                by_key.get(gp_key)
            } else if let Some(gp_key_num) = item["grandparentRatingKey"].as_u64() {
                by_key.get(&gp_key_num.to_string())
            } else {
                None
            }
        } else {
            None
        };

        from_key.or_else(|| {
            if let Some(by_title) = show_info_by_title {
                if let Some(ref st) = series_title {
                    by_title.get(&st.trim().to_lowercase())
                } else {
                    None
                }
            } else {
                None
            }
        })
    } else {
        None
    };

    let year = item["year"]
        .as_u64()
        .map(|y| y as u32)
        .or_else(|| {
            item["originallyAvailableAt"]
                .as_str()
                .and_then(|d| d.split('-').next())
                .and_then(|y| y.parse::<u32>().ok())
        })
        .or_else(|| parent_show_info.and_then(|s| s.year));

    let tagline = item["tagline"].as_str().map(|s| s.to_string());
    let summary = item["summary"].as_str().map(|s| s.to_string());
    let content_rating = item["contentRating"].as_str().map(|s| s.to_string());
    let duration_ms = item["duration"].as_u64();
    let rating = item["rating"]
        .as_f64()
        .or_else(|| item["audienceRating"].as_f64())
        .map(|r| r as f32)
        .or_else(|| parent_show_info.and_then(|s| s.rating));
    let studio = item["studio"].as_str().map(|s| s.to_string());

    let mut genres = Vec::new();
    if let Some(genre_arr) = item["Genre"].as_array() {
        for g in genre_arr {
            if let Some(tag) = g["tag"].as_str() {
                let trimmed = tag.trim();
                if !trimmed.is_empty() {
                    genres.push(trimmed.to_string());
                }
            }
        }
    }

    if genres.is_empty() {
        if let Some(show_info) = parent_show_info {
            genres = show_info.genres.clone();
        }
    }

    if is_anime_lib {
        if !genres.iter().any(|g| g.eq_ignore_ascii_case("animation")) {
            genres.push("Animation".to_string());
        }
        if !genres.iter().any(|g| g.eq_ignore_ascii_case("anime")) {
            genres.push("Anime".to_string());
        }
    } else if genres.iter().any(|g| g.eq_ignore_ascii_case("anime")) {
        if !genres.iter().any(|g| g.eq_ignore_ascii_case("animation")) {
            genres.push("Animation".to_string());
        }
    }

    let mut directors = Vec::new();
    if let Some(dir_arr) = item["Director"].as_array() {
        for d in dir_arr {
            if let Some(tag) = d["tag"].as_str() {
                directors.push(tag.to_string());
            }
        }
    }

    let mut actors = Vec::new();
    if let Some(role_arr) = item["Role"].as_array() {
        for r in role_arr {
            if let Some(tag) = r["tag"].as_str() {
                actors.push(tag.to_string());
            }
        }
    }

    let thumb_url = item["thumb"].as_str().map(|s| s.to_string());
    let art_url = item["art"].as_str().map(|s| s.to_string());
    let view_offset_ms = item["viewOffset"].as_u64();

    Some(PlexRawItem {
        rating_key,
        title,
        series_title,
        season_number,
        episode_number,
        year,
        tagline,
        summary,
        content_rating,
        duration_ms,
        rating,
        studio,
        genres,
        directors,
        actors,
        thumb_url,
        art_url,
        file_paths,
        view_offset_ms,
    })
}

/// Matches a Plex server file path against the local database of video file paths.
///
/// Uses hierarchical suffix matching so that files mounted at `/mnt/bob/movies/...`
/// accurately match Plex server paths like `/pub/movies/...` or `/volume1/...`.
pub fn match_plex_path(
    plex_file: &str,
    local_index: &HashMap<String, i64>,
    normalized_local_cache: &[(String, i64)],
) -> Option<i64> {
    let norm_plex = plex_file.replace('\\', "/");

    // Tier 1: Direct exact match
    if let Some(&id) = local_index.get(&norm_plex) {
        return Some(id);
    }

    let segments: Vec<&str> = norm_plex.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return None;
    }

    // Tier 2: 2-segment suffix (e.g. "Alien (1979)/Alien (1979).mkv" or "Season 1/S01E01.mkv")
    if segments.len() >= 2 {
        let suffix_2 = format!(
            "/{}/{}",
            segments[segments.len() - 2],
            segments[segments.len() - 1]
        );
        let mut matches: Vec<i64> = Vec::new();

        for (norm_local, id) in normalized_local_cache {
            if norm_local.ends_with(&suffix_2) {
                matches.push(*id);
            }
        }

        if matches.len() == 1 {
            return Some(matches[0]);
        }

        // Tier 3: 3-segment suffix if disambiguation needed (e.g. "TV/Show/S01E01.mkv")
        if segments.len() >= 3 && matches.len() > 1 {
            let suffix_3 = format!(
                "/{}/{}/{}",
                segments[segments.len() - 3],
                segments[segments.len() - 2],
                segments[segments.len() - 1]
            );
            for (norm_local, id) in normalized_local_cache {
                if norm_local.ends_with(&suffix_3) {
                    return Some(*id);
                }
            }
        }
    }

    // Tier 4: Fallback to exact filename match if unique across whole local library
    let filename = format!("/{}", segments[segments.len() - 1]);
    let mut filename_matches: Vec<i64> = Vec::new();
    for (norm_local, id) in normalized_local_cache {
        if norm_local.ends_with(&filename) {
            filename_matches.push(*id);
        }
    }

    if filename_matches.len() == 1 {
        return Some(filename_matches[0]);
    }

    None
}

/// Orchestrates full metadata sync from Plex Media Server into Wazoo's SQLite database.
pub async fn sync_plex_library(
    client: &PlexClient,
    db: Arc<Mutex<Database>>,
    progress_tx: Option<mpsc::Sender<PlexSyncProgress>>,
) -> Result<PlexSyncSummary, String> {
    let sections = client.fetch_sections().await?;
    let mut summary = PlexSyncSummary {
        libraries_scanned: sections.len(),
        ..Default::default()
    };

    // Load local videos for path correlation
    let (local_index, norm_cache) = {
        let guard = db.lock().await;
        let videos = guard
            .get_all_videos()
            .map_err(|e| format!("Failed to read videos from db: {e}"))?;
        let mut idx = HashMap::new();
        let mut norm = Vec::new();
        for v in videos {
            let n = v.path.replace('\\', "/");
            idx.insert(n.clone(), v.id);
            norm.push((n, v.id));
        }
        (idx, norm)
    };

    let total_sections = sections.len();
    for (sec_idx, sec) in sections.iter().enumerate() {
        if let Some(ref tx) = progress_tx {
            let _ = tx
                .send(PlexSyncProgress {
                    stage: format!("Fetching {} ({}/{})", sec.title, sec_idx + 1, total_sections),
                    processed_items: 0,
                    total_items: 0,
                    matched_items: summary.matched_and_updated,
                    current_library: sec.title.clone(),
                })
                .await;
        }

        let items = client
            .fetch_section_items_with_title(&sec.key, &sec.section_type, Some(&sec.title))
            .await?;
        summary.total_plex_items += items.len();

        let mut to_upsert = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for (item_idx, item) in items.iter().enumerate() {
            for remote_file in &item.file_paths {
                if let Some(video_id) = match_plex_path(remote_file, &local_index, &norm_cache) {
                    to_upsert.push(MediaMetadata {
                        video_id,
                        plex_rating_key: Some(item.rating_key.clone()),
                        title: item.title.clone(),
                        series_title: item.series_title.clone(),
                        season_number: item.season_number,
                        episode_number: item.episode_number,
                        year: item.year,
                        tagline: item.tagline.clone(),
                        summary: item.summary.clone(),
                        content_rating: item.content_rating.clone(),
                        duration_ms: item.duration_ms,
                        rating: item.rating,
                        studio: item.studio.clone(),
                        genres: item.genres.clone(),
                        directors: item.directors.clone(),
                        actors: item.actors.clone(),
                        thumb_url: item.thumb_url.clone(),
                        art_url: item.art_url.clone(),
                        updated_at: Some(now),
                    });
                    summary.matched_and_updated += 1;
                    break;
                }
            }

            if item_idx % 100 == 0 && progress_tx.is_some() {
                if let Some(ref tx) = progress_tx {
                    let _ = tx
                        .send(PlexSyncProgress {
                            stage: format!("Indexing {}", sec.title),
                            processed_items: item_idx + 1,
                            total_items: items.len(),
                            matched_items: summary.matched_and_updated,
                            current_library: sec.title.clone(),
                        })
                        .await;
                }
            }
        }

        if !to_upsert.is_empty() {
            let mut guard = db.lock().await;
            guard
                .batch_upsert_metadata(&to_upsert)
                .map_err(|e| format!("Failed to write metadata batch to database: {e}"))?;
        }
    }

    Ok(summary)
}

fn extract_xml_attr(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = xml.find(&pattern)? + pattern.len();
    let end = xml[start..].find('"')? + start;
    Some(xml[start..end].to_string())
}

fn extract_json_title1(json_str: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
    val["MediaContainer"]["title1"]
        .as_str()
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_plex_path_exact_and_suffix() {
        let mut local_index = HashMap::new();
        local_index.insert(
            "/mnt/bob/movies/Alien (1979)/Alien.1080p.mkv".to_string(),
            100,
        );
        local_index.insert(
            "/mnt/bob/tv/Breaking Bad (2008)/Season 1/S01E01.mkv".to_string(),
            200,
        );

        let norm_cache: Vec<(String, i64)> = local_index
            .iter()
            .map(|(p, &id)| (p.clone(), id))
            .collect();

        // Remote NAS path has /pub instead of /mnt/bob
        let plex_path1 = "/pub/movies/Alien (1979)/Alien.1080p.mkv";
        let matched1 = match_plex_path(plex_path1, &local_index, &norm_cache);
        assert_eq!(matched1, Some(100));

        // Remote path with backslashes
        let plex_path2 = "\\pub\\tv\\Breaking Bad (2008)\\Season 1\\S01E01.mkv";
        let matched2 = match_plex_path(plex_path2, &local_index, &norm_cache);
        assert_eq!(matched2, Some(200));

        // Non-existent path
        let plex_path3 = "/pub/movies/Unknown.mkv";
        let matched3 = match_plex_path(plex_path3, &local_index, &norm_cache);
        assert_eq!(matched3, None);
    }

    #[test]
    fn test_tv_episode_inherits_show_genres_year_and_rating() {
        let mut show_by_key = HashMap::new();
        show_by_key.insert(
            "1001".to_string(),
            PlexShowInfo {
                genres: vec!["Animation".to_string(), "Action".to_string()],
                year: Some(1964),
                rating: Some(8.5),
            },
        );

        let episode_json = serde_json::json!({
            "ratingKey": "2001",
            "grandparentRatingKey": "1001",
            "grandparentTitle": "Jonny Quest",
            "title": "The Mystery of the Lizard Men",
            "type": "episode",
            "parentIndex": 1,
            "index": 1,
            "Media": [{
                "Part": [{
                    "file": "/media/tv/Jonny Quest/S01E01.mkv"
                }]
            }]
        });

        let parsed = parse_plex_metadata_item(
            &episode_json,
            "show",
            Some(&show_by_key),
            None,
            false,
        )
        .expect("parsed item");

        assert_eq!(parsed.series_title.as_deref(), Some("Jonny Quest"));
        assert_eq!(parsed.genres, vec!["Animation", "Action"]);
        assert_eq!(parsed.year, Some(1964));
        assert_eq!(parsed.rating, Some(8.5));
    }

    #[test]
    fn test_anime_library_ensures_animation_and_anime_tags() {
        let mut show_by_key = HashMap::new();
        show_by_key.insert(
            "1002".to_string(),
            PlexShowInfo {
                genres: vec!["Sci-Fi".to_string(), "Mecha".to_string()],
                year: Some(1995),
                rating: Some(9.0),
            },
        );

        let episode_json = serde_json::json!({
            "ratingKey": "2002",
            "grandparentRatingKey": "1002",
            "grandparentTitle": "Neon Genesis Evangelion",
            "title": "Angel Attack",
            "type": "episode",
            "Media": [{
                "Part": [{
                    "file": "/media/anime/NGE/S01E01.mkv"
                }]
            }]
        });

        let parsed = parse_plex_metadata_item(
            &episode_json,
            "show",
            Some(&show_by_key),
            None,
            true, // is_anime_lib
        )
        .expect("parsed item");

        assert!(parsed.genres.contains(&"Animation".to_string()));
        assert!(parsed.genres.contains(&"Anime".to_string()));
        assert!(parsed.genres.contains(&"Sci-Fi".to_string()));
    }
}

