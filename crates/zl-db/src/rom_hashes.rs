//! Repository: the digests of the files of each game (`rom_hashes`).
//!
//! A row is current while its `file_size` and `file_mtime` equal the ones of the file in
//! `game_files`; a rescan that sees a file change therefore makes it pending again without
//! touching this table.

use rusqlite::{params, Row};

use crate::db::Db;
use crate::error::DbError;
use crate::models::{GameFile, NewRomHash, RomHash};

/// The files of `game_files` of games that are present and have no current row, as a SQL
/// fragment from `FROM` on. `{skip}` is replaced by the extension filter.
const PENDING_FROM: &str = "FROM game_files gf JOIN games g ON g.id = gf.game_id
     WHERE g.missing = 0 {skip}
       AND NOT EXISTS (SELECT 1 FROM rom_hashes h
                       WHERE h.game_id = gf.game_id AND h.file_path = gf.path
                         AND h.file_size = gf.size AND h.file_mtime = gf.mtime)";

/// Columns of [`RomHash`] in the order [`hash_from_row`] reads them.
const HASH_COLUMNS: &str = "h.game_id, h.file_path, h.entry, h.file_size, h.file_mtime, \
    h.container, h.size, h.crc32, h.md5, h.sha1, h.header_bytes, h.hl_crc32, h.hl_md5, \
    h.hl_sha1, h.error, h.hashed_at";

/// Maps a row selected with [`HASH_COLUMNS`] to a [`RomHash`].
fn hash_from_row(row: &Row<'_>) -> rusqlite::Result<RomHash> {
    Ok(RomHash {
        game_id: row.get(0)?,
        file_path: row.get(1)?,
        entry: row.get(2)?,
        file_size: row.get(3)?,
        file_mtime: row.get(4)?,
        container: row.get(5)?,
        size: row.get(6)?,
        crc32: row.get(7)?,
        md5: row.get(8)?,
        sha1: row.get(9)?,
        header_bytes: row.get(10)?,
        hl_crc32: row.get(11)?,
        hl_md5: row.get(12)?,
        hl_sha1: row.get(13)?,
        error: row.get(14)?,
        hashed_at: row.get(15)?,
    })
}

/// A SQL `AND ...` clause that leaves out the files with one of `extensions` (lower case, no
/// dot). The extensions are constants of the program, never user text.
fn skip_clause(extensions: &[&str]) -> String {
    extensions
        .iter()
        .filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(|e| format!("AND gf.path NOT LIKE '%.{e}'"))
        .collect::<Vec<_>>()
        .join(" ")
}

impl Db {
    /// The files that still need hashing (no row, or a row of an older size or mtime), grouped
    /// by game in game order, leaving out files with one of the `skip` extensions.
    pub fn files_to_hash(&self, skip: &[&str]) -> Result<Vec<(i64, GameFile)>, DbError> {
        let from = PENDING_FROM.replace("{skip}", &skip_clause(skip));
        let sql = format!(
            "SELECT gf.game_id, gf.path, gf.size, gf.mtime {from} ORDER BY gf.game_id, gf.path"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let file = GameFile {
                path: row.get(1)?,
                size: row.get(2)?,
                mtime_ms: row.get(3)?,
            };
            Ok((row.get(0)?, file))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Games with at least one file to hash, and how many of them have all their files hashed.
    pub fn hash_coverage(&self, skip: &[&str]) -> Result<(u32, u32), DbError> {
        let clause = skip_clause(skip);
        let total: u32 = self.conn.query_row(
            &format!(
                "SELECT count(DISTINCT gf.game_id) FROM game_files gf
                 JOIN games g ON g.id = gf.game_id WHERE g.missing = 0 {clause}"
            ),
            [],
            |row| row.get(0),
        )?;
        let from = PENDING_FROM.replace("{skip}", &clause);
        let pending: u32 = self.conn.query_row(
            &format!("SELECT count(DISTINCT gf.game_id) {from}"),
            [],
            |row| row.get(0),
        )?;
        Ok((total, total.saturating_sub(pending)))
    }

    /// Replaces what is stored for one file of a game with `rows` (one per entry, or a single
    /// row with an error), in one transaction.
    pub fn replace_file_hashes(
        &self,
        game_id: i64,
        file_path: &str,
        rows: &[NewRomHash],
    ) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            db.conn.execute(
                "DELETE FROM rom_hashes WHERE game_id = ?1 AND file_path = ?2",
                params![game_id, file_path],
            )?;
            let mut insert = db.conn.prepare_cached(
                "INSERT INTO rom_hashes (game_id, file_path, entry, file_size, file_mtime,
                     container, size, crc32, md5, sha1, header_bytes, hl_crc32, hl_md5,
                     hl_sha1, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            )?;
            for row in rows {
                insert.execute(params![
                    game_id,
                    file_path,
                    row.entry,
                    row.file_size,
                    row.file_mtime,
                    row.container,
                    row.size,
                    row.crc32,
                    row.md5,
                    row.sha1,
                    row.header_bytes,
                    row.hl_crc32,
                    row.hl_md5,
                    row.hl_sha1,
                    row.error,
                ])?;
            }
            Ok(())
        })
    }

    /// The current digests of a game's files: rows of a file that has changed or is no longer
    /// a member of the game are left out. Ordered by file and entry.
    pub fn game_hashes(&self, game_id: i64) -> Result<Vec<RomHash>, DbError> {
        let sql = format!(
            "SELECT {HASH_COLUMNS} FROM rom_hashes h
             JOIN game_files gf ON gf.game_id = h.game_id AND gf.path = h.file_path
                 AND gf.size = h.file_size AND gf.mtime = h.file_mtime
             WHERE h.game_id = ?1 ORDER BY h.file_path, h.entry"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![game_id], hash_from_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Ids of the games that have a file with this digest (`kind` is `crc32`, `md5` or `sha1`,
    /// `hex` lower case), headerless digests included. Meant for matching metadata by hash.
    pub fn games_with_digest(&self, kind: &str, hex: &str) -> Result<Vec<i64>, DbError> {
        let (full, headerless) = match kind {
            "crc32" => ("crc32", "hl_crc32"),
            "md5" => ("md5", "hl_md5"),
            "sha1" => ("sha1", "hl_sha1"),
            other => return Err(DbError::InvalidValue(other.to_string())),
        };
        let sql = format!(
            "SELECT DISTINCT game_id FROM rom_hashes WHERE {full} = ?1 OR {headerless} = ?1
             ORDER BY game_id"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![hex], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}
