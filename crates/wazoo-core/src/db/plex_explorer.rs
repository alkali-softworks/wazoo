/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Plex Explorer & Live Plex Matching Queries
 *
 * Queries the indexed SQLite library + `VideoMetadata` for Plex Explorer tabs
 * (decade/genre/recently-added/random) and matches live Plex items to local paths.
 */

use super::Database;
use crate::models::{MediaMetadata, PlexExplorerItem, PlexExplorerTab};
use crate::plex::{PlexRawItem, match_plex_path};
use rusqlite::{Result, params};
use std::collections::HashMap;

fn genre_matches(filter_genre: &str, item_genre: &str) -> bool {
    let f = filter_genre.trim().to_lowercase();
    let g = item_genre.trim().to_lowercase();
    if f == "sci-fi" || f == "scifi" || f == "science fiction" {
        g.contains("science fiction") || g.contains("sci-fi") || g.contains("scifi")
    } else if f == "animation" || f == "anime" {
        g.contains("animation") || g.contains("anime")
    } else {
        g.contains(&f)
    }
}

fn metadata_matches_words(meta: &MediaMetadata, words: &[String]) -> bool {
    let title = meta.title.to_lowercase();
    let series = meta.series_title.as_deref().unwrap_or("").to_lowercase();
    let summary = meta.summary.as_deref().unwrap_or("").to_lowercase();
    let directors = meta.directors.join(" ").to_lowercase();
    let actors = meta.actors.join(" ").to_lowercase();
    let mut genres = meta.genres.join(" ").to_lowercase();
    if genres.contains("science fiction") || genres.contains("sci-fi") {
        genres.push_str(" scifi sci-fi science fiction");
    }
    if genres.contains("anime") || genres.contains("animation") {
        genres.push_str(" anime animation");
    }
    let year_str = match meta.year {
        Some(y) => {
            let dec_full = format!("{}s", (y / 10) * 10);
            let dec_short = format!("{:02}s", ((y / 10) * 10) % 100);
            format!("{y} {dec_full} {dec_short}")
        }
        None => String::new(),
    };

    words.iter().all(|w| {
        title.contains(w)
            || series.contains(w)
            || summary.contains(w)
            || directors.contains(w)
            || actors.contains(w)
            || genres.contains(w)
            || year_str.contains(w)
    })
}

fn format_explorer_title_and_subtitle(
    series_title: Option<&str>,
    title: &str,
    season_number: Option<u32>,
    episode_number: Option<u32>,
) -> (String, Option<String>) {
    let display_title = series_title
        .map(str::to_string)
        .unwrap_or_else(|| title.to_string());
    let subtitle = match (season_number, episode_number) {
        (Some(sn), Some(en)) => Some(format!("S{sn:02}E{en:02} - {title}")),
        _ => None,
    };
    (display_title, subtitle)
}

impl Database {
    /// Queries the SQLite library + Plex metadata globally (independent of any main search filter).
    pub fn query_plex_explorer(
        &self,
        tab: &PlexExplorerTab,
        decade: Option<u32>,
        genre: Option<&str>,
        query: &str,
    ) -> Result<Vec<PlexExplorerItem>> {
        let is_filterable = matches!(
            tab,
            PlexExplorerTab::All | PlexExplorerTab::Decade(_) | PlexExplorerTab::Genre(_)
        );

        let effective_decade = if is_filterable {
            decade.or_else(|| match tab {
                PlexExplorerTab::Decade(d) => Some(*d),
                _ => None,
            })
        } else {
            None
        };

        let effective_genre = if is_filterable {
            genre.map(|s| s.to_string()).or_else(|| match tab {
                PlexExplorerTab::Genre(g) => Some(g.clone()),
                _ => None,
            })
        } else {
            None
        };

        let q = query.trim().to_lowercase();
        let query_words: Vec<String> = if q.is_empty() {
            Vec::new()
        } else {
            q.replace(['_', '-'], " ")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        let where_clause = match effective_decade {
            Some(1920) => "WHERE m.year IS NOT NULL AND m.year < 1930",
            Some(_) => "WHERE m.year IS NOT NULL AND m.year >= ?1 AND m.year < ?2",
            None => "",
        };

        let order_clause = match tab {
            PlexExplorerTab::RecentlyAdded => {
                "ORDER BY COALESCE(m.updated_at, 0) DESC, v.id DESC"
            }
            _ => "ORDER BY COALESCE(m.rating, 0.0) DESC, m.year DESC, m.title ASC",
        };

        let sql = format!(
            "SELECT v.id, v.path,
                    m.video_id, m.plex_rating_key, m.title, m.series_title,
                    m.season_number, m.episode_number, m.year, m.tagline, m.summary,
                    m.content_rating, m.duration_ms, m.rating, m.studio,
                    m.genres, m.directors, m.actors, m.thumb_url, m.art_url, m.updated_at
             FROM Video v
             INNER JOIN VideoMetadata m ON v.id = m.video_id
             {where_clause}
             {order_clause}"
        );

        let mut stmt = self.conn.prepare(&sql)?;

        let row_mapper = |row: &rusqlite::Row<'_>| -> Result<(i64, String, MediaMetadata)> {
            let video_id: i64 = row.get(0)?;
            let path: String = row.get(1)?;
            let meta = Self::map_metadata_row(row, 2)?;
            Ok((video_id, path, meta))
        };

        let mut rows = match effective_decade {
            Some(dec) if dec != 1920 => stmt.query_map(params![dec, dec + 10], row_mapper)?,
            _ => stmt.query_map([], row_mapper)?,
        };

        let limit = match tab {
            PlexExplorerTab::Random => usize::MAX,
            _ => 250,
        };

        let mut results = Vec::new();
        while let Some(row_res) = rows.next() {
            let (video_id, path, meta) = row_res?;

            if let Some(ref g) = effective_genre {
                if !meta.genres.iter().any(|item_g| genre_matches(g, item_g)) {
                    continue;
                }
            }

            if !query_words.is_empty() && !metadata_matches_words(&meta, &query_words) {
                continue;
            }

            let (title, subtitle) = format_explorer_title_and_subtitle(
                meta.series_title.as_deref(),
                &meta.title,
                meta.season_number,
                meta.episode_number,
            );

            results.push(PlexExplorerItem {
                video_id,
                path,
                title,
                subtitle,
                year: meta.year,
                content_rating: meta.content_rating,
                rating: meta.rating,
                genres: meta.genres,
                duration_ms: meta.duration_ms,
                view_offset_ms: None,
                tagline: meta.tagline,
                summary: meta.summary,
                directors: meta.directors,
                actors: meta.actors,
            });

            if results.len() >= limit {
                break;
            }
        }

        Ok(results)
    }

    /// Matches live Plex items (On Deck / Recently Added) against all indexed videos in SQLite.
    pub fn match_plex_raw_items(&self, raw_items: &[PlexRawItem]) -> Result<Vec<PlexExplorerItem>> {
        if raw_items.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare("SELECT id, path FROM Video")?;
        let mut local_index: HashMap<String, i64> = HashMap::new();
        let mut id_to_path: HashMap<i64, String> = HashMap::new();
        let mut norm_cache: Vec<(String, i64)> = Vec::new();

        let rows =
            stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
        for r in rows {
            let (id, path) = r?;
            local_index.insert(path.clone(), id);
            id_to_path.insert(id, path.clone());
            norm_cache.push((path.replace('\\', "/"), id));
        }

        let mut result = Vec::new();
        for raw in raw_items {
            let matched_id = raw
                .file_paths
                .iter()
                .find_map(|p| match_plex_path(p, &local_index, &norm_cache));

            if let Some(vid) = matched_id {
                if let Some(path) = id_to_path.get(&vid) {
                    let meta = self.get_metadata_by_video_id(vid).ok().flatten();
                    let (title, subtitle) = format_explorer_title_and_subtitle(
                        raw.series_title.as_deref(),
                        &raw.title,
                        raw.season_number,
                        raw.episode_number,
                    );

                    result.push(PlexExplorerItem {
                        video_id: vid,
                        path: path.clone(),
                        title,
                        subtitle,
                        year: raw.year.or_else(|| meta.as_ref().and_then(|m| m.year)),
                        content_rating: raw
                            .content_rating
                            .clone()
                            .or_else(|| meta.as_ref().and_then(|m| m.content_rating.clone())),
                        rating: raw.rating.or_else(|| meta.as_ref().and_then(|m| m.rating)),
                        genres: if !raw.genres.is_empty() {
                            raw.genres.clone()
                        } else {
                            meta.as_ref().map(|m| m.genres.clone()).unwrap_or_default()
                        },
                        duration_ms: raw
                            .duration_ms
                            .or_else(|| meta.as_ref().and_then(|m| m.duration_ms)),
                        view_offset_ms: raw.view_offset_ms,
                        tagline: raw
                            .tagline
                            .clone()
                            .or_else(|| meta.as_ref().and_then(|m| m.tagline.clone())),
                        summary: raw
                            .summary
                            .clone()
                            .or_else(|| meta.as_ref().and_then(|m| m.summary.clone())),
                        directors: if !raw.directors.is_empty() {
                            raw.directors.clone()
                        } else {
                            meta.as_ref().map(|m| m.directors.clone()).unwrap_or_default()
                        },
                        actors: if !raw.actors.is_empty() {
                            raw.actors.clone()
                        } else {
                            meta.as_ref().map(|m| m.actors.clone()).unwrap_or_default()
                        },
                    });
                }
            }
        }

        Ok(result)
    }
}
