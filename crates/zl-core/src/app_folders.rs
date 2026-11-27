//! The Apps and Emulators Smart Folders (RF-13), made of installed Android apps instead of games.
//!
//! The app side reports the launcher apps installed (package and whether the app declares
//! itself a game). Emulators are the packages the emulator data knows (the same detection the
//! settings use); Apps are the rest that are not games, so a game installed from a store stays
//! in the drawer and in its console, not here.

use std::collections::HashSet;

use crate::error::CoreError;
use crate::smart_folders::{id_of, smart_entry};
use crate::types::{FolderKind, InstalledApp, PlatformEntry};
use crate::Service;

impl Service {
    /// The packages of the installed emulators among `apps`.
    fn emulator_packages(&self, apps: &[InstalledApp]) -> HashSet<String> {
        let packages: Vec<&str> = apps.iter().map(|a| a.package.as_str()).collect();
        self.platforms
            .detect_installed(&packages)
            .installed
            .into_iter()
            .flat_map(|e| e.packages)
            .collect()
    }

    /// The packages of one app folder (`@apps` or `@emulators`) among the installed `apps`, in
    /// the order given. `InvalidInput` for any other folder id.
    pub fn app_folder_packages(
        &self,
        folder: &str,
        apps: &[InstalledApp],
    ) -> Result<Vec<String>, CoreError> {
        let emulators = self.emulator_packages(apps);
        let wanted = |app: &InstalledApp| {
            let is_emulator = emulators.contains(&app.package);
            if folder == id_of(FolderKind::Emulators) {
                is_emulator
            } else {
                !is_emulator && !app.is_game
            }
        };
        if folder != id_of(FolderKind::Emulators) && folder != id_of(FolderKind::Apps) {
            return Err(CoreError::InvalidInput);
        }
        Ok(apps
            .iter()
            .filter(|a| wanted(a))
            .map(|a| a.package.clone())
            .collect())
    }

    /// The Home entries of Apps and Emulators, each only when it has something in it.
    pub fn app_folder_entries(
        &self,
        apps: &[InstalledApp],
    ) -> Result<Vec<PlatformEntry>, CoreError> {
        let mut out = Vec::new();
        for kind in [FolderKind::Apps, FolderKind::Emulators] {
            let n = self.app_folder_packages(id_of(kind), apps)?.len();
            if n > 0 {
                out.push(smart_entry(kind, n));
            }
        }
        Ok(out)
    }
}
