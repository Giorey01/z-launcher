//! Repository: game metadata and user overrides.

use std::collections::HashMap;

use rusqlite::{params, Row};

use crate::db::Db;
use crate::error::DbError;
use crate::models::{GameMeta, MetaField, MetaOverride};

/// Columns of `game_meta` in standard order.
const META_COLUMNS: &str = "game_id, description, release_date, developer, publisher, genres, \
                            players, rating, source, scraped_at";

/// Maps a database row to [`GameMeta`].
fn meta_from_row(row: &Row<'_>) -> Result<GameMeta, DbError> {
    Ok(GameMeta {
        game_id: row.get(0)?,
        description: row.get(1)?,
        release_date: row.get(2)?,
        developer: row.get(3)?,
        publisher: row.get(4)?,
        genres: row.get(5)?,
        players: row.get(6)?,
        rating: row.get(7)?,
        source: row.get(8)?,
        scraped_at: row.get(9)?,
    })
}

impl Db {
    /// Loads raw metadata of a game without applying user overrides.
    pub fn raw_game_meta(&self, game_id: i64) -> Result<Option<GameMeta>, DbError> {
        let sql = format!("SELECT {META_COLUMNS} FROM game_meta WHERE game_id = ?1");
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![game_id])?;
        rows.next()?.map(meta_from_row).transpose()
    }

    /// Loads metadata of a game with any protected user overrides merged in.
    pub fn game_meta(&self, game_id: i64) -> Result<Option<GameMeta>, DbError> {
        let mut meta = match self.raw_game_meta(game_id)? {
            Some(m) => m,
            None => return Ok(None),
        };
        let overrides = self.meta_overrides(game_id)?;
        for ov in overrides {
            apply_override(&mut meta, ov.field, &ov.value);
        }
        Ok(Some(meta))
    }

    /// Saves or updates the metadata for a game. If `overwrite_protected` is false,
    /// fields with active overrides are not modified.
    pub fn save_game_meta(
        &self,
        meta: &GameMeta,
        overwrite_protected: bool,
    ) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            if overwrite_protected {
                db.conn.execute(
                    "DELETE FROM game_meta_overrides WHERE game_id = ?1",
                    params![meta.game_id],
                )?;
            }
            db.conn.execute(
                "INSERT INTO game_meta (game_id, description, release_date, developer, publisher,
                                        genres, players, rating, source, scraped_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, unixepoch())
                 ON CONFLICT(game_id) DO UPDATE SET
                     description = excluded.description,
                     release_date = excluded.release_date,
                     developer = excluded.developer,
                     publisher = excluded.publisher,
                     genres = excluded.genres,
                     players = excluded.players,
                     rating = excluded.rating,
                     source = excluded.source,
                     scraped_at = excluded.scraped_at",
                params![
                    meta.game_id,
                    meta.description,
                    meta.release_date,
                    meta.developer,
                    meta.publisher,
                    meta.genres,
                    meta.players,
                    meta.rating,
                    meta.source,
                ],
            )?;
            Ok(())
        })
    }

    /// Sets a user override on a specific field.
    pub fn set_meta_override(
        &self,
        game_id: i64,
        field: MetaField,
        value: &str,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO game_meta_overrides (game_id, field, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(game_id, field) DO UPDATE SET value = excluded.value",
            params![game_id, field.as_str(), value],
        )?;
        Ok(())
    }

    /// Removes a user override for a specific field.
    pub fn clear_meta_override(&self, game_id: i64, field: MetaField) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM game_meta_overrides WHERE game_id = ?1 AND field = ?2",
            params![game_id, field.as_str()],
        )?;
        Ok(())
    }

    /// Returns all user overrides for a game.
    pub fn meta_overrides(&self, game_id: i64) -> Result<Vec<MetaOverride>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT field, value FROM game_meta_overrides WHERE game_id = ?1 ORDER BY field",
        )?;
        let rows = stmt.query_map(params![game_id], |row| {
            let field_str: String = row.get(0)?;
            Ok((field_str, row.get(1)?))
        })?;
        let mut out = Vec::new();
        for item in rows {
            let (f, val): (String, String) = item?;
            if let Ok(field) = MetaField::parse(&f) {
                out.push(MetaOverride {
                    game_id,
                    field,
                    value: val,
                });
            }
        }
        Ok(out)
    }

    /// Returns all overrides for a specific field as a map from game_id to value.
    pub fn all_meta_overrides(&self, field: MetaField) -> Result<HashMap<i64, String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_id, value FROM game_meta_overrides WHERE field = ?1")?;
        let rows = stmt.query_map(params![field.as_str()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut map = HashMap::new();
        for item in rows {
            let (id, val): (i64, String) = item?;
            map.insert(id, val);
        }
        Ok(map)
    }
}

/// Helper function to merge an override into a [`GameMeta`] record.
fn apply_override(meta: &mut GameMeta, field: MetaField, value: &str) {
    match field {
        MetaField::Title => {} // Title override is handled in game title resolution
        MetaField::Description => meta.description = Some(value.to_string()),
        MetaField::ReleaseDate => meta.release_date = Some(value.to_string()),
        MetaField::Developer => meta.developer = Some(value.to_string()),
        MetaField::Publisher => meta.publisher = Some(value.to_string()),
        MetaField::Genres => meta.genres = Some(value.to_string()),
        MetaField::Players => meta.players = value.parse().ok(),
        MetaField::Rating => meta.rating = value.parse().ok(),
    }
}
