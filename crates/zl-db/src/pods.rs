//! Database persistence for Pods, trusted signers, and crash health records.

use rusqlite::params;

use crate::db::Db;
use crate::error::DbError;

/// Operational state of an installed Pod.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodState {
    /// Pod is active and loaded at startup.
    Active,
    /// Pod was explicitly disabled by the user.
    DisabledByUser,
    /// Pod was disabled automatically after exceeding crash threshold.
    DisabledByCrashes,
    /// Pod's API version is incompatible with current launcher API.
    Incompatible,
}

impl PodState {
    /// Database string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            PodState::Active => "active",
            PodState::DisabledByUser => "disabled_by_user",
            PodState::DisabledByCrashes => "disabled_by_crashes",
            PodState::Incompatible => "incompatible",
        }
    }

    /// Parses string representation from database.
    pub fn parse(text: &str) -> Result<PodState, DbError> {
        match text {
            "active" => Ok(PodState::Active),
            "disabled_by_user" => Ok(PodState::DisabledByUser),
            "disabled_by_crashes" => Ok(PodState::DisabledByCrashes),
            "incompatible" => Ok(PodState::Incompatible),
            other => Err(DbError::InvalidValue(format!("invalid pod state: {other}"))),
        }
    }
}

/// A Pod installed in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPod {
    /// Stable identifier (e.g. `sample.hello`).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// API version required by the Pod.
    pub api: u32,
    /// Author name.
    pub author_name: String,
    /// Signer public key fingerprint (64 lowercase hex digits).
    pub signer: String,
    /// Fully qualified entry class name.
    pub entry: String,
    /// Optional short description.
    pub description: Option<String>,
    /// Slots declared by the Pod.
    pub slots: Vec<String>,
    /// Permissions requested by the Pod.
    pub permissions: Vec<String>,
    /// ABIs supported if native libraries are present.
    pub abis: Option<Vec<String>>,
    /// Optional project homepage.
    pub homepage: Option<String>,
    /// Operational state.
    pub state: PodState,
    /// Unix timestamp when first installed.
    pub installed_at: i64,
    /// Unix timestamp when last updated.
    pub updated_at: i64,
}

impl Db {
    /// Inserts or updates an installed Pod record.
    pub fn save_pod(&self, pod: &InstalledPod) -> Result<(), DbError> {
        let slots_json =
            serde_json::to_string(&pod.slots).map_err(|e| DbError::InvalidValue(e.to_string()))?;
        let perms_json = serde_json::to_string(&pod.permissions)
            .map_err(|e| DbError::InvalidValue(e.to_string()))?;
        let abis_json = pod
            .abis
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| DbError::InvalidValue(e.to_string()))?;

        self.conn.execute(
            "INSERT INTO pods (
                id, name, version, api, author_name, signer, entry, description,
                slots, permissions, abis, homepage, state, installed_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                version = excluded.version,
                api = excluded.api,
                author_name = excluded.author_name,
                signer = excluded.signer,
                entry = excluded.entry,
                description = excluded.description,
                slots = excluded.slots,
                permissions = excluded.permissions,
                abis = excluded.abis,
                homepage = excluded.homepage,
                state = excluded.state,
                updated_at = excluded.updated_at",
            params![
                pod.id,
                pod.name,
                pod.version,
                pod.api,
                pod.author_name,
                pod.signer,
                pod.entry,
                pod.description,
                slots_json,
                perms_json,
                abis_json,
                pod.homepage,
                pod.state.as_str(),
                pod.installed_at,
                pod.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Reads an installed Pod by ID.
    pub fn get_pod(&self, id: &str) -> Result<Option<InstalledPod>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, api, author_name, signer, entry, description,
                    slots, permissions, abis, homepage, state, installed_at, updated_at
             FROM pods WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let slots_str: String = row.get(8)?;
            let perms_str: String = row.get(9)?;
            let abis_str: Option<String> = row.get(10)?;
            let state_str: String = row.get(12)?;

            let slots = serde_json::from_str(&slots_str)
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;
            let permissions = serde_json::from_str(&perms_str)
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;
            let abis = abis_str
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;

            Ok(Some(InstalledPod {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                api: row.get(3)?,
                author_name: row.get(4)?,
                signer: row.get(5)?,
                entry: row.get(6)?,
                description: row.get(7)?,
                slots,
                permissions,
                abis,
                homepage: row.get(11)?,
                state: PodState::parse(&state_str)?,
                installed_at: row.get(13)?,
                updated_at: row.get(14)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists all installed Pods ordered by installation time.
    pub fn list_pods(&self) -> Result<Vec<InstalledPod>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, api, author_name, signer, entry, description,
                    slots, permissions, abis, homepage, state, installed_at, updated_at
             FROM pods ORDER BY installed_at ASC, id ASC",
        )?;
        let mut rows = stmt.query([])?;
        let mut pods = Vec::new();
        while let Some(row) = rows.next()? {
            let slots_str: String = row.get(8)?;
            let perms_str: String = row.get(9)?;
            let abis_str: Option<String> = row.get(10)?;
            let state_str: String = row.get(12)?;

            let slots = serde_json::from_str(&slots_str)
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;
            let permissions = serde_json::from_str(&perms_str)
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;
            let abis = abis_str
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|e| DbError::InvalidValue(e.to_string()))?;

            pods.push(InstalledPod {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                api: row.get(3)?,
                author_name: row.get(4)?,
                signer: row.get(5)?,
                entry: row.get(6)?,
                description: row.get(7)?,
                slots,
                permissions,
                abis,
                homepage: row.get(11)?,
                state: PodState::parse(&state_str)?,
                installed_at: row.get(13)?,
                updated_at: row.get(14)?,
            });
        }
        Ok(pods)
    }

    /// Deletes a Pod record, along with its trust pinning and health logs.
    pub fn delete_pod(&self, id: &str) -> Result<bool, DbError> {
        self.conn
            .execute("DELETE FROM pod_trust WHERE id = ?1", params![id])?;
        self.conn
            .execute("DELETE FROM pod_health WHERE pod_id = ?1", params![id])?;
        let count = self
            .conn
            .execute("DELETE FROM pods WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    /// Updates the operational state of an installed Pod.
    pub fn set_pod_state(&self, id: &str, state: PodState) -> Result<bool, DbError> {
        let count = self.conn.execute(
            "UPDATE pods SET state = ?1, updated_at = unixepoch() WHERE id = ?2",
            params![state.as_str(), id],
        )?;
        Ok(count > 0)
    }

    /// Retrieves the pinned signer fingerprint for a Pod ID, if established.
    pub fn get_pod_trust(&self, id: &str) -> Result<Option<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT signer FROM pod_trust WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Records or updates the pinned trusted signer fingerprint for a Pod ID.
    pub fn save_pod_trust(&self, id: &str, signer: &str, trusted_at: i64) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO pod_trust (id, signer, trusted_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET signer = excluded.signer, trusted_at = excluded.trusted_at",
            params![id, signer, trusted_at],
        )?;
        Ok(())
    }

    /// Removes trust pinning for a Pod ID (e.g. upon uninstallation).
    pub fn delete_pod_trust(&self, id: &str) -> Result<bool, DbError> {
        let count = self
            .conn
            .execute("DELETE FROM pod_trust WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    /// Records a crash event for a Pod in the health log.
    pub fn record_pod_crash(
        &self,
        pod_id: &str,
        crashed_at: i64,
        reason: Option<&str>,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO pod_health (pod_id, crashed_at, reason) VALUES (?1, ?2, ?3)",
            params![pod_id, crashed_at, reason],
        )?;
        Ok(())
    }

    /// Counts crashes for a Pod occurring at or after the given `since` Unix timestamp.
    pub fn count_recent_pod_crashes(&self, pod_id: &str, since: i64) -> Result<u32, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pod_health WHERE pod_id = ?1 AND crashed_at >= ?2",
            params![pod_id, since],
            |row| row.get(0),
        )?;
        Ok(count as u32)
    }

    /// Clears crash health records for a Pod (e.g. when re-enabled by the user).
    pub fn clear_pod_crashes(&self, pod_id: &str) -> Result<(), DbError> {
        self.conn
            .execute("DELETE FROM pod_health WHERE pod_id = ?1", params![pod_id])?;
        Ok(())
    }
}
