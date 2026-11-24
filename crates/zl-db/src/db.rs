//! The connection wrapper: opening, pragmas and transactions.

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

use crate::error::DbError;
use crate::migrations;

/// An open, migrated database. Methods are spread over the topic modules (roots, games,
/// emulators); all take `&self`, and [`Db::transaction`] groups writes.
pub struct Db {
    /// The underlying connection.
    pub(crate) conn: Connection,
}

impl Db {
    /// Opens (creating if needed) the database file, enables WAL and foreign keys, and migrates.
    pub fn open(path: &Path) -> Result<Db, DbError> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_millis(zl_config::db::BUSY_TIMEOUT_MS))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Db::finish(conn)
    }

    /// Opens a throwaway in-memory database (tests).
    pub fn open_in_memory() -> Result<Db, DbError> {
        Db::finish(Connection::open_in_memory()?)
    }

    /// Common setup after the connection exists.
    fn finish(conn: Connection) -> Result<Db, DbError> {
        conn.pragma_update(None, "foreign_keys", true)?;
        migrations::migrate(&conn)?;
        Ok(Db { conn })
    }

    /// Runs `body` inside one transaction: committed when it returns `Ok`, rolled back otherwise.
    /// Transactions do not nest.
    pub fn transaction<T, E: From<DbError>>(
        &self,
        body: impl FnOnce(&Db) -> Result<T, E>,
    ) -> Result<T, E> {
        let tx = self.conn.unchecked_transaction().map_err(DbError::from)?;
        let value = body(self)?;
        tx.commit().map_err(DbError::from)?;
        Ok(value)
    }

    /// Schema version of the open database.
    pub fn schema_version(&self) -> Result<u32, DbError> {
        migrations::current_version(&self.conn)
    }

    /// Starts a scan: increments and returns the scan counter stored in `meta`.
    pub fn begin_scan(&self) -> Result<i64, DbError> {
        Ok(self.conn.query_row(
            "INSERT INTO meta (key, value) VALUES ('scan_seq', 1)
             ON CONFLICT(key) DO UPDATE SET value = value + 1 RETURNING value",
            [],
            |row| row.get(0),
        )?)
    }
}
