//! Repository: games, their member files and tags.

use rusqlite::{params, Row};

use crate::db::Db;
use crate::error::DbError;
use crate::models::{
    Counts, ExistingGame, Game, GameFile, GameKind, GameQuery, NewGame, PlatformCount,
};

/// Columns of [`Game`] in the order [`game_from_row`] reads them.
const GAME_COLUMNS: &str = "id, root_id, platform_id, title, path, kind, size, mtime, missing, \
                            added_at, last_seen_scan";

/// Maps a row selected with [`GAME_COLUMNS`] to a [`Game`].
fn game_from_row(row: &Row<'_>) -> Result<Game, DbError> {
    let kind: String = row.get(5)?;
    Ok(Game {
        id: row.get(0)?,
        root_id: row.get(1)?,
        platform_id: row.get(2)?,
        title: row.get(3)?,
        path: row.get(4)?,
        kind: GameKind::parse(&kind)?,
        size: row.get(6)?,
        mtime_ms: row.get(7)?,
        missing: row.get(8)?,
        added_at: row.get(9)?,
        last_seen_scan: row.get(10)?,
    })
}

impl Db {
    /// Inserts a game or updates the one with the same primary path (keeping `added_at`),
    /// clears its missing flag and replaces its member files and tags. Returns the row id.
    pub fn upsert_game(&self, game: &NewGame, scan_id: i64) -> Result<i64, DbError> {
        self.ensure_platform(&game.platform_id)?;
        let id: i64 = self.conn.query_row(
            "INSERT INTO games (root_id, platform_id, title, path, kind, size, mtime, fingerprint,
                                missing, added_at, last_seen_scan)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, unixepoch(), ?9)
             ON CONFLICT(path) DO UPDATE SET root_id = excluded.root_id,
                 platform_id = excluded.platform_id, title = excluded.title,
                 kind = excluded.kind, size = excluded.size, mtime = excluded.mtime,
                 fingerprint = excluded.fingerprint, missing = 0,
                 last_seen_scan = excluded.last_seen_scan
             RETURNING id",
            params![
                game.root_id,
                game.platform_id,
                game.title,
                game.path,
                game.kind.as_str(),
                game.size,
                game.mtime_ms,
                game.fingerprint,
                scan_id
            ],
            |row| row.get(0),
        )?;
        self.replace_members(id, game)?;
        Ok(id)
    }

    /// Replaces the member files and tags of a game.
    fn replace_members(&self, id: i64, game: &NewGame) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM game_files WHERE game_id = ?1", params![id])?;
        self.conn
            .execute("DELETE FROM game_tags WHERE game_id = ?1", params![id])?;
        let mut files = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO game_files (game_id, path, size, mtime) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for file in &game.files {
            files.execute(params![id, file.path, file.size, file.mtime_ms])?;
        }
        let mut tags = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO game_tags (game_id, key, value) VALUES (?1, ?2, ?3)",
        )?;
        for (key, value) in &game.tags {
            tags.execute(params![id, key, value])?;
        }
        Ok(())
    }

    /// Marks games as seen by a scan without rewriting them: clears the missing flag and
    /// stores the scan id.
    pub fn touch_games(&self, ids: &[i64], scan_id: i64) -> Result<(), DbError> {
        let mut stmt = self
            .conn
            .prepare_cached("UPDATE games SET missing = 0, last_seen_scan = ?2 WHERE id = ?1")?;
        for id in ids {
            stmt.execute(params![id, scan_id])?;
        }
        Ok(())
    }

    /// Flags games as missing. Rows stay, with their tags and emulator overrides.
    pub fn mark_missing(&self, ids: &[i64]) -> Result<(), DbError> {
        let mut stmt = self
            .conn
            .prepare_cached("UPDATE games SET missing = 1 WHERE id = ?1")?;
        for id in ids {
            stmt.execute(params![id])?;
        }
        Ok(())
    }

    /// Loads, keyed by primary path, what a rescan of one root needs to compare.
    pub fn existing_games(&self, root_id: i64) -> Result<Vec<(String, ExistingGame)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT path, id, platform_id, fingerprint, missing FROM games WHERE root_id = ?1",
        )?;
        let rows = stmt.query_map(params![root_id], |row| {
            Ok((
                row.get(0)?,
                ExistingGame {
                    id: row.get(1)?,
                    platform_id: row.get(2)?,
                    fingerprint: row.get(3)?,
                    missing: row.get(4)?,
                },
            ))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Returns games matching the filter, ordered by title (case-insensitive) then path.
    pub fn query_games(&self, query: &GameQuery) -> Result<Vec<Game>, DbError> {
        let sql = format!(
            "SELECT {GAME_COLUMNS} FROM games
             WHERE (?1 IS NULL OR platform_id = ?1)
               AND (?2 IS NULL OR instr(lower(title), lower(?2)) > 0)
               AND (?3 OR missing = 0)
             ORDER BY title COLLATE NOCASE, path LIMIT ?4"
        );
        let limit = query.limit.map_or(-1, i64::from);
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![
            query.platform,
            query.title_contains,
            query.include_missing,
            limit
        ])?;
        let mut games = Vec::new();
        while let Some(row) = rows.next()? {
            games.push(game_from_row(row)?);
        }
        Ok(games)
    }

    /// Games of one platform, missing ones excluded.
    pub fn list_games_by_platform(&self, platform_id: &str) -> Result<Vec<Game>, DbError> {
        self.query_games(&GameQuery {
            platform: Some(platform_id.to_string()),
            ..GameQuery::default()
        })
    }

    /// Ids of the games added at or after `since` (Unix seconds), missing ones excluded.
    pub fn games_added_since(&self, since: i64) -> Result<Vec<i64>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM games WHERE added_at >= ?1 AND missing = 0 ORDER BY id")?;
        let ids = stmt.query_map(params![since], |row| row.get(0))?;
        Ok(ids.collect::<Result<Vec<i64>, _>>()?)
    }

    /// Looks up one game by id.
    pub fn game(&self, game_id: i64) -> Result<Option<Game>, DbError> {
        let sql = format!("SELECT {GAME_COLUMNS} FROM games WHERE id = ?1");
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![game_id])?;
        rows.next()?.map(game_from_row).transpose()
    }

    /// Looks up one game by the path of its primary file.
    pub fn game_by_path(&self, path: &str) -> Result<Option<Game>, DbError> {
        let sql = format!("SELECT {GAME_COLUMNS} FROM games WHERE path = ?1");
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![path])?;
        rows.next()?.map(game_from_row).transpose()
    }

    /// Count of visible games per stored platform id, platforms without games left out.
    pub fn platform_counts(&self) -> Result<Vec<PlatformCount>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT platform_id, count(*) FROM games WHERE missing = 0
             GROUP BY platform_id ORDER BY platform_id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PlatformCount {
                platform_id: row.get(0)?,
                games: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Member files of a game, ordered by path.
    pub fn game_files(&self, game_id: i64) -> Result<Vec<GameFile>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, size, mtime FROM game_files WHERE game_id = ?1 ORDER BY path")?;
        let rows = stmt.query_map(params![game_id], |row| {
            Ok(GameFile {
                path: row.get(0)?,
                size: row.get(1)?,
                mtime_ms: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Tags of a game, ordered by key.
    pub fn game_tags(&self, game_id: i64) -> Result<Vec<(String, String)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM game_tags WHERE game_id = ?1 ORDER BY key")?;
        let rows = stmt.query_map(params![game_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Totals of games, missing games and platforms.
    pub fn counts(&self) -> Result<Counts, DbError> {
        Ok(self.conn.query_row(
            "SELECT (SELECT count(*) FROM games),
                    (SELECT count(*) FROM games WHERE missing = 1),
                    (SELECT count(*) FROM platforms)",
            [],
            |row| {
                Ok(Counts {
                    games: row.get(0)?,
                    missing: row.get(1)?,
                    platforms: row.get(2)?,
                })
            },
        )?)
    }
}
