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

/// Step 3: per-game media (covers, heroes, logos, screenshots, videos). One row per file found
/// or chosen; `edit` is the number of a Cocoon user edit (NULL for a plain file); `protected`
/// rows are the user's choices and are never touched by an import.
const STEP_3: &str = "
CREATE TABLE game_media (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('icon','hero','logo','screenshot','video','3dbox',
        'titlescreen','backcover','miximage','physicalmedia','manual')),
    path TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('cocoon','esde','user')),
    edit INTEGER,
    protected INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (game_id, kind, path)
) WITHOUT ROWID;
";

/// Step 4: extended game metadata (synopsis, release date, developer, publisher, genres, players,
/// rating), protected user overrides, and ROM hashes cache.
const STEP_4: &str = "
CREATE TABLE game_meta (
    game_id INTEGER PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
    description TEXT,
    release_date TEXT,
    developer TEXT,
    publisher TEXT,
    genres TEXT,
    players INTEGER,
    rating REAL,
    source TEXT NOT NULL,
    scraped_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE TABLE game_meta_overrides (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    field TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (game_id, field)
) WITHOUT ROWID;
";

/// Step 5: media downloaded from SteamGridDB (`sgdb`). SQLite cannot change a CHECK, so the table
/// is rebuilt with the same rows.
const STEP_5: &str = "
CREATE TABLE game_media_new (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('icon','hero','logo','screenshot','video','3dbox',
        'titlescreen','backcover','miximage','physicalmedia','manual')),
    path TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('cocoon','esde','user','sgdb')),
    edit INTEGER,
    protected INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (game_id, kind, path)
) WITHOUT ROWID;
INSERT INTO game_media_new SELECT game_id, kind, path, source, edit, protected FROM game_media;
DROP TABLE game_media;
ALTER TABLE game_media_new RENAME TO game_media;
";

/// Step 6: screenshots downloaded from the Steam store (`steam`). The table is rebuilt again with
/// the same rows, as in step 5.
const STEP_6: &str = "
CREATE TABLE game_media_new (
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('icon','hero','logo','screenshot','video','3dbox',
        'titlescreen','backcover','miximage','physicalmedia','manual')),
    path TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('cocoon','esde','user','sgdb','steam')),
    edit INTEGER,
    protected INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (game_id, kind, path)
) WITHOUT ROWID;
INSERT INTO game_media_new SELECT game_id, kind, path, source, edit, protected FROM game_media;
DROP TABLE game_media;
ALTER TABLE game_media_new RENAME TO game_media;
";

/// Step 7: the user's flags on a game (favourite, hidden) and the play sessions. A session is
/// time measured on the device (`usage`, from the system's app usage events; `estimated`, from
/// the launch to the return when those events are not readable) or the total Steam reports for
/// the account (`steam`, one row per game, replaced on every import).
const STEP_7: &str = "
CREATE TABLE game_flags (
    game_id INTEGER PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
    favorite INTEGER NOT NULL DEFAULT 0,
    hidden INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE game_sessions (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NOT NULL,
    seconds INTEGER NOT NULL CHECK (seconds >= 0),
    source TEXT NOT NULL CHECK (source IN ('usage','estimated','steam'))
);
CREATE INDEX game_sessions_game ON game_sessions(game_id, ended_at);
";

/// All steps in order; step `n` is `MIGRATIONS[n - 1]`.
pub const MIGRATIONS: &[&str] = &[STEP_1, STEP_2, STEP_3, STEP_4, STEP_5, STEP_6, STEP_7];

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
