use std::collections::HashSet;
use std::path::Path;
use rusqlite::{params, Connection, Result};
use crate::models::VideoRecord;

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
                codec TEXT DEFAULT 'unknown',
                width INTEGER DEFAULT 0,
                height INTEGER DEFAULT 0,
                duration REAL DEFAULT 0,
                has_subtitles INTEGER DEFAULT 0,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS path_idx ON Video(path)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS name_idx ON Video(name)",
            [],
        )?;

        Ok(())
    }

    pub fn insert_or_update_video(&self, video: &VideoRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO Video (name, path, codec, width, height, duration, has_subtitles, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                codec = excluded.codec,
                width = excluded.width,
                height = excluded.height,
                duration = excluded.duration,
                has_subtitles = excluded.has_subtitles",
            params![
                video.name,
                video.path,
                video.codec,
                video.width,
                video.height,
                video.duration,
                video.has_subtitles as i32,
                video.created_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn batch_insert_videos(&mut self, videos: &[VideoRecord]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO Video (name, path, codec, width, height, duration, has_subtitles, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    codec = excluded.codec,
                    width = excluded.width,
                    height = excluded.height,
                    duration = excluded.duration,
                    has_subtitles = excluded.has_subtitles"
            )?;

            for video in videos {
                stmt.execute(params![
                    video.name,
                    video.path,
                    video.codec,
                    video.width,
                    video.height,
                    video.duration,
                    video.has_subtitles as i32,
                    video.created_at,
                ])?;
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

    pub fn get_video_count(&self) -> Result<usize> {
        let count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM Video",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_all_videos(&self) -> Result<Vec<VideoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, codec, width, height, duration, has_subtitles, created_at
             FROM Video ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(VideoRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                codec: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                duration: row.get(6)?,
                has_subtitles: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
            })
        })?;

        let mut videos = Vec::new();
        for r in rows {
            videos.push(r?);
        }
        Ok(videos)
    }

    pub fn check_video_subs(&self, file_path: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT has_subtitles FROM Video WHERE path = ?1 LIMIT 1")?;
        let mut rows = stmt.query([file_path])?;
        if let Some(row) = rows.next()? {
            let has_subs: i32 = row.get(0)?;
            Ok(has_subs != 0)
        } else {
            Ok(false)
        }
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
            let mut clause_conditions: Vec<String> = Vec::new();
            for clause in clauses {
                if let Some(term) = clause.strip_prefix("not ") {
                    let wildcard = format!("%{}%", term.split_whitespace().collect::<Vec<_>>().join("%"));
                    param_values.push(wildcard);
                    clause_conditions.push(format!("path NOT LIKE ?{}", param_values.len()));
                } else {
                    let wildcard = format!("%{}%", clause.split_whitespace().collect::<Vec<_>>().join("%"));
                    param_values.push(wildcard);
                    clause_conditions.push(format!("path LIKE ?{}", param_values.len()));
                }
            }
            if !clause_conditions.is_empty() {
                where_conditions.push(format!("({})", clause_conditions.join(" OR ")));
            }
        }

        // Process folder filters
        let active_folders: Vec<&String> = folders
            .iter()
            .filter(|f| !f.trim().is_empty() && f.as_str() != "All")
            .collect();

        if !active_folders.is_empty() {
            let mut folder_conditions: Vec<String> = Vec::new();
            for folder in active_folders {
                let folder_prefix = format!("{folder}%");
                param_values.push(folder_prefix);
                folder_conditions.push(format!("path LIKE ?{}", param_values.len()));
            }
            where_conditions.push(format!("({})", folder_conditions.join(" OR ")));
        }

        let query = if where_conditions.is_empty() {
            "SELECT id, name, path, codec, width, height, duration, has_subtitles, created_at FROM Video ORDER BY path ASC".to_string()
        } else {
            format!(
                "SELECT id, name, path, codec, width, height, duration, has_subtitles, created_at FROM Video WHERE {} ORDER BY path ASC",
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
                codec: row.get(3)?,
                width: row.get(4)?,
                height: row.get(5)?,
                duration: row.get(6)?,
                has_subtitles: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
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
        let existing_set: std::collections::HashSet<&str> = existing_paths.iter().map(|s| s.as_str()).collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_operations() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.get_video_count().unwrap(), 0);

        let video = VideoRecord {
            id: 0,
            name: "Ambient Video 1".to_string(),
            path: "/media/ambient1.mp4".to_string(),
            codec: "hevc-10bit".to_string(),
            width: 3840,
            height: 2160,
            duration: 120.5,
            has_subtitles: true,
            created_at: 1000,
        };

        db.insert_or_update_video(&video).unwrap();
        assert_eq!(db.get_video_count().unwrap(), 1);

        let search_results = db.search_videos("ambient", &[]).unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].name, "Ambient Video 1");

        let subs = db.check_video_subs("/media/ambient1.mp4").unwrap();
        assert!(subs);

        let mut db = db;
        let pruned = db.prune_missing_videos(&[]).unwrap();
        assert_eq!(pruned, 1);
        assert_eq!(db.get_video_count().unwrap(), 0);
    }
}
