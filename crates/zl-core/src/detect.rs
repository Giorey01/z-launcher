//! Emulator detection and the defaults proposed at onboarding.

use std::collections::HashSet;

use zl_platforms::ProfileRef;

use crate::error::CoreError;
use crate::types::{Candidate, Detection, InstalledView, ProposedDefault};
use crate::Service;

impl Service {
    /// The names a screen shows for a profile reference.
    pub(crate) fn candidate(&self, profile: ProfileRef) -> Candidate {
        let emulator = self.platforms.emulator(&profile.emulator);
        let core_name = profile.core.as_deref().and_then(|core| {
            emulator?
                .cores
                .values()
                .flatten()
                .find(|c| c.id == core)
                .map(|c| c.name.clone())
        });
        Candidate {
            emulator_name: emulator.map_or_else(|| profile.emulator.clone(), |e| e.name.clone()),
            core_name,
            profile,
        }
    }

    /// Which emulators of the database are among `packages` (the launcher activities installed
    /// on the device), and the default proposed for each platform that has games in the library.
    pub fn detect(&self, packages: &[String]) -> Result<Detection, CoreError> {
        let found = self.platforms.detect_installed(packages);
        let present: HashSet<String> = self.platforms()?.into_iter().map(|p| p.id).collect();
        let installed = found
            .installed
            .iter()
            .map(|i| InstalledView {
                name: self
                    .platforms
                    .emulator(&i.emulator)
                    .map_or_else(|| i.emulator.clone(), |e| e.name.clone()),
                emulator: i.emulator.clone(),
                packages: i.packages.clone(),
            })
            .collect();
        let mut proposed: Vec<ProposedDefault> = found
            .proposed_defaults
            .into_iter()
            .filter(|(platform, _)| present.contains(platform))
            .map(|(platform_id, profile)| ProposedDefault {
                platform_name: self.platform_name(&platform_id),
                platform_id,
                candidate: self.candidate(profile),
            })
            .collect();
        proposed.sort_by_key(|p| p.platform_name.to_lowercase());
        Ok(Detection {
            installed,
            proposed,
        })
    }

    /// Saves the proposed default of every library platform that has none yet. Returns how many
    /// were saved.
    pub fn accept_proposed_defaults(&self, packages: &[String]) -> Result<u32, CoreError> {
        let mut saved = 0;
        for p in self.detect(packages)?.proposed {
            if self.platform_default(&p.platform_id)?.is_none() {
                self.set_platform_default(&p.platform_id, Some(&p.candidate.profile))?;
                saved += 1;
            }
        }
        Ok(saved)
    }
}
