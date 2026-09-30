/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * SQLite Database Manager
 *
 * Provides database initialization, indexing, full-text pattern search, folder filtering,
 * and bulk upsert operations for indexed video files.
 */

use crate::models::VideoRecord;
use rusqlite::{Connection, Result, params};
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
                let has_end_anchor =
                    term.len() > 1 && term.ends_with('$') && !term.ends_with("$$");

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
                        negative_conditions.push(format!("path NOT LIKE ?{p_idx} ESCAPE '\\'"));
                    } else {
                        positive_conditions.push(format!("path LIKE ?{p_idx} ESCAPE '\\'"));
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
                                    "(name NOT LIKE ?{p1} ESCAPE '\\' AND path NOT LIKE ?{p2} ESCAPE '\\' AND path NOT LIKE ?{p3} ESCAPE '\\')"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(name LIKE ?{p1} ESCAPE '\\' OR path LIKE ?{p2} ESCAPE '\\' OR path LIKE ?{p3} ESCAPE '\\')"
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
                                    "(name NOT LIKE ?{p1} ESCAPE '\\' AND path NOT LIKE ?{p2} ESCAPE '\\' AND path NOT LIKE ?{p3} ESCAPE '\\')"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(name LIKE ?{p1} ESCAPE '\\' OR path LIKE ?{p2} ESCAPE '\\' OR path LIKE ?{p3} ESCAPE '\\')"
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
                                    "(name NOT LIKE ?{p1} ESCAPE '\\' AND path NOT LIKE ?{p2} ESCAPE '\\' AND path NOT LIKE ?{p1} ESCAPE '\\')"
                                ));
                            } else {
                                positive_conditions.push(format!(
                                    "(name LIKE ?{p1} ESCAPE '\\' OR path LIKE ?{p2} ESCAPE '\\' OR path LIKE ?{p1} ESCAPE '\\')"
                                ));
                            }
                        }
                        (false, false) => {
                            // Standard unanchored search
                            let wildcard = format!("%{words}%");
                            param_values.push(wildcard);
                            let p_idx = param_values.len();

                            if is_not {
                                negative_conditions
                                    .push(format!("path NOT LIKE ?{p_idx} ESCAPE '\\'"));
                            } else {
                                positive_conditions
                                    .push(format!("path LIKE ?{p_idx} ESCAPE '\\'"));
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
                    folder_conditions.push("folder = 'Misc'".to_string());
                } else {
                    let escaped_folder = folder
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                        .replace('_', "\\_");
                    let folder_prefix = format!("{escaped_folder}%");
                    param_values.push(folder_prefix);
                    folder_conditions
                        .push(format!("path LIKE ?{} ESCAPE '\\'", param_values.len()));
                }
            }
            where_conditions.push(format!("({})", folder_conditions.join(" OR ")));
        }

        let query = if where_conditions.is_empty() {
            "SELECT id, name, path, folder FROM Video ORDER BY path ASC".to_string()
        } else {
            format!(
                "SELECT id, name, path, folder FROM Video WHERE {} ORDER BY path ASC",
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
