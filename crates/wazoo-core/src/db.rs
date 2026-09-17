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
                path TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS path_idx ON Video(path)", [])?;

        self.conn
            .execute("CREATE INDEX IF NOT EXISTS name_idx ON Video(name)", [])?;

        Ok(())
    }

    pub fn insert_or_update_video(&self, video: &VideoRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO Video (name, path)
             VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name",
            params![video.name, video.path,],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn batch_insert_videos(&mut self, videos: &[VideoRecord]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO Video (name, path)
                 VALUES (?1, ?2)
                 ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name",
            )?;

            for video in videos {
                stmt.execute(params![video.name, video.path,])?;
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
            .filter(|v| !path_set.contains(v.path.as_str()))
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
            "SELECT id, name, path
             FROM Video ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(VideoRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
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

                // Escape SQL LIKE special characters: '\', '%', and '_'
                let escaped_term = term
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_");

                // Match original wazoo-js: replace '-' with space, split words, join with '%'
                let wildcard = format!(
                    "%{}%",
                    escaped_term
                        .replace('-', " ")
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join("%")
                );

                param_values.push(wildcard);
                let param_idx = param_values.len();

                if is_not {
                    negative_conditions.push(format!("path NOT LIKE ?{param_idx} ESCAPE '\\'"));
                } else {
                    positive_conditions.push(format!("path LIKE ?{param_idx} ESCAPE '\\'"));
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
                let escaped_folder = folder
                    .replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_");
                let folder_prefix = format!("{escaped_folder}%");
                param_values.push(folder_prefix);
                folder_conditions.push(format!("path LIKE ?{} ESCAPE '\\'", param_values.len()));
            }
            where_conditions.push(format!("({})", folder_conditions.join(" OR ")));
        }

        let query = if where_conditions.is_empty() {
            "SELECT id, name, path FROM Video ORDER BY path ASC".to_string()
        } else {
            format!(
                "SELECT id, name, path FROM Video WHERE {} ORDER BY path ASC",
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
                if !existing_set.contains(video.path.as_str()) {
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

