/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * SQLite Database Manager
 *
 * Manages SQLite connection lifecycle and schema migrations, delegating video inventory,
 * metadata persistence, search query building, and Plex Explorer queries to submodules.
 */

mod metadata;
mod plex_explorer;
mod search;
mod videos;

use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    pub(super) conn: Connection,
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

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS meta_series_idx ON VideoMetadata(series_title)",
            [],
        )?;

        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM Video", [])?;
        Ok(())
    }
}
