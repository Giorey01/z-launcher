//! Plain data the service hands to the apps.

use zl_db::GameKind;
use zl_platforms::{LaunchIntent, ProfileRef, ResolvedBy};

/// A ROM folder the user added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootInfo {
    /// Row id.
    pub id: i64,
    /// Absolute path of the folder.
    pub path: String,
    /// SAF tree URI the user granted for the folder, if any.
    pub saf_tree: Option<String>,
    /// Whether scans include the folder.
    pub enabled: bool,
}

/// A platform present in the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformEntry {
    /// Canonical platform id.
    pub id: String,
    /// English display name.
    pub name: String,
    /// Games that are not missing.
    pub games: u32,
}

/// A game as listed on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEntry {
    /// Row id.
    pub id: i64,
    /// Display title.
    pub title: String,
    /// Canonical platform id.
    pub platform_id: String,
    /// English platform name.
    pub platform_name: String,
    /// Shape of the entry.
    pub kind: GameKind,
    /// Not found by the last scan.
    pub missing: bool,
    /// The title to show: `title` without the tags of the file name (`(Europe)`, `[v0]`).
    pub display_title: String,
    /// Best cover, if the game has media.
    pub icon: Option<String>,
    /// Best hero image.
    pub hero: Option<String>,
    /// Best logo.
    pub logo: Option<String>,
}

/// A platform of the library with its default emulator, for the settings screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformEmulatorSetting {
    /// Canonical platform id.
    pub platform_id: String,
    /// English platform name.
    pub platform_name: String,
    /// The default set for the platform, if any (it may not be installed any more).
    pub current: Option<Candidate>,
    /// Emulators installed on the device that can launch the platform, preferred first.
    pub candidates: Vec<Candidate>,
}

/// An emulator choice shown to the user, with the names the screen needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// What to save when the user picks it.
    pub profile: ProfileRef,
    /// Emulator display name.
    pub emulator_name: String,
    /// RetroArch core display name.
    pub core_name: Option<String>,
}

/// What to do to launch a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Start this intent.
    Launch {
        /// The intent.
        intent: Box<LaunchIntent>,
        /// The profile it comes from.
        profile: ProfileRef,
        /// The step that chose it.
        by: ResolvedBy,
    },
    /// The user must pick; empty means no installed emulator can launch the game.
    NeedsChoice {
        /// Installed choices, preferred first.
        candidates: Vec<Candidate>,
    },
}

/// An emulator found on the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledView {
    /// Emulator id.
    pub emulator: String,
    /// Display name.
    pub name: String,
    /// Installed packages of it.
    pub packages: Vec<String>,
}

/// The default proposed for one platform of the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedDefault {
    /// Canonical platform id.
    pub platform_id: String,
    /// English platform name.
    pub platform_name: String,
    /// The proposed emulator.
    pub candidate: Candidate,
}

/// Result of detecting emulators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    /// Emulators present, in database order.
    pub installed: Vec<InstalledView>,
    /// Proposed default for each platform of the library an installed emulator launches.
    pub proposed: Vec<ProposedDefault>,
}

/// How the Home shows the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryMode {
    /// One folder per platform.
    Smart,
    /// All games on the Home.
    Simple,
}
