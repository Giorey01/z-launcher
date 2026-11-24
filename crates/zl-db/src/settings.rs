//! Repository: the small key/value table of app settings.

use rusqlite::{params, OptionalExtension};

use crate::db::Db;
use crate::error::DbError;

impl Db {
    /// Reads a setting; `None` when it was never written.
    pub fn setting(&self, key: &str) -> Result<Option<String>, DbError> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Writes a setting, replacing the previous value.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
