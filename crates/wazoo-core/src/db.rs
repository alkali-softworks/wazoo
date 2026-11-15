/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * SQLite Database Manager
 *
 * Provides database initialization, indexing, full-text pattern search, folder filtering,
 * and bulk upsert operations for indexed video files.
 */

use crate::models::{MediaMetadata, PlexExplorerItem, PlexExplorerTab, VideoRecord};
use crate::plex::{PlexRawItem, match_plex_path};
use rusqlite::{Connection, OptionalExtension, Result, params};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    pub fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS Video (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                folder TEXT
            )",
            [],
        )?;

        // Non-destructive migration if column didn't exist previously
        let _ = self
            .conn
            .execute("ALTER TABLE Video ADD COLUMN folder TEXT", []);

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS path_idx ON Video(path)", [])?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS name_idx ON Video(name)", [])?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS folder_idx ON Video(folder)", [])?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS VideoMetadata (
                video_id INTEGER PRIMARY KEY,
                plex_rating_key TEXT,
                title TEXT NOT NULL,
                series_title TEXT,
                season_number INTEGER,
                episode_number INTEGER,
                year INTEGER,
                tagline TEXT,
                summary TEXT,
                content_rating TEXT,
                duration_ms INTEGER,
                rating REAL,
                studio TEXT,
                genres TEXT,
                directors TEXT,
                actors TEXT,
                thumb_url TEXT,
                art_url TEXT,
                updated_at INTEGER,
                FOREIGN KEY (video_id) REFERENCES Video(id) ON DELETE CASCADE
            )",
            [],
        )?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS meta_year_idx ON VideoMetadata(year)", [])?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS meta_title_idx ON VideoMetadata(title)", [])?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS meta_series_idx ON VideoMetadata(series_title)", [])?;

        Ok(())
    }

    pub fn insert_or_update_video(&self, video: &VideoRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO Video (name, path, folder)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                folder = COALESCE(excluded.folder, Video.folder)",
            params![video.name, video.path, video.folder],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_misc_video(&self, name: &str, path: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO Video (name, path, folder)
             VALUES (?1, ?2, 'Misc')
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                folder = 'Misc'",
            params![name, path],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn clear_misc_videos(&self) -> Result<usize> {
        let count = self
            .conn
            .execute("DELETE FROM Video WHERE folder = 'Misc'", [])?;
        Ok(count)
    }

    pub fn get_misc_video_count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM Video WHERE folder = 'Misc'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn has_misc_videos(&self) -> Result<bool> {
        Ok(self.get_misc_video_count()? > 0)
    }

    pub fn is_video_in_other_folder(&self, path: &str, media_folders: &[String]) -> Result<bool> {
        let p = Path::new(path);
        let canon_p = std::fs::canonicalize(p).ok();
        for folder in media_folders {
            let f = Path::new(folder);
            if p.starts_with(f) {
                return Ok(true);
            }
            if let Some(ref cp) = canon_p {
                if let Ok(cf) = std::fs::canonicalize(f) {
                    if cp.starts_with(&cf) {
                        return Ok(true);
                    }
                }
            }
        }

        let mut stmt = self
            .conn
            .prepare("SELECT folder FROM Video WHERE path = ?1")?;
        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            let folder: Option<String> = row.get(0)?;
            if folder.as_deref() != Some("Misc") {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn batch_insert_videos(&mut self, videos: &[VideoRecord]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO Video (name, path, folder)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    folder = COALESCE(excluded.folder, Video.folder)",
            )?;

            for video in videos {
                stmt.execute(params![video.name, video.path, video.folder])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn prune_missing_videos(&mut self, existing_paths: &[String]) -> Result<usize> {
        let all_db = self.get_all_videos()?;
        let path_set: HashSet<&str> = existing_paths.iter().map(|s| s.as_str()).collect();
        let stale: Vec<i64> = all_db
            .into_iter()
            .filter(|v| {
                if v.folder.as_deref() == Some("Misc") {
                    !Path::new(&v.path).exists()
                } else {
                    !path_set.contains(v.path.as_str())
                }
            })
            .map(|v| v.id)
            .collect();
        if stale.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.transaction()?;
        let count = stale.len();
        {
            let mut stmt = tx.prepare("DELETE FROM Video WHERE id = ?1")?;
            for id in &stale {
                stmt.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn remove_videos_in_folder(&mut self, folder: &str) -> Result<usize> {
        if folder == "Misc" {
            return self.clear_misc_videos();
        }

        let folder_clean = folder.trim_end_matches(['/', '\\']);
        let folder_prefix_slash = format!("{folder_clean}/");
        let folder_prefix_backslash = format!("{folder_clean}\\");
        let folder_path = std::path::Path::new(folder_clean);
        let canon_folder = std::fs::canonicalize(folder_path).ok();

        let all_db = self.get_all_videos()?;
        let matching_ids: Vec<i64> = all_db
            .into_iter()
            .filter(|v| {
                if v.path == folder_clean
                    || v.path.starts_with(&folder_prefix_slash)
                    || v.path.starts_with(&folder_prefix_backslash)
                {
                    return true;
                }
                let p = std::path::Path::new(&v.path);
                if p.starts_with(folder_path) {
                    return true;
                }
                if let Some(ref cf) = canon_folder {
                    let canon_cf_str = cf.to_string_lossy();
                    if v.path.contains(folder_clean) || v.path.contains(&*canon_cf_str) {
                        if let Ok(canon) = std::fs::canonicalize(p) {
                            if canon.starts_with(cf) {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .map(|v| v.id)
            .collect();

        if matching_ids.is_empty() {
            return Ok(0);
        }

        let count = matching_ids.len();
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM Video WHERE id = ?1")?;
            for id in &matching_ids {
                stmt.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn get_video_count(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM Video", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_all_videos(&self) -> Result<Vec<VideoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, folder
             FROM Video ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(VideoRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                folder: row.get(3)?,
            })
        })?;

        let mut videos = Vec::new();
        for r in rows {
            videos.push(r?);
        }
        Ok(videos)
    }

    pub fn upsert_metadata(&self, meta: &MediaMetadata) -> Result<()> {
        let genres_json = serde_json::to_string(&meta.genres).unwrap_or_default();
        let directors_json = serde_json::to_string(&meta.directors).unwrap_or_default();
        let actors_json = serde_json::to_string(&meta.actors).unwrap_or_default();

        self.conn.execute(
            "INSERT INTO VideoMetadata (
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
                updated_at = excluded.updated_at",
            params![
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
            ],
        )?;
        Ok(())
    }

    pub fn batch_upsert_metadata(&mut self, metas: &[MediaMetadata]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO VideoMetadata (
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
                    updated_at = excluded.updated_at",
            )?;

            for meta in metas {
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
                ])?;
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
            let genres_str: Option<String> = row.get(13)?;
            let directors_str: Option<String> = row.get(14)?;
            let actors_str: Option<String> = row.get(15)?;

            let genres: Vec<String> = genres_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let directors: Vec<String> = directors_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let actors: Vec<String> = actors_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(Some(MediaMetadata {
                video_id: row.get(0)?,
                plex_rating_key: row.get(1)?,
                title: row.get(2)?,
                series_title: row.get(3)?,
                season_number: row.get(4)?,
                episode_number: row.get(5)?,
                year: row.get(6)?,
                tagline: row.get(7)?,
                summary: row.get(8)?,
                content_rating: row.get(9)?,
                duration_ms: row.get(10)?,
                rating: row.get(11)?,
                studio: row.get(12)?,
                genres,
                directors,
                actors,
                thumb_url: row.get(16)?,
                art_url: row.get(17)?,
                updated_at: row.get(18)?,
            }))
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
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM VideoMetadata", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_all_metadata_map(&self) -> Result<std::collections::HashMap<i64, MediaMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT video_id, plex_rating_key, title, series_title, season_number, episode_number,
                    year, tagline, summary, content_rating, duration_ms, rating, studio,
                    genres, directors, actors, thumb_url, art_url, updated_at
             FROM VideoMetadata",
        )?;

        let mut map = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            let genres_str: Option<String> = row.get(13)?;
            let directors_str: Option<String> = row.get(14)?;
            let actors_str: Option<String> = row.get(15)?;

            let genres: Vec<String> = genres_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let directors: Vec<String> = directors_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            let actors: Vec<String> = actors_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(MediaMetadata {
                video_id: row.get(0)?,
                plex_rating_key: row.get(1)?,
                title: row.get(2)?,
                series_title: row.get(3)?,
                season_number: row.get(4)?,
                episode_number: row.get(5)?,
                year: row.get(6)?,
                tagline: row.get(7)?,
                summary: row.get(8)?,
                content_rating: row.get(9)?,
                duration_ms: row.get(10)?,
                rating: row.get(11)?,
                studio: row.get(12)?,
                genres,
                directors,
                actors,
                thumb_url: row.get(16)?,
                art_url: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })?;

        for r in rows {
            let item = r?;
            map.insert(item.video_id, item);
        }
        Ok(map)
    }

    pub fn search_videos(&self, query_str: &str, folders: &[String]) -> Result<Vec<VideoRecord>> {
        let clauses: Vec<&str> = query_str
            .split(',')
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect();

        let mut where_conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<String> = Vec::new();

        // Process search clauses
        if !clauses.is_empty() {
            let mut positive_conditions: Vec<String> = Vec::new();
            let mut negative_conditions: Vec<String> = Vec::new();

            for clause in clauses {
                let trimmed = clause.trim();
                let (is_not, term) =
                    if trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("not ") {
                        (true, trimmed[4..].trim())
                    } else if let Some(rest) = trimmed.strip_prefix('!') {
                        (true, rest.trim())
                    } else {
                        (false, trimmed)
                    };

                if term.is_empty() {
                    continue;
                }

                let has_start_anchor = term.starts_with('^') && !term.starts_with("^^");
                let has_end_anchor = term.len() > 1 && term.ends_with('$') && !term.ends_with("$$");

                let core_term = match (has_start_anchor, has_end_anchor) {
                    (true, true) => {
                        if term.len() <= 2 {
                            ""
                        } else {
                            &term[1..term.len() - 1]
                        }
                    }
                    (true, false) => &term[1..],
                    (false, true) => &term[..term.len() - 1],
                    (false, false) => term,
                };

                if core_term.is_empty() {
                    continue;
                }

                // Escape SQL LIKE special characters: '\', '%', and '_'
                // Also support '*' as an unescaped SQL wildcard '%'
                let escaped_term = core_term
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
                    .replace('*', "%");

                // Match original wazoo-js: replace '-' with space, split words, join with '%'
                let words = escaped_term
                    .replace('-', " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join("%");

                let is_path_prefix = has_start_anchor
                    && (core_term.starts_with('/')
                        || core_term.starts_with('\\')
                        || (core_term.len() >= 2 && core_term.as_bytes()[1] == b':'));

                if is_path_prefix {
                    let pattern = if has_end_anchor {
                        words
                    } else {
                        format!("{words}%")
                    };
                    param_values.push(pattern);
                    let p_idx = param_values.len();
                    if is_not {
                        negative_conditions.push(format!("v.path NOT LIKE ?{p_idx} ESCAPE '\\'"));
                    } else {
                        positive_conditions.push(format!("v.path LIKE ?{p_idx} ESCAPE '\\'"));
                    }
                } else {
                    match (has_start_anchor, has_end_anchor) {
                        (true, true) => {
                            // Exact anchor: ^term$
                            param_values.push(words.clone());
                            let p1 = param_values.len();
                            param_values.push(format!("%/{words}.%"));
                            let p2 = param_values.len();
                            param_values.push(format!("%\\\\{words}.%"));
                            let p3 = param_values.len();

                            if is_not {
                                negative_conditions.push(format!(
                                    "(v.name NOT LIKE ?{p1} ESCAPE '\\' AND v.path NOT LIKE ?{p2} ESCAPE '\\' AND v.path NOT LIKE ?{p3} ESCAPE '\\' AND (m.title IS NULL OR m.title NOT LIKE ?{p1} ESCAPE '\\'))"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(v.name LIKE ?{p1} ESCAPE '\\' OR v.path LIKE ?{p2} ESCAPE '\\' OR v.path LIKE ?{p3} ESCAPE '\\' OR m.title LIKE ?{p1} ESCAPE '\\')"
                                ));
                            }
                        }
                        (true, false) => {
                            // Prefix anchor: ^term
                            param_values.push(format!("{words}%"));
                            let p1 = param_values.len();
                            param_values.push(format!("%/{words}%"));
                            let p2 = param_values.len();
                            param_values.push(format!("%\\\\{words}%"));
                            let p3 = param_values.len();

                            if is_not {
                                negative_conditions.push(format!(
                                    "(v.name NOT LIKE ?{p1} ESCAPE '\\' AND v.path NOT LIKE ?{p2} ESCAPE '\\' AND v.path NOT LIKE ?{p3} ESCAPE '\\' AND (m.title IS NULL OR m.title NOT LIKE ?{p1} ESCAPE '\\'))"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(v.name LIKE ?{p1} ESCAPE '\\' OR v.path LIKE ?{p2} ESCAPE '\\' OR v.path LIKE ?{p3} ESCAPE '\\' OR m.title LIKE ?{p1} ESCAPE '\\')"
                                ));
                            }
                        }
                        (false, true) => {
                            // Suffix anchor: term$
                            param_values.push(format!("%{words}"));
                            let p1 = param_values.len();
                            param_values.push(format!("%{words}.%"));
                            let p2 = param_values.len();

                            if is_not {
                                negative_conditions.push(format!(
                                    "(v.name NOT LIKE ?{p1} ESCAPE '\\' AND v.path NOT LIKE ?{p2} ESCAPE '\\' AND v.path NOT LIKE ?{p1} ESCAPE '\\' AND (m.title IS NULL OR m.title NOT LIKE ?{p1} ESCAPE '\\'))"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(v.name LIKE ?{p1} ESCAPE '\\' OR v.path LIKE ?{p2} ESCAPE '\\' OR v.path LIKE ?{p1} ESCAPE '\\' OR m.title LIKE ?{p1} ESCAPE '\\')"
                                ));
                            }
                        }
                        (false, false) => {
                            // Standard unanchored search matching path, name, and rich metadata
                            let wildcard = format!("%{words}%");
                            param_values.push(wildcard);
                            let p_idx = param_values.len();

                            if is_not {
                                negative_conditions.push(format!(
                                    "(v.path NOT LIKE ?{p_idx} ESCAPE '\\' AND (m.genres IS NULL OR m.genres NOT LIKE ?{p_idx} ESCAPE '\\') AND (m.title IS NULL OR m.title NOT LIKE ?{p_idx} ESCAPE '\\') AND (m.series_title IS NULL OR m.series_title NOT LIKE ?{p_idx} ESCAPE '\\') AND (m.directors IS NULL OR m.directors NOT LIKE ?{p_idx} ESCAPE '\\'))"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(v.path LIKE ?{p_idx} ESCAPE '\\' OR v.name LIKE ?{p_idx} ESCAPE '\\' OR m.title LIKE ?{p_idx} ESCAPE '\\' OR m.series_title LIKE ?{p_idx} ESCAPE '\\' OR m.genres LIKE ?{p_idx} ESCAPE '\\' OR CAST(m.year AS TEXT) LIKE ?{p_idx} ESCAPE '\\' OR m.directors LIKE ?{p_idx} ESCAPE '\\' OR m.actors LIKE ?{p_idx} ESCAPE '\\' OR m.summary LIKE ?{p_idx} ESCAPE '\\')"
                                ));
                            }
                        }
                    }
                }
            }

            // Positive clauses are grouped with OR: at least one positive term must match
            if !positive_conditions.is_empty() {
                where_conditions.push(format!("({})", positive_conditions.join(" OR ")));
            }

            // Negative clauses are grouped with AND: all negative terms must NOT match
            if !negative_conditions.is_empty() {
                where_conditions.push(format!("({})", negative_conditions.join(" AND ")));
            }
        }

        // Process folder filters
        let active_folders: Vec<&String> = folders
            .iter()
            .filter(|f| !crate::i18n::is_all_folder(f))
            .collect();

        if !active_folders.is_empty() {
            let mut folder_conditions: Vec<String> = Vec::new();
            for folder in active_folders {
                if folder == "Misc" {
                    folder_conditions.push("v.folder = 'Misc'".to_string());
                } else {
                    let escaped_folder = folder
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_");
                    let folder_prefix = format!("{escaped_folder}%");
                    param_values.push(folder_prefix);
                    folder_conditions
                        .push(format!("v.path LIKE ?{} ESCAPE '\\'", param_values.len()));
                }
            }
            where_conditions.push(format!("({})", folder_conditions.join(" OR ")));
        }

        let query = if where_conditions.is_empty() {
            "SELECT DISTINCT v.id, v.name, v.path, v.folder FROM Video v LEFT JOIN VideoMetadata m ON v.id = m.video_id ORDER BY v.path ASC".to_string()
        } else {
            format!(
                "SELECT DISTINCT v.id, v.name, v.path, v.folder FROM Video v LEFT JOIN VideoMetadata m ON v.id = m.video_id WHERE {} ORDER BY v.path ASC",
                where_conditions.join(" AND ")
            )
        };

        let mut stmt = self.conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(VideoRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                folder: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn delete_missing_paths(&mut self, existing_paths: &[String]) -> Result<usize> {
        let all_videos = self.get_all_videos()?;
        let existing_set: std::collections::HashSet<&str> =
            existing_paths.iter().map(|s| s.as_str()).collect();

        let tx = self.conn.transaction()?;
        let mut deleted = 0;
        {
            let mut stmt = tx.prepare("DELETE FROM Video WHERE id = ?1")?;
            for video in all_videos {
                let is_missing = if video.folder.as_deref() == Some("Misc") {
                    !Path::new(&video.path).exists()
                } else {
                    !existing_set.contains(video.path.as_str())
                };
                if is_missing {
                    stmt.execute(params![video.id])?;
                    deleted += 1;
                }
            }
        }
        tx.commit()?;
        Ok(deleted)
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM Video", [])?;
        Ok(())
    }

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
                format!("{} {} {}", y, dec_full, dec_short)
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
            "SELECT v.id, v.path, m.plex_rating_key, m.title, m.series_title,
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
            let genres_str: Option<String> = row.get(14)?;
            let directors_str: Option<String> = row.get(15)?;
            let actors_str: Option<String> = row.get(16)?;

            let meta = MediaMetadata {
                video_id,
                plex_rating_key: row.get(2)?,
                title: row.get(3)?,
                series_title: row.get(4)?,
                season_number: row.get(5)?,
                episode_number: row.get(6)?,
                year: row.get(7)?,
                tagline: row.get(8)?,
                summary: row.get(9)?,
                content_rating: row.get(10)?,
                duration_ms: row.get(11)?,
                rating: row.get(12)?,
                studio: row.get(13)?,
                genres: genres_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                directors: directors_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                actors: actors_str
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
                thumb_url: row.get(17)?,
                art_url: row.get(18)?,
                updated_at: row.get(19)?,
            };
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
                if !meta.genres.iter().any(|item_g| Self::genre_matches(g, item_g)) {
                    continue;
                }
            }

            if !query_words.is_empty() && !Self::metadata_matches_words(&meta, &query_words) {
                continue;
            }

            let title = meta
                .series_title
                .clone()
                .unwrap_or_else(|| meta.title.clone());
            let subtitle = if let (Some(sn), Some(en)) = (meta.season_number, meta.episode_number)
            {
                Some(format!("S{:02}E{:02} - {}", sn, en, meta.title))
            } else {
                None
            };

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

        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
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
                    let title = raw
                        .series_title
                        .clone()
                        .unwrap_or_else(|| raw.title.clone());
                    let subtitle =
                        if let (Some(sn), Some(en)) = (raw.season_number, raw.episode_number) {
                            Some(format!("S{:02}E{:02} - {}", sn, en, raw.title))
                        } else {
                            None
                        };

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
