//! The settings screen's view of the library: platforms with their default emulator and the
//! candidates, and removing a ROM folder.

use crate::error::CoreError;
use crate::types::{Candidate, PlatformEmulatorSetting};
use crate::Service;

impl Service {
    /// The platforms of the library with the default emulator set for each and the choices
    /// installed on the device (`packages` as for detection), preferred first.
    pub fn platform_emulators(
        &self,
        packages: &[String],
    ) -> Result<Vec<PlatformEmulatorSetting>, CoreError> {
        let installed = self.platform_db().detect_installed(packages).installed;
        let mut out = Vec::new();
        for platform in self.platforms()? {
            let candidates: Vec<Candidate> = self
                .platform_db()
                .candidates(&platform.id, &installed)
                .into_iter()
                .map(|c| self.candidate(c))
                .collect();
            let current = self
                .platform_default(&platform.id)?
                .map(|p| self.candidate(p));
            out.push(PlatformEmulatorSetting {
                platform_id: platform.id,
                platform_name: platform.name,
                current,
                candidates,
            });
        }
        Ok(out)
    }

    /// Removes a ROM folder and the games found under it from the library (the files on disk
    /// stay). Platform defaults are kept.
    pub fn remove_root(&self, root_id: i64) -> Result<(), CoreError> {
        let db = self.db();
        db.root(root_id)?.ok_or(CoreError::NotFound)?;
        db.delete_root(root_id)?;
        Ok(())
    }
}
