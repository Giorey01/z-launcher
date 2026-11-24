//! Versioned schema migrations: a numbered list of SQL steps, each applied in its own
//! transaction, with the applied count stored in `PRAGMA user_version`.
//!
//! Rules: never edit a step that has shipped; add a new one at the end.

use rusqlite::Connection;

use crate::error::DbError;

/// Step 1: the library schema (see the crate docs for the reasoning).
const STEP_1: &str = "
CREATE TABLE rom_roots (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    enabled INTEGER NOT NULL DEFAULT 1,
    scan_subfolders INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE platforms (
    id TEXT PRIMARY KEY
);
CREATE TABLE games (
    id INTEGER PRIMARY KEY,
    root_id INTEGER NOT NULL REFERENCES rom_roots(id),
    platform_id TEXT NOT NULL REFERENCES platforms(id),
    title TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('rom','m3u','multidisc','tagfile','folder')),
    size INTEGER NOT NULL,
    mtime INTEGER NOT NULL,
    fingerprint INTEGER NOT NULL,
    missing INTEGER NOT NULL DEFAULT 0,
    added_at INTEGER NOT NULL,
    last_seen_scan INTEGER NOT NULL
);
CREATE INDEX games_platform ON games(platform_id, title);
CREATE INDEX games_root ON games(root_id);
CREATE TABLE game_files (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    mtime INTEGER NOT NULL,
    PRIMARY KEY (game_id, path)
) WITHOUT ROWID;
CREATE TABLE game_tags (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (game_id, key)
) WITHOUT ROWID;
CREATE TABLE platform_emulator (
    platform_id TEXT PRIMARY KEY REFERENCES platforms(id),
    profile_id TEXT
);
CREATE TABLE game_emulator (
    game_id INTEGER PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
    profile_id TEXT
);
CREATE TABLE meta (
    key TEXT PRIMARY KEY,
    value INTEGER NOT NULL
);
";

/// Step 2: the SAF tree the user granted for a root (needed to hand ROMs to emulators as
/// `content://` documents) and a small key/value table for app settings.
const STEP_2: &str = "
ALTER TABLE rom_roots ADD COLUMN saf_tree TEXT;
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// All steps in order; step `n` is `MIGRATIONS[n - 1]`.
pub const MIGRATIONS: &[&str] = &[STEP_1, STEP_2];

/// Reads the schema version of the open database.
pub fn current_version(conn: &Connection) -> Result<u32, DbError> {
    Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

/// Brings the database to the latest schema. Safe to call on every open: already applied steps
/// are skipped. Refuses a database from a newer app version.
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let found = current_version(conn)?;
    let supported = MIGRATIONS.len() as u32;
    if found > supported {
        return Err(DbError::NewerSchema { found, supported });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(found as usize) {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute_batch(&format!("PRAGMA user_version = {}", index + 1))?;
        tx.commit()?;
    }
    Ok(())
}
