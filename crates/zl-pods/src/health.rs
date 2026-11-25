//! Fault monitoring and health threshold enforcement (`docs/spec/pods.md` §10).

use std::time::{SystemTime, UNIX_EPOCH};
use zl_db::{Db, PodState};

use crate::error::PodError;

/// Tracks crash counts and manages automatic disabling of unstable Pods.
pub struct HealthMonitor<'a> {
    /// Underlying database handle.
    db: &'a Db,
}

impl<'a> HealthMonitor<'a> {
    /// Creates a health monitor instance.
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Records a crash event for a Pod and checks whether it crossed the crash threshold.
    ///
    /// Returns `true` if the Pod was automatically disabled due to exceeding [`zl_config::pods::MAX_CRASHES`].
    pub fn record_crash(&self, pod_id: &str, reason: Option<&str>) -> Result<bool, PodError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.db.record_pod_crash(pod_id, now, reason)?;

        let window_secs = zl_config::pods::CRASH_WINDOW_HOURS * 3600;
        let since = now.saturating_sub(window_secs);
        let recent_crashes = self.db.count_recent_pod_crashes(pod_id, since)?;

        if recent_crashes >= zl_config::pods::MAX_CRASHES {
            self.db.set_pod_state(pod_id, PodState::DisabledByCrashes)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Resets crash history and reactivates a Pod when requested by the user.
    pub fn reset_health(&self, pod_id: &str) -> Result<(), PodError> {
        self.db.clear_pod_crashes(pod_id)?;
        self.db.set_pod_state(pod_id, PodState::Active)?;
        Ok(())
    }
}
