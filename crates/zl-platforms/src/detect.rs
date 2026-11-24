//! Detection of installed emulators and the defaults proposed to the user at onboarding.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::db::PlatformDb;
use crate::model::{Emulator, EmulatorKind, PrefEntry};
use crate::resolve::ProfileRef;

/// An emulator found on the device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledEmulator {
    /// Emulator id.
    pub emulator: String,
    /// Installed packages of it, in the order the database lists them (the first is used).
    pub packages: Vec<String>,
}

/// Result of [`PlatformDb::detect_installed`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detection {
    /// Emulators present on the device, in database order.
    pub installed: Vec<InstalledEmulator>,
    /// Proposed default per platform id, only for platforms some installed emulator launches.
    pub proposed_defaults: BTreeMap<String, ProfileRef>,
}

/// Position of a candidate in a preference list; unlisted ones go last.
fn rank(prefs: &[PrefEntry], c: &ProfileRef) -> usize {
    prefs
        .iter()
        .position(|p| p.emulator == c.emulator && (p.core.is_none() || p.core == c.core))
        .unwrap_or(prefs.len())
}

impl PlatformDb {
    /// Which emulators of the database are among `package_names`, and the proposed default
    /// profile for every platform they can launch. The preference order is in the data.
    pub fn detect_installed<S: AsRef<str>>(&self, package_names: &[S]) -> Detection {
        let installed: Vec<InstalledEmulator> = self
            .emulators()
            .iter()
            .filter_map(|e| installed_packages(e, package_names))
            .collect();
        let mut proposed_defaults = BTreeMap::new();
        for platform in self.platforms() {
            if let Some(first) = self.candidates(&platform.id, &installed).into_iter().next() {
                proposed_defaults.insert(platform.id.clone(), first);
            }
        }
        Detection {
            installed,
            proposed_defaults,
        }
    }

    /// Every (emulator, profile, core) among the installed emulators that launches `platform`,
    /// preferred first. RetroArch yields one entry per core of the platform.
    pub fn candidates(&self, platform: &str, installed: &[InstalledEmulator]) -> Vec<ProfileRef> {
        let mut out = Vec::new();
        for emu in self.emulators() {
            if installed.iter().any(|i| i.emulator == emu.id) {
                out.extend(emulator_candidates(emu, platform));
            }
        }
        let prefs = self.preference(platform);
        out.sort_by_key(|c| rank(prefs, c));
        out
    }
}

/// The packages of `emu` found in `names`, or `None` when none is installed.
fn installed_packages<S: AsRef<str>>(emu: &Emulator, names: &[S]) -> Option<InstalledEmulator> {
    let mut found: Vec<String> = Vec::new();
    for pkg in &emu.packages {
        let present = names.iter().any(|n| n.as_ref() == pkg.package);
        if present && !found.contains(&pkg.package) {
            found.push(pkg.package.clone());
        }
    }
    (!found.is_empty()).then(|| InstalledEmulator {
        emulator: emu.id.clone(),
        packages: found,
    })
}

/// Candidates of one emulator for a platform.
fn emulator_candidates(emu: &Emulator, platform: &str) -> Vec<ProfileRef> {
    let profiles = emu
        .profiles
        .iter()
        .filter(|p| p.platforms.iter().any(|x| x == platform));
    let mut out = Vec::new();
    for profile in profiles {
        if emu.kind == EmulatorKind::Retroarch && profile.uses_core {
            for core in emu.cores.get(platform).into_iter().flatten() {
                out.push(ProfileRef::new(&emu.id, &profile.id, Some(&core.id)));
            }
        } else {
            out.push(ProfileRef::new(&emu.id, &profile.id, None));
        }
    }
    out
}
