/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * SQLite Database Manager
 *
 * Provides database initialization, indexing, full-text pattern search, folder filtering,
 * and bulk upsert operations for indexed video files.
 */

use crate::models::{MediaMetadata, VideoRecord};
use rusqlite::{Connection, OptionalExtension, Result, params};
use std::collections::HashSet;
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
}
