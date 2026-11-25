//! Cryptographic trust store and key pinning (`docs/spec/pods.md` §4).

use std::time::{SystemTime, UNIX_EPOCH};
use zl_db::Db;

use crate::error::PodError;

/// Manages pinned cryptographic signers for Pods.
pub struct TrustStore<'a> {
    /// Underlying database handle.
    db: &'a Db,
}

impl<'a> TrustStore<'a> {
    /// Creates a new trust store wrapping the database.
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Retrieves the established trusted signer fingerprint for a Pod ID.
    pub fn get_trusted_signer(&self, pod_id: &str) -> Result<Option<String>, PodError> {
        Ok(self.db.get_pod_trust(pod_id)?)
    }

    /// Pins a trusted signer fingerprint for a Pod ID.
    pub fn pin_trust(&self, pod_id: &str, signer: &str) -> Result<(), PodError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.db.save_pod_trust(pod_id, signer, now)?;
        Ok(())
    }

    /// Clears the trusted signer for a Pod ID (e.g. upon uninstallation).
    pub fn clear_trust(&self, pod_id: &str) -> Result<bool, PodError> {
        Ok(self.db.delete_pod_trust(pod_id)?)
    }
}
