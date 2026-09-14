/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Core Library Root
 *
 * Re-exports common domain models, SQLite database management, and persistent configuration
 * services used across all Wazoo engines and client applications.
 */

pub mod config;
pub mod db;
pub mod i18n;
pub mod keybinds;
pub mod models;

pub use config::ConfigManager;
pub use db::Database;
pub use i18n::{Language, t, t_with};
pub use keybinds::{KeyAction, KeybindSettings};
pub use models::*;
