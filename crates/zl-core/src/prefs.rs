//! Small choices of the Home kept in the database: library mode, onboarding, dock.

use zl_config::app::DOCK_SLOTS;

use crate::error::CoreError;
use crate::types::LibraryMode;
use crate::Service;

/// Setting key of the library mode.
const KEY_MODE: &str = "library_mode";
/// Setting key of the onboarding flag.
const KEY_ONBOARDING: &str = "onboarding_done";
/// Setting key of the dock's packages (comma separated; package names never contain a comma).
const KEY_DOCK: &str = "dock_packages";
/// Setting key of the external pods enabled flag.
const KEY_EXTERNAL_PODS: &str = "external_pods_enabled";

impl Service {
    /// How the Home shows the library; Smart Folders until the user chooses otherwise.
    pub fn library_mode(&self) -> Result<LibraryMode, CoreError> {
        let stored = self.db().setting(KEY_MODE)?;
        Ok(match stored.as_deref() {
            Some("simple") => LibraryMode::Simple,
            _ => LibraryMode::Smart,
        })
    }

    /// Stores the library mode.
    pub fn set_library_mode(&self, mode: LibraryMode) -> Result<(), CoreError> {
        let text = match mode {
            LibraryMode::Smart => "smart",
            LibraryMode::Simple => "simple",
        };
        self.db().set_setting(KEY_MODE, text)?;
        Ok(())
    }

    /// Whether the user finished the onboarding.
    pub fn onboarding_done(&self) -> Result<bool, CoreError> {
        Ok(self.db().setting(KEY_ONBOARDING)?.as_deref() == Some("1"))
    }

    /// Records that the onboarding is finished (or not).
    pub fn set_onboarding_done(&self, done: bool) -> Result<(), CoreError> {
        self.db()
            .set_setting(KEY_ONBOARDING, if done { "1" } else { "0" })?;
        Ok(())
    }

    /// Packages pinned to the dock, in order.
    pub fn dock(&self) -> Result<Vec<String>, CoreError> {
        let stored = self.db().setting(KEY_DOCK)?.unwrap_or_default();
        Ok(stored
            .split(',')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect())
    }

    /// Pins a package to the dock, or unpins it when it is already there. The first dock slot is
    /// the "All apps" button, so the dock holds `DOCK_SLOTS - 1` shortcuts; a full dock is
    /// [`CoreError::DockFull`]. Returns the new list.
    pub fn toggle_dock(&self, package: &str) -> Result<Vec<String>, CoreError> {
        if package.is_empty() || package.contains(',') {
            return Err(CoreError::InvalidInput);
        }
        let mut list = self.dock()?;
        if let Some(at) = list.iter().position(|p| p == package) {
            list.remove(at);
        } else if list.len() + 1 >= DOCK_SLOTS {
            return Err(CoreError::DockFull);
        } else {
            list.push(package.to_string());
        }
        self.db().set_setting(KEY_DOCK, &list.join(","))?;
        Ok(list)
    }

    /// Whether external Pods are enabled (default: false per §6).
    pub fn external_pods_enabled(&self) -> Result<bool, CoreError> {
        Ok(self.db().setting(KEY_EXTERNAL_PODS)?.as_deref() == Some("1"))
    }

    /// Sets whether external Pods are enabled.
    pub fn set_external_pods_enabled(&self, enabled: bool) -> Result<(), CoreError> {
        self.db()
            .set_setting(KEY_EXTERNAL_PODS, if enabled { "1" } else { "0" })?;
        Ok(())
    }
}
