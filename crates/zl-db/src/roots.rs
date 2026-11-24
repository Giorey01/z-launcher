//! Repository: rom roots and platforms.

use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;
use crate::models::Root;

impl Db {
    /// Adds a root or updates its sub-folder option; returns its id. Enabled state is kept.
    pub fn upsert_root(&self, path: &str, scan_subfolders: bool) -> Result<i64, DbError> {
        Ok(self.conn.query_row(
            "INSERT INTO rom_roots (path, scan_subfolders) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET scan_subfolders = excluded.scan_subfolders
             RETURNING id",
            params![path, scan_subfolders],
            |row| row.get(0),
        )?)
    }

    /// Adds a root like [`Db::upsert_root`] and stores (or clears, with `None`) the SAF tree URI
    /// the user granted for it.
    pub fn upsert_root_with_saf(
        &self,
        path: &str,
        scan_subfolders: bool,
        saf_tree: Option<&str>,
    ) -> Result<i64, DbError> {
        let id = self.upsert_root(path, scan_subfolders)?;
        self.conn.execute(
            "UPDATE rom_roots SET saf_tree = ?2 WHERE id = ?1",
            params![id, saf_tree],
        )?;
        Ok(id)
    }

    /// Looks up one root by id.
    pub fn root(&self, root_id: i64) -> Result<Option<Root>, DbError> {
        Ok(self.list_roots()?.into_iter().find(|r| r.id == root_id))
    }

    /// Enables or disables a root.
    pub fn set_root_enabled(&self, root_id: i64, enabled: bool) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE rom_roots SET enabled = ?2 WHERE id = ?1",
            params![root_id, enabled],
        )?;
        Ok(())
    }

    /// Removes a root and every game found under it (their files, tags, emulator overrides and
    /// media go with them). Platform defaults are kept.
    pub fn delete_root(&self, root_id: i64) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            db.conn
                .execute("DELETE FROM games WHERE root_id = ?1", params![root_id])?;
            db.conn
                .execute("DELETE FROM rom_roots WHERE id = ?1", params![root_id])?;
            Ok(())
        })
    }

    /// Lists all roots in insertion order.
    pub fn list_roots(&self) -> Result<Vec<Root>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, enabled, scan_subfolders, saf_tree FROM rom_roots ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Root {
                id: row.get(0)?,
                path: row.get(1)?,
                enabled: row.get(2)?,
                scan_subfolders: row.get(3)?,
                saf_tree: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Records that a platform id exists on disk (idempotent).
    pub fn ensure_platform(&self, platform_id: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO platforms (id) VALUES (?1)",
            params![platform_id],
        )?;
        Ok(())
    }

    /// Lists the platform ids seen, sorted.
    pub fn list_platforms(&self) -> Result<Vec<String>, DbError> {
        let mut stmt = self.conn.prepare("SELECT id FROM platforms ORDER BY id")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}
