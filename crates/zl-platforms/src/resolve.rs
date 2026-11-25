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

/// Separator of the parts of a stored profile reference ([`ProfileRef::to_key`]).
const KEY_SEPARATOR: char = '/';

impl ProfileRef {
    /// Text form kept in the database: `emulator/profile` or `emulator/profile/core`. Ids never
    /// contain a slash (the database validation only accepts plain ids).
    pub fn to_key(&self) -> String {
        match &self.core {
            Some(core) => format!(
                "{}{KEY_SEPARATOR}{}{KEY_SEPARATOR}{core}",
                self.emulator, self.profile
            ),
            None => format!("{}{KEY_SEPARATOR}{}", self.emulator, self.profile),
        }
    }

    /// Reads the text form back; `None` when it does not have two or three non-empty parts.
    pub fn from_key(key: &str) -> Option<ProfileRef> {
        let parts: Vec<&str> = key.split(KEY_SEPARATOR).collect();
        match parts.as_slice() {
            [e, p] if !e.is_empty() && !p.is_empty() => Some(ProfileRef::new(e, p, None)),
            [e, p, c] if !e.is_empty() && !p.is_empty() && !c.is_empty() => {
                Some(ProfileRef::new(e, p, Some(c)))
            }
            _ => None,
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
    /// The game is an Android app (a recomp or port) filed under a console by its
    /// `localgameid` tag: the app itself starts, no emulator is involved.
    AndroidApp,
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

#[cfg(test)]
mod key_tests {
    use super::ProfileRef;

    /// A reference survives the text form, with and without a core.
    #[test]
    fn key_round_trip() {
        for r in [
            ProfileRef::new("ppsspp", "ppsspp", None),
            ProfileRef::new("retroarch", "core", Some("pcsx_rearmed")),
        ] {
            assert_eq!(ProfileRef::from_key(&r.to_key()), Some(r));
        }
        assert_eq!(ProfileRef::new("a", "b", Some("c")).to_key(), "a/b/c");
    }

    /// Malformed keys are refused instead of guessed.
    #[test]
    fn bad_keys_are_refused() {
        for k in ["", "a", "a/", "/b", "a/b/", "a/b/c/d"] {
            assert_eq!(ProfileRef::from_key(k), None, "{k}");
        }
    }
}
