//! Repository: the media files of games and the choice of the best one per kind.
//!
//! Precedence, best first: a protected row (the user's choice), a Cocoon user edit (the higher
//! number first), a plain Cocoon file, an ES-DE file; ties break on path so the result is stable.

use rusqlite::{params, Row};

use crate::db::Db;
use crate::error::DbError;
use crate::models::{MediaKind, MediaRow, MediaSource, NewMedia};

/// Columns of [`MediaRow`] in the order [`media_from_row`] reads them.
const MEDIA_COLUMNS: &str = "game_id, kind, path, source, edit, protected";

/// SQL ordering that puts the best file of a (game, kind) first.
const BEST_FIRST: &str = "protected DESC,
    CASE WHEN source = 'user' THEN 3
         WHEN source = 'cocoon' AND edit IS NOT NULL THEN 2
         WHEN source = 'cocoon' THEN 1 ELSE 0 END DESC,
    edit DESC, path";

/// Maps a row selected with [`MEDIA_COLUMNS`] to a [`MediaRow`].
fn media_from_row(row: &Row<'_>) -> Result<MediaRow, DbError> {
    let kind: String = row.get(1)?;
    let source: String = row.get(3)?;
    Ok(MediaRow {
        game_id: row.get(0)?,
        kind: MediaKind::parse(&kind)?,
        path: row.get(2)?,
        source: MediaSource::parse(&source)?,
        edit: row.get(4)?,
        protected: row.get(5)?,
    })
}

impl Db {
    /// Replaces what an import found: removes the unprotected rows of `sources`, then stores
    /// `items`. A protected row is never removed, and an item whose (game, kind, path) is already
    /// stored (protected or not) is left as it is. One transaction.
    pub fn replace_media(
        &self,
        sources: &[MediaSource],
        items: &[NewMedia],
    ) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            for source in sources {
                db.conn.execute(
                    "DELETE FROM game_media WHERE source = ?1 AND protected = 0",
                    params![source.as_str()],
                )?;
            }
            let mut stmt = db.conn.prepare_cached(
                "INSERT OR IGNORE INTO game_media (game_id, kind, path, source, edit, protected)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0)",
            )?;
            for m in items {
                stmt.execute(params![
                    m.game_id,
                    m.kind.as_str(),
                    m.path,
                    m.source.as_str(),
                    m.edit
                ])?;
            }
            Ok(())
        })
    }

    /// Sets the user's choice of one kind for a game: the file becomes a protected `user` row and
    /// the previous protected choice of that kind goes away. With `None` the choice is cleared and
    /// the imported media is used again.
    pub fn set_user_media(
        &self,
        game_id: i64,
        kind: MediaKind,
        path: Option<&str>,
    ) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            db.conn.execute(
                "DELETE FROM game_media WHERE game_id = ?1 AND kind = ?2 AND protected = 1",
                params![game_id, kind.as_str()],
            )?;
            if let Some(path) = path {
                db.conn.execute(
                    "INSERT INTO game_media (game_id, kind, path, source, edit, protected)
                     VALUES (?1, ?2, ?3, 'user', NULL, 1)
                     ON CONFLICT(game_id, kind, path)
                     DO UPDATE SET source = 'user', edit = NULL, protected = 1",
                    params![game_id, kind.as_str(), path],
                )?;
            }
            Ok(())
        })
    }

    /// All media of one game, best first within each kind.
    pub fn game_media(&self, game_id: i64) -> Result<Vec<MediaRow>, DbError> {
        let sql = format!(
            "SELECT {MEDIA_COLUMNS} FROM game_media WHERE game_id = ?1 ORDER BY kind, {BEST_FIRST}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![game_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(media_from_row(row)?);
        }
        Ok(out)
    }

    /// The best file of each (game, kind) for the given kinds: one row per pair.
    pub fn best_media(&self, kinds: &[MediaKind]) -> Result<Vec<MediaRow>, DbError> {
        let list = kinds
            .iter()
            .map(|k| format!("'{}'", k.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT {MEDIA_COLUMNS} FROM game_media WHERE kind IN ({list})
             ORDER BY game_id, kind, {BEST_FIRST}"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut out: Vec<MediaRow> = Vec::new();
        while let Some(row) = rows.next()? {
            let item = media_from_row(row)?;
            let same = out
                .last()
                .is_some_and(|l| l.game_id == item.game_id && l.kind == item.kind);
            if !same {
                out.push(item);
            }
        }
        Ok(out)
    }
}
