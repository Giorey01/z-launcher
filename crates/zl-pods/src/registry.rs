//! Pod registry and lifecycle coordinator (`docs/spec/pods.md` §6, §10).

use std::fs;
use std::path::{Path, PathBuf};
use zl_db::{Db, InstalledPod, PodState};

use crate::error::PodError;

/// Coordinates installed Pods and handles safe-mode recovery.
pub struct PodRegistry<'a> {
    /// Database handle.
    db: &'a Db,
    /// Base directory where pods are installed.
    pods_dir: PathBuf,
}

impl<'a> PodRegistry<'a> {
    /// Creates a new registry instance.
    pub fn new(db: &'a Db, pods_dir: impl Into<PathBuf>) -> Self {
        Self {
            db,
            pods_dir: pods_dir.into(),
        }
    }

    /// Lists all installed Pods from database.
    pub fn list_installed(&self) -> Result<Vec<InstalledPod>, PodError> {
        Ok(self.db.list_pods()?)
    }

    /// Lists active Pods that should be loaded by the host classloader.
    pub fn list_active(&self) -> Result<Vec<InstalledPod>, PodError> {
        let all = self.list_installed()?;
        Ok(all
            .into_iter()
            .filter(|p| p.state == PodState::Active)
            .collect())
    }

    /// Enables or disables an installed Pod by user action.
    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<bool, PodError> {
        let state = if enabled {
            PodState::Active
        } else {
            PodState::DisabledByUser
        };
        Ok(self.db.set_pod_state(id, state)?)
    }

    /// Uninstalls a Pod by removing its files and clearing database records.
    pub fn uninstall(&self, id: &str) -> Result<bool, PodError> {
        let pod_dir = self.pods_dir.join(id);
        if pod_dir.exists() {
            let _ = fs::remove_dir_all(&pod_dir);
        }
        Ok(self.db.delete_pod(id)?)
    }

    /// Checks the startup sentinel marker for a crash during loading (`docs/spec/pods.md` §10).
    ///
    /// If `marker_file` exists, reads the crashed pod ID, marks it as [`PodState::DisabledByCrashes`],
    /// deletes the marker, and returns the ID of the faulty Pod.
    pub fn check_and_clear_loading_marker(
        &self,
        marker_file: &Path,
    ) -> Result<Option<String>, PodError> {
        if !marker_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(marker_file)?;
        let _ = fs::remove_file(marker_file);

        if let Some(crashed_id) = content.trim().strip_prefix("loading=") {
            let crashed_id = crashed_id.trim();
            if !crashed_id.is_empty() {
                self.db
                    .set_pod_state(crashed_id, PodState::DisabledByCrashes)?;
                return Ok(Some(crashed_id.to_string()));
            }
        }

        Ok(None)
    }

    /// Writes the startup loading marker before attempting to load a Pod.
    pub fn set_loading_marker(&self, marker_file: &Path, pod_id: &str) -> Result<(), PodError> {
        if let Some(parent) = marker_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(marker_file, format!("loading={pod_id}"))?;
        Ok(())
    }

    /// Clears the startup loading marker after successful loading.
    pub fn clear_loading_marker(&self, marker_file: &Path) -> Result<(), PodError> {
        if marker_file.exists() {
            fs::remove_file(marker_file)?;
        }
        Ok(())
    }
}
