/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Core Library Root
 * 
 * Re-exports common domain models, SQLite database management, and persistent configuration
 * services used across all Wazoo engines and client applications.
 */

pub mod models;
pub mod db;
pub mod config;
pub mod i18n;
pub mod keybinds;

pub use models::*;
pub use db::Database;
pub use config::ConfigManager;
pub use i18n::{t, t_with, Language};
pub use keybinds::{KeyAction, KeybindSettings};
