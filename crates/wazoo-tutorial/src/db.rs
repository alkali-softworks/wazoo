/*!
 * ==============================================================================
 * TUTORIAL LESSON: Embedded SQLite Persistence with `rusqlite`
 * ==============================================================================
 *
 * In desktop applications (including `wazoo-rs`), you often need to persist data
 * locally without running an external database server like PostgreSQL or MySQL.
 *
 * In `wazoo-rs`, `wazoo-core` uses SQLite (via the `rusqlite` crate with "bundled"
 * SQLite C library) to store library metadata, playback history, and bookmarks.
 *
 * In this module, you will learn:
 * 1. How to open or create an SQLite database in Rust.
 * 2. How to run migrations / schema creation (`CREATE TABLE IF NOT EXISTS`).
 * 3. Parameterized queries (`params![...]`) to avoid SQL injection.
 * 4. Mapping raw database rows into strongly-typed Rust structs.
 * 5. Rust's `Result<T, E>` pattern for elegant error handling.
 */

use rusqlite::{params, Connection, Result};
use std::path::Path;

/// A strongly-typed Rust struct representing a note stored in SQLite.
///
/// Note the `derive` attributes:
/// - `Debug`: Allows formatting with `println!("{:?}", note)` or `format!("{:?}", note)`.
/// - `Clone`: Allows duplicating the struct data in memory (e.g. `note.clone()`).
/// - `PartialEq`: Allows equality checks like `note1 == note2`.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteRecord {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

/// The database manager struct.
/// Wrapping the `rusqlite::Connection` in our own struct is a common Rust pattern
/// called "Encapsulation". It lets us expose clean, domain-specific methods (like
/// `add_note` or `get_notes`) instead of exposing raw SQL to the rest of the app.
pub struct TutorialDatabase {
    conn: Connection,
}

impl TutorialDatabase {
    /// Opens an in-memory SQLite database.
    /// In-memory databases (`:memory:`) exist only in RAM and are discarded
    /// when the connection closes. This is perfect for unit tests and tutorials!
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Opens or creates a file-backed SQLite database at the given path.
    /// In `wazoo-rs`, this is typically placed in the OS app-data directory
    /// (e.g. `~/.local/share/wazoo/` on Linux or `%APPDATA%\wazoo` on Windows).
    #[allow(dead_code)]
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initializes the database schema.
    /// Notice the `?` operator at the end of `execute`.
    ///
    /// RUST CONCEPT: The `?` Operator
    /// In Rust, functions that can fail return `Result<T, E>`.
    /// The `?` operator is syntactic sugar that says:
    /// "If this operation returned `Ok(value)`, unwrap it and keep going.
    ///  If it returned `Err(error)`, immediately return `Err(error)` from this function."
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Inserts a new note into SQLite.
    ///
    /// RUST CONCEPT: Borrowing (`&str`)
    /// Notice we take `&str` (string slices) instead of `String`.
    /// `&str` is a borrowed reference to string data. It avoids allocating a new
    /// heap string just to pass text into this function!
    pub fn add_note(&self, title: &str, content: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (title, content) VALUES (?1, ?2)",
            params![title, content],
        )?;

        // Return the auto-generated primary key ID
        Ok(self.conn.last_insert_rowid())
    }

    /// Fetches all notes ordered by most recently created.
    ///
    /// RUST CONCEPT: Iterator Mapping
    /// We use `query_map` to convert each database row into a `NoteRecord`.
    /// SQLite data types map naturally to Rust types:
    /// - `INTEGER` -> `i64`
    /// - `TEXT`    -> `String`
    pub fn get_all_notes(&self) -> Result<Vec<NoteRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, content, created_at FROM notes ORDER BY id DESC")?;

        let note_iter = stmt.query_map([], |row| {
            Ok(NoteRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        // Collect all items from the iterator into a Vec<NoteRecord>.
        // Notice `collect()` handles turning an iterator of `Result<NoteRecord>`
        // into a single `Result<Vec<NoteRecord>>`!
        let mut notes = Vec::new();
        for note in note_iter {
            notes.push(note?);
        }

        Ok(notes)
    }

    /// Deletes a note by its primary key ID.
    /// Returns the number of rows affected (should be 1 if found, 0 if not).
    pub fn delete_note(&self, id: i64) -> Result<usize> {
        self.conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])
    }

    /// Returns the total count of notes.
    #[allow(dead_code)]
    pub fn count_notes(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_db_crud() {
        // Create an in-memory database
        let db = TutorialDatabase::open_in_memory().expect("Failed to create in-memory database");

        // Assert initial count is 0
        assert_eq!(db.count_notes().unwrap(), 0);

        // Add a note
        let id1 = db
            .add_note("Learn Rust", "Master ownership and borrowing")
            .unwrap();
        assert!(id1 > 0);

        // Add a second note
        let _id2 = db
            .add_note("Learn Iced", "Understand The Elm Architecture")
            .unwrap();
        assert_eq!(db.count_notes().unwrap(), 2);

        // Fetch notes and verify content
        let notes = db.get_all_notes().unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Learn Iced"); // Most recent first
        assert_eq!(notes[1].title, "Learn Rust");

        // Delete note 1
        let deleted = db.delete_note(id1).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(db.count_notes().unwrap(), 1);
    }
}
