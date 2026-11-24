//! The launcher's SQLite database: connection setup, versioned migrations and a small typed
//! repository. All SQL of the project lives in this crate; callers see Rust types only.
//!
//! Design choices of the week-1 schema (see [`migrations`] for the SQL):
//! - `rom_roots` lists the folders the user added (internal storage, microSD, USB are just
//!   several roots). A root can be disabled without losing its games.
//! - `platforms` records the platform ids seen on disk. The platform *definitions* live in
//!   another crate; the database only needs a stable id to hang games on.
//! - `games` has one row per library entry. The primary `path` is the identity (UNIQUE), so a
//!   rescan updates in place and keeps `added_at`. A game that disappears is flagged `missing`,
//!   never deleted, so favourites, play time and emulator overrides survive a removed SD card.
//!   `fingerprint` is a hash of the game's shape (kind, title, member files with size and mtime)
//!   that lets a rescan skip unchanged games with one integer comparison.
//! - `game_files` holds every member file of a game (discs, tracks, the playlist itself).
//! - `game_tags` holds the `[key] value` pairs of tag files.
//! - `rom_roots.saf_tree` keeps the SAF tree URI the user granted for the folder, and `settings`
//!   is a small key/value table for app choices (library mode, onboarding, dock).
//! - `platform_emulator` and `game_emulator` hold the emulator assignment overrides; a NULL
//!   profile id means "explicitly none", an absent row means "use the default".
//!
//! - `game_media` holds the media files of each game (cover, hero, logo, screenshots, video) with
//!   their source; a `protected` row is the user's choice and imports leave it alone.
//!
//! Timestamps are Unix time, `added_at` in seconds and file modification times in milliseconds.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod db;
mod emulators;
mod error;
mod games;
mod media;
pub mod migrations;
mod models;
mod roots;
mod settings;

pub use db::Db;
pub use error::DbError;
pub use models::{
    Counts, ExistingGame, Game, GameFile, GameKind, GameQuery, MediaKind, MediaRow, MediaSource,
    NewGame, NewMedia, PlatformCount, Root,
};
