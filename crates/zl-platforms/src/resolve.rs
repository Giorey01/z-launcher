//! Choice of the profile that launches a game (RF-08): game override, then platform default,
//! then extension rule, then the user.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::db::PlatformDb;
use crate::detect::InstalledEmulator;
use crate::error::LaunchError;

/// A reference to a profile, and for RetroArch the core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileRef {
    /// Emulator id.
    pub emulator: String,
    /// Profile id inside the emulator.
    pub profile: String,
    /// Core id, for RetroArch profiles; absent means the platform's preferred core.
    #[serde(default)]
    pub core: Option<String>,
}

impl ProfileRef {
    /// Builds a reference.
    pub fn new(emulator: &str, profile: &str, core: Option<&str>) -> Self {
        Self {
            emulator: emulator.to_string(),
            profile: profile.to_string(),
            core: core.map(str::to_string),
        }
    }
}

/// Which step of the order picked the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedBy {
    /// The game's own override.
    GameOverride,
    /// The platform's default.
    PlatformDefault,
    /// A rule for the game file's extension.
    ExtensionRule,
    /// Nothing was configured and exactly one installed profile can launch the game.
    OnlyCandidate,
}

/// Everything the choice depends on.
#[derive(Debug, Clone, Copy)]
pub struct ResolveInput<'a> {
    /// Platform id or alias of the game.
    pub platform: &'a str,
    /// File name of the game, used for the extension rules.
    pub file_name: &'a str,
    /// Profile set on the game itself.
    pub game_override: Option<&'a ProfileRef>,
    /// Profile set as the platform's default.
    pub platform_default: Option<&'a ProfileRef>,
    /// Profiles by extension (lowercase, no dot, multi-part allowed), set by the user.
    pub extension_defaults: &'a HashMap<String, ProfileRef>,
    /// Emulators present on the device.
    pub installed: &'a [InstalledEmulator],
}

/// Outcome of [`PlatformDb::resolve_profile`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// A profile was chosen.
    Resolved {
        /// The profile, with the core filled in for RetroArch.
        profile: ProfileRef,
        /// The step that chose it.
        by: ResolvedBy,
    },
    /// The user must pick one; an empty list means no installed emulator can launch the game.
    NeedsChoice {
        /// Installed profiles that can launch the game, preferred first.
        candidates: Vec<ProfileRef>,
    },
}

/// The extensions a file name could be known by, longest first (`a.p8.png` gives `p8.png`, `png`).
fn extension_suffixes(file_name: &str) -> Vec<String> {
    let lower = file_name.to_lowercase();
    lower
        .match_indices('.')
        .skip(usize::from(lower.starts_with('.')))
        .map(|(i, _)| lower[i + 1..].to_string())
        .collect()
}

/// The candidate a reference stands for, if it is installed and launches the platform.
fn usable(cands: &[ProfileRef], wanted: &ProfileRef) -> Option<ProfileRef> {
    cands
        .iter()
        .find(|c| {
            c.emulator == wanted.emulator
                && c.profile == wanted.profile
                && (wanted.core.is_none() || c.core == wanted.core)
        })
        .cloned()
}

impl PlatformDb {
    /// Applies the RF-08 order. A reference that points to an emulator that is not installed, or
    /// to a profile that does not launch the platform, is skipped and the next step applies.
    /// Without any configuration a single installed candidate is used; several, or none, give
    /// [`Resolution::NeedsChoice`].
    pub fn resolve_profile(&self, input: &ResolveInput<'_>) -> Result<Resolution, LaunchError> {
        let platform = self
            .platform(input.platform)
            .ok_or_else(|| LaunchError::UnknownPlatform(input.platform.to_string()))?;
        let cands = self.candidates(&platform.id, input.installed);
        let ext_rule = extension_suffixes(input.file_name)
            .iter()
            .find_map(|e| input.extension_defaults.get(e));
        let steps = [
            (input.game_override, ResolvedBy::GameOverride),
            (input.platform_default, ResolvedBy::PlatformDefault),
            (ext_rule, ResolvedBy::ExtensionRule),
        ];
        for (wanted, by) in steps {
            if let Some(profile) = wanted.and_then(|w| usable(&cands, w)) {
                return Ok(Resolution::Resolved { profile, by });
            }
        }
        if let [only] = cands.as_slice() {
            return Ok(Resolution::Resolved {
                profile: only.clone(),
                by: ResolvedBy::OnlyCandidate,
            });
        }
        Ok(Resolution::NeedsChoice { candidates: cands })
    }
}
