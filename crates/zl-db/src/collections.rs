//! Repository: manual folders (collections) and the games in them.

use std::collections::HashMap;

use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;

/// A manual folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// Row id.
    pub id: i64,
    /// Name the user gave it.
    pub name: String,
}

impl Db {
    /// Creates a folder and returns its id. A name already used (any case) is a database error.
    pub fn create_collection(&self, name: &str) -> Result<i64, DbError> {
        Ok(self.conn.query_row(
            "INSERT INTO collections (name) VALUES (?1) RETURNING id",
            params![name],
            |row| row.get(0),
        )?)
    }

    /// Renames a folder; `false` when there is no such folder.
    pub fn rename_collection(&self, id: i64, name: &str) -> Result<bool, DbError> {
        let n = self.conn.execute(
            "UPDATE collections SET name = ?2 WHERE id = ?1",
            params![id, name],
        )?;
        Ok(n > 0)
    }

    /// Deletes a folder and its memberships (the games stay); `false` when there is none.
    pub fn delete_collection(&self, id: i64) -> Result<bool, DbError> {
        let n = self
            .conn
            .execute("DELETE FROM collections WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    /// Every folder, in creation order.
    pub fn list_collections(&self) -> Result<Vec<Collection>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM collections ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Ids of the games of a folder in the user's order (then by id).
    pub fn collection_game_ids(&self, id: i64) -> Result<Vec<i64>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT game_id FROM collection_games WHERE collection_id = ?1
             ORDER BY position, game_id",
        )?;
        let rows = stmt.query_map(params![id], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Adds games at the end of a folder, keeping the place of those already in it. A game id
    /// that does not exist is a database error (foreign key).
    pub fn add_to_collection(&self, id: i64, game_ids: &[i64]) -> Result<(), DbError> {
        let mut next: i64 = self.conn.query_row(
            "SELECT coalesce(max(position) + 1, 0) FROM collection_games WHERE collection_id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let mut stmt = self.conn.prepare_cached(
            "INSERT OR IGNORE INTO collection_games (collection_id, game_id, position)
             VALUES (?1, ?2, ?3)",
        )?;
        for game in game_ids {
            next += i64::try_from(stmt.execute(params![id, game, next])?).unwrap_or(0);
        }
        Ok(())
    }

    /// Takes games out of a folder.
    pub fn remove_from_collection(&self, id: i64, game_ids: &[i64]) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare_cached(
            "DELETE FROM collection_games WHERE collection_id = ?1 AND game_id = ?2",
        )?;
        for game in game_ids {
            stmt.execute(params![id, game])?;
        }
        Ok(())
    }

    /// Rewrites the order of a folder: the games of `ordered` take places 0, 1, 2, ...
    pub fn set_collection_order(&self, id: i64, ordered: &[i64]) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare_cached(
            "UPDATE collection_games SET position = ?3 WHERE collection_id = ?1 AND game_id = ?2",
        )?;
        for (place, game) in (0_i64..).zip(ordered) {
            stmt.execute(params![id, game, place])?;
        }
        Ok(())
    }

    /// The folders a game is in.
    pub fn collections_of_game(&self, game_id: i64) -> Result<Vec<i64>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT collection_id FROM collection_games WHERE game_id = ?1 ORDER BY collection_id",
        )?;
        let rows = stmt.query_map(params![game_id], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Games per folder, not counting missing games; folders with none are absent.
    pub fn collection_counts(&self) -> Result<HashMap<i64, i64>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT cg.collection_id, count(*) FROM collection_games cg
             JOIN games g ON g.id = cg.game_id WHERE g.missing = 0 GROUP BY cg.collection_id",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}
