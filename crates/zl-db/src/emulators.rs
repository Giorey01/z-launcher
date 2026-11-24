//! Repository: emulator assignment overrides per platform and per game.

use rusqlite::{params, OptionalExtension};

use crate::db::Db;
use crate::error::DbError;

impl Db {
    /// Sets the emulator profile for a platform (`None` = explicitly no emulator).
    pub fn set_platform_emulator(
        &self,
        platform_id: &str,
        profile_id: Option<&str>,
    ) -> Result<(), DbError> {
        self.ensure_platform(platform_id)?;
        self.conn.execute(
            "INSERT INTO platform_emulator (platform_id, profile_id) VALUES (?1, ?2)
             ON CONFLICT(platform_id) DO UPDATE SET profile_id = excluded.profile_id",
            params![platform_id, profile_id],
        )?;
        Ok(())
    }

    /// Returns the override for a platform: `None` = no override, `Some(None)` = explicit none.
    pub fn platform_emulator(&self, platform_id: &str) -> Result<Option<Option<String>>, DbError> {
        Ok(self
            .conn
            .query_row(
                "SELECT profile_id FROM platform_emulator WHERE platform_id = ?1",
                params![platform_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Sets the emulator profile override for one game.
    pub fn set_game_emulator(&self, game_id: i64, profile_id: Option<&str>) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO game_emulator (game_id, profile_id) VALUES (?1, ?2)
             ON CONFLICT(game_id) DO UPDATE SET profile_id = excluded.profile_id",
            params![game_id, profile_id],
        )?;
        Ok(())
    }

    /// Returns the override for a game, with the same meaning as [`Db::platform_emulator`].
    pub fn game_emulator(&self, game_id: i64) -> Result<Option<Option<String>>, DbError> {
        Ok(self
            .conn
            .query_row(
                "SELECT profile_id FROM game_emulator WHERE game_id = ?1",
                params![game_id],
                |row| row.get(0),
            )
            .optional()?)
    }
}
