//! UniFFI records and enums mirroring the service types, with their conversions.

use std::collections::{BTreeMap, HashMap};

use zl_core as core;
use zl_db::GameKind as DbKind;
use zl_platforms as plat;

/// A reference to an emulator profile (and for RetroArch the core).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ProfileRef {
    /// Emulator id.
    pub emulator: String,
    /// Profile id.
    pub profile: String,
    /// RetroArch core id.
    pub core: Option<String>,
}

impl From<plat::ProfileRef> for ProfileRef {
    /// One to one.
    fn from(p: plat::ProfileRef) -> Self {
        ProfileRef {
            emulator: p.emulator,
            profile: p.profile,
            core: p.core,
        }
    }
}

impl From<ProfileRef> for plat::ProfileRef {
    /// One to one.
    fn from(p: ProfileRef) -> Self {
        plat::ProfileRef {
            emulator: p.emulator,
            profile: p.profile,
            core: p.core,
        }
    }
}

/// A ROM folder.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RomRoot {
    /// Row id.
    pub id: i64,
    /// Absolute path.
    pub path: String,
    /// SAF tree URI granted for it.
    pub saf_tree: Option<String>,
    /// Included in scans.
    pub enabled: bool,
}

impl From<core::RootInfo> for RomRoot {
    /// One to one.
    fn from(r: core::RootInfo) -> Self {
        RomRoot {
            id: r.id,
            path: r.path,
            saf_tree: r.saf_tree,
            enabled: r.enabled,
        }
    }
}

/// A platform with games.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct PlatformEntry {
    /// Canonical id.
    pub id: String,
    /// English name.
    pub name: String,
    /// Games not missing.
    pub games: u32,
}

impl From<core::PlatformEntry> for PlatformEntry {
    /// One to one.
    fn from(p: core::PlatformEntry) -> Self {
        PlatformEntry {
            id: p.id,
            name: p.name,
            games: p.games,
        }
    }
}

/// Shape of a library entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GameKind {
    /// A single ROM file.
    Rom,
    /// An `.m3u` playlist.
    M3u,
    /// Several discs of one game.
    Multidisc,
    /// A tag file.
    TagFile,
    /// A game folder.
    Folder,
}

impl From<DbKind> for GameKind {
    /// One to one.
    fn from(k: DbKind) -> Self {
        match k {
            DbKind::Rom => GameKind::Rom,
            DbKind::M3u => GameKind::M3u,
            DbKind::Multidisc => GameKind::Multidisc,
            DbKind::TagFile => GameKind::TagFile,
            DbKind::Folder => GameKind::Folder,
        }
    }
}

/// A game to show.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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
}

impl From<core::GameEntry> for GameEntry {
    /// One to one.
    fn from(g: core::GameEntry) -> Self {
        GameEntry {
            id: g.id,
            title: g.title,
            platform_id: g.platform_id,
            platform_name: g.platform_name,
            kind: g.kind.into(),
            missing: g.missing,
        }
    }
}

/// An emulator choice with the names to show.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Candidate {
    /// What to save when picked.
    pub profile: ProfileRef,
    /// Emulator name.
    pub emulator_name: String,
    /// RetroArch core name.
    pub core_name: Option<String>,
}

impl From<core::Candidate> for Candidate {
    /// One to one.
    fn from(c: core::Candidate) -> Self {
        Candidate {
            profile: c.profile.into(),
            emulator_name: c.emulator_name,
            core_name: c.core_name,
        }
    }
}

/// A substituted extra value.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ExtraValue {
    /// A string.
    Text {
        /// The text.
        value: String,
    },
    /// A boolean.
    Flag {
        /// The flag.
        value: bool,
    },
    /// An integer.
    Number {
        /// The number.
        value: i64,
    },
    /// A string array.
    TextList {
        /// The elements.
        value: Vec<String>,
    },
}

/// One extra of an intent.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Extra {
    /// Name.
    pub key: String,
    /// Typed value.
    pub value: ExtraValue,
}

/// ClipData to attach.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ClipData {
    /// Item URI.
    pub uri: String,
    /// Item MIME type.
    pub mime: String,
}

/// A URI permission to grant before the launch.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct UriGrant {
    /// URI to grant.
    pub uri: String,
    /// Read access.
    pub read: bool,
    /// Write access.
    pub write: bool,
}

/// A complete intent, ready for Kotlin.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct LaunchIntent {
    /// Action.
    pub action: Option<String>,
    /// Emulator package.
    pub package: String,
    /// Full activity class name.
    pub activity: Option<String>,
    /// Categories.
    pub categories: Vec<String>,
    /// Data URI.
    pub data: Option<String>,
    /// MIME type.
    pub mime: Option<String>,
    /// Extras in order.
    pub extras: Vec<Extra>,
    /// Flag names (`new_task`, `grant_read_uri_permission`, ...).
    pub flags: Vec<String>,
    /// ClipData.
    pub clip_data: Option<ClipData>,
    /// Permissions to grant to `package` first.
    pub uri_grants: Vec<UriGrant>,
}

impl From<plat::ExtraValue> for ExtraValue {
    /// One to one.
    fn from(v: plat::ExtraValue) -> Self {
        match v {
            plat::ExtraValue::String { value } => ExtraValue::Text { value },
            plat::ExtraValue::Bool { value } => ExtraValue::Flag { value },
            plat::ExtraValue::Int { value } => ExtraValue::Number { value },
            plat::ExtraValue::StringArray { value } => ExtraValue::TextList { value },
        }
    }
}

impl From<plat::LaunchIntent> for LaunchIntent {
    /// One to one.
    fn from(i: plat::LaunchIntent) -> Self {
        LaunchIntent {
            action: i.action,
            package: i.package,
            activity: i.activity,
            categories: i.categories,
            data: i.data,
            mime: i.mime,
            extras: i
                .extras
                .into_iter()
                .map(|e| Extra {
                    key: e.key,
                    value: e.value.into(),
                })
                .collect(),
            flags: i.flags,
            clip_data: i.clip_data.map(|c| ClipData {
                uri: c.uri,
                mime: c.mime,
            }),
            uri_grants: i
                .uri_grants
                .into_iter()
                .map(|g| UriGrant {
                    uri: g.uri,
                    read: g.read,
                    write: g.write,
                })
                .collect(),
        }
    }
}

/// Which step of the order chose the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ResolvedBy {
    /// The game's override.
    GameOverride,
    /// The platform default.
    PlatformDefault,
    /// An extension rule.
    ExtensionRule,
    /// The only installed candidate.
    OnlyCandidate,
}

impl From<plat::ResolvedBy> for ResolvedBy {
    /// One to one.
    fn from(b: plat::ResolvedBy) -> Self {
        match b {
            plat::ResolvedBy::GameOverride => ResolvedBy::GameOverride,
            plat::ResolvedBy::PlatformDefault => ResolvedBy::PlatformDefault,
            plat::ResolvedBy::ExtensionRule => ResolvedBy::ExtensionRule,
            plat::ResolvedBy::OnlyCandidate => ResolvedBy::OnlyCandidate,
        }
    }
}

/// What to do to launch a game.
// UniFFI enums cannot hold a `Box`; the size difference is irrelevant for a value built once per launch.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum Plan {
    /// Start this intent.
    Launch {
        /// The intent.
        intent: LaunchIntent,
        /// The chosen profile.
        profile: ProfileRef,
        /// The step that chose it.
        by: ResolvedBy,
    },
    /// The user must pick one; empty means no installed emulator launches the game.
    NeedsChoice {
        /// Choices, preferred first.
        candidates: Vec<Candidate>,
    },
}

impl From<core::Plan> for Plan {
    /// One to one.
    fn from(p: core::Plan) -> Self {
        match p {
            core::Plan::Launch {
                intent,
                profile,
                by,
            } => Plan::Launch {
                intent: (*intent).into(),
                profile: profile.into(),
                by: by.into(),
            },
            core::Plan::NeedsChoice { candidates } => Plan::NeedsChoice {
                candidates: candidates.into_iter().map(Candidate::from).collect(),
            },
        }
    }
}

/// An emulator found on the device.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct InstalledEmulator {
    /// Emulator id.
    pub emulator: String,
    /// Display name.
    pub name: String,
    /// Installed packages.
    pub packages: Vec<String>,
}

/// The default proposed for a platform.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ProposedDefault {
    /// Canonical platform id.
    pub platform_id: String,
    /// English platform name.
    pub platform_name: String,
    /// The proposed emulator.
    pub candidate: Candidate,
}

/// Result of detecting emulators.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Detection {
    /// Emulators present.
    pub installed: Vec<InstalledEmulator>,
    /// Proposed defaults for platforms of the library.
    pub proposed: Vec<ProposedDefault>,
}

impl From<core::Detection> for Detection {
    /// One to one.
    fn from(d: core::Detection) -> Self {
        Detection {
            installed: d
                .installed
                .into_iter()
                .map(|i| InstalledEmulator {
                    emulator: i.emulator,
                    name: i.name,
                    packages: i.packages,
                })
                .collect(),
            proposed: d
                .proposed
                .into_iter()
                .map(|p| ProposedDefault {
                    platform_id: p.platform_id,
                    platform_name: p.platform_name,
                    candidate: p.candidate.into(),
                })
                .collect(),
        }
    }
}

/// How the Home shows the library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum LibraryMode {
    /// One folder per platform.
    Smart,
    /// All games on the Home.
    Simple,
}

impl From<core::LibraryMode> for LibraryMode {
    /// One to one.
    fn from(m: core::LibraryMode) -> Self {
        match m {
            core::LibraryMode::Smart => LibraryMode::Smart,
            core::LibraryMode::Simple => LibraryMode::Simple,
        }
    }
}

impl From<LibraryMode> for core::LibraryMode {
    /// One to one.
    fn from(m: LibraryMode) -> Self {
        match m {
            LibraryMode::Smart => core::LibraryMode::Smart,
            LibraryMode::Simple => core::LibraryMode::Simple,
        }
    }
}

/// A launch request that does not come from the library (the shared fixtures use it).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RawLaunchRequest {
    /// Path of the ROM.
    pub rom_path: String,
    /// `file://` URI.
    pub file_uri: Option<String>,
    /// SAF document URI.
    pub saf_uri: Option<String>,
    /// SAF tree URI.
    pub saf_tree: Option<String>,
    /// FileProvider URI.
    pub provider_uri: Option<String>,
    /// Platform id or alias.
    pub platform: String,
    /// Platform folder.
    pub platform_dir: Option<String>,
    /// Tags of the game.
    pub tags: HashMap<String, String>,
    /// RetroArch core to use.
    pub core: Option<String>,
    /// Package to launch.
    pub package: Option<String>,
    /// Profile set on the game.
    pub game_override: Option<ProfileRef>,
    /// Profile set on the platform.
    pub platform_default: Option<ProfileRef>,
    /// Profiles by extension.
    pub extension_defaults: HashMap<String, ProfileRef>,
}

impl From<RawLaunchRequest> for plat::LaunchRequest {
    /// Builds the request the platform crate understands.
    fn from(r: RawLaunchRequest) -> Self {
        plat::LaunchRequest {
            context: plat::LaunchContext {
                rom_path: r.rom_path,
                rom_uris: plat::RomUris {
                    file: r.file_uri,
                    saf: r.saf_uri,
                    saf_tree: r.saf_tree,
                    provider: r.provider_uri,
                },
                platform: r.platform,
                platform_dir: r.platform_dir,
                tags: r.tags.into_iter().collect::<BTreeMap<_, _>>(),
                core: r.core,
                package: r.package,
            },
            game_override: r.game_override.map(Into::into),
            platform_default: r.platform_default.map(Into::into),
            extension_defaults: r
                .extension_defaults
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
