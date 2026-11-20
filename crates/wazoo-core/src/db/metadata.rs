/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Media Metadata Persistence
 *
 * Handles upserts and lookups for `VideoMetadata` (genres, cast, ratings,
 * episode numbering, and Plex linkage).
 */

use super::Database;
use crate::models::MediaMetadata;
use rusqlite::{OptionalExtension, Result, Statement, params};
use std::collections::HashMap;

const METADATA_UPSERT_SQL: &str = "INSERT INTO VideoMetadata (
    video_id, plex_rating_key, title, series_title, season_number, episode_number,
    year, tagline, summary, content_rating, duration_ms, rating, studio,
    genres, directors, actors, thumb_url, art_url, updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
ON CONFLICT(video_id) DO UPDATE SET
    plex_rating_key = excluded.plex_rating_key,
    title = excluded.title,
    series_title = excluded.series_title,
    season_number = excluded.season_number,
    episode_number = excluded.episode_number,
    year = excluded.year,
    tagline = excluded.tagline,
    summary = excluded.summary,
    content_rating = excluded.content_rating,
    duration_ms = excluded.duration_ms,
    rating = excluded.rating,
    studio = excluded.studio,
    genres = excluded.genres,
    directors = excluded.directors,
    actors = excluded.actors,
    thumb_url = excluded.thumb_url,
    art_url = excluded.art_url,
    updated_at = excluded.updated_at";

fn parse_json_list(raw: Option<String>) -> Vec<String> {
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn execute_metadata_upsert(stmt: &mut Statement<'_>, meta: &MediaMetadata) -> Result<usize> {
    let genres_json = serde_json::to_string(&meta.genres).unwrap_or_default();
    let directors_json = serde_json::to_string(&meta.directors).unwrap_or_default();
    let actors_json = serde_json::to_string(&meta.actors).unwrap_or_default();

    stmt.execute(params![
        meta.video_id,
        meta.plex_rating_key,
        meta.title,
        meta.series_title,
        meta.season_number,
        meta.episode_number,
        meta.year,
        meta.tagline,
        meta.summary,
        meta.content_rating,
        meta.duration_ms,
        meta.rating,
        meta.studio,
        genres_json,
        directors_json,
        actors_json,
        meta.thumb_url,
        meta.art_url,
        meta.updated_at,
    ])
}

impl Database {
    /// Maps the 19 `VideoMetadata` columns starting at `offset` in a query row.
    pub(super) fn map_metadata_row(
        row: &rusqlite::Row<'_>,
        offset: usize,
    ) -> Result<MediaMetadata> {
        let genres = parse_json_list(row.get(offset + 13)?);
        let directors = parse_json_list(row.get(offset + 14)?);
        let actors = parse_json_list(row.get(offset + 15)?);

        Ok(MediaMetadata {
            video_id: row.get(offset)?,
            plex_rating_key: row.get(offset + 1)?,
            title: row.get(offset + 2)?,
            series_title: row.get(offset + 3)?,
            season_number: row.get(offset + 4)?,
            episode_number: row.get(offset + 5)?,
            year: row.get(offset + 6)?,
            tagline: row.get(offset + 7)?,
            summary: row.get(offset + 8)?,
            content_rating: row.get(offset + 9)?,
            duration_ms: row.get(offset + 10)?,
            rating: row.get(offset + 11)?,
            studio: row.get(offset + 12)?,
            genres,
            directors,
            actors,
            thumb_url: row.get(offset + 16)?,
            art_url: row.get(offset + 17)?,
            updated_at: row.get(offset + 18)?,
        })
    }

    pub fn upsert_metadata(&self, meta: &MediaMetadata) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(METADATA_UPSERT_SQL)?;
        execute_metadata_upsert(&mut stmt, meta)?;
        Ok(())
    }

    pub fn batch_upsert_metadata(&mut self, metas: &[MediaMetadata]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(METADATA_UPSERT_SQL)?;
            for meta in metas {
                execute_metadata_upsert(&mut stmt, meta)?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn get_metadata_by_video_id(&self, video_id: i64) -> Result<Option<MediaMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT video_id, plex_rating_key, title, series_title, season_number, episode_number,
                    year, tagline, summary, content_rating, duration_ms, rating, studio,
                    genres, directors, actors, thumb_url, art_url, updated_at
             FROM VideoMetadata WHERE video_id = ?1",
        )?;

        let mut rows = stmt.query(params![video_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_metadata_row(row, 0)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_metadata_by_path(&self, path: &str) -> Result<Option<MediaMetadata>> {
        let video_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM Video WHERE path = ?1",
                params![path],
                |r| r.get(0),
            )
            .optional()?;

        match video_id {
            Some(id) => self.get_metadata_by_video_id(id),
            None => Ok(None),
        }
    }

    pub fn get_metadata_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM VideoMetadata", [], |row| row.get(0))
    }

    pub fn get_all_metadata_map(&self) -> Result<HashMap<i64, MediaMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT video_id, plex_rating_key, title, series_title, season_number, episode_number,
                    year, tagline, summary, content_rating, duration_ms, rating, studio,
                    genres, directors, actors, thumb_url, art_url, updated_at
             FROM VideoMetadata",
        )?;

        let mut map = HashMap::new();
        let rows = stmt.query_map([], |row| Self::map_metadata_row(row, 0))?;
        for r in rows {
            let item = r?;
            map.insert(item.video_id, item);
        }
        Ok(map)
    }
}
