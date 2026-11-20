/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Video Inventory CRUD & Folder Pruning
 *
 * Handles video record inserts, Misc virtual folder management, folder removal,
 * and pruning of missing filesystem paths.
 */

use super::Database;
use crate::models::VideoRecord;
use rusqlite::{Result, params};
use std::collections::HashSet;
use std::path::Path;

impl Database {
    pub(super) fn map_video_row(row: &rusqlite::Row<'_>) -> Result<VideoRecord> {
        Ok(VideoRecord {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            folder: row.get(3)?,
        })
    }

    fn delete_video_ids(&mut self, ids: &[i64]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM Video WHERE id = ?1")?;
            for id in ids {
                stmt.execute(params![id])?;
            }
        }
        tx.commit()?;
        Ok(ids.len())
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
        self.conn
            .execute("DELETE FROM Video WHERE folder = 'Misc'", [])
    }

    pub fn get_misc_video_count(&self) -> Result<usize> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM Video WHERE folder = 'Misc'",
            [],
            |row| row.get(0),
        )
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
        let stale_ids: Vec<i64> = all_db
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
        self.delete_video_ids(&stale_ids)
    }

    pub fn delete_missing_paths(&mut self, existing_paths: &[String]) -> Result<usize> {
        self.prune_missing_videos(existing_paths)
    }

    pub fn remove_videos_in_folder(&mut self, folder: &str) -> Result<usize> {
        if folder == "Misc" {
            return self.clear_misc_videos();
        }

        let folder_clean = folder.trim_end_matches(['/', '\\']);
        let folder_prefix_slash = format!("{folder_clean}/");
        let folder_prefix_backslash = format!("{folder_clean}\\");
        let folder_path = Path::new(folder_clean);
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
                let p = Path::new(&v.path);
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

        self.delete_video_ids(&matching_ids)
    }

    pub fn get_video_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM Video", [], |row| row.get(0))
    }

    pub fn get_all_videos(&self) -> Result<Vec<VideoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, folder
             FROM Video ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], Self::map_video_row)?;
        let mut videos = Vec::new();
        for r in rows {
            videos.push(r?);
        }
        Ok(videos)
    }
}
