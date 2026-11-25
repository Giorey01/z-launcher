//! Serde types of the two data files, exactly as `docs/spec/platforms.md` describes them.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A file with an extension that stands in for a game and carries a tag (Steam app id, ...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagFileRule {
    /// Extension of the tag file, lowercase, without the dot.
    pub ext: String,
    /// Key of the tag the file carries (`steamappid`, `scummvm_id`, ...).
    pub key: String,
}

/// One platform, identified by the Cocoon folder name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Platform {
    /// Folder name and id of the platform (`psx`, `snes`, ...).
    pub id: String,
    /// Display name, in English; the apps may translate it.
    pub name: String,
    /// Extensions of the game files, lowercase, without the dot; multi-part ones like `p8.png`.
    pub extensions: Vec<String>,
    /// Other names this platform goes by (ES-DE ids such as `sfc`), accepted wherever an id is.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Games of this platform come as several discs bundled by an `.m3u` list.
    #[serde(default)]
    pub multi_disc: bool,
    /// Whether the id is one of Cocoon's folder names (otherwise it is an extra of ours).
    #[serde(default)]
    pub cocoon: bool,
    /// Tag files that represent games of this platform.
    #[serde(default)]
    pub tag_files: Vec<TagFileRule>,
}

/// An Android package that provides an emulator, with the activity to start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// Android package name.
    pub package: String,
    /// Full class name of the activity to start; absent means "the app's launcher activity".
    #[serde(default)]
    pub activity: Option<String>,
    /// Whether a source confirms this package/activity pair.
    pub verified: bool,
}

/// How an emulator is built, which decides how its cores and profiles work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmulatorKind {
    /// A single emulator app.
    Standalone,
    /// RetroArch: one app, many cores chosen per platform.
    Retroarch,
}

/// The form in which a profile needs the ROM to be handed over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RomAccess {
    /// A raw filesystem path (`{rom}`).
    Path,
    /// A `file://` URI (`{rom_uri}`).
    FileUri,
    /// A `content://` URI served by the launcher's FileProvider (`{rom_provider}`).
    ContentUri,
    /// A Storage Access Framework document URI (`{rom_saf}`).
    Saf,
    /// No ROM at all: the launch is driven by tags.
    None,
}

/// An integer extra: a literal number or a template that must render to one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IntTemplate {
    /// A fixed number.
    Literal(i64),
    /// Text with variables, parsed as an integer after substitution.
    Template(String),
}

/// The typed value of an extra, still with variables to substitute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtraTemplateValue {
    /// A string extra.
    String {
        /// Template of the string.
        value: String,
    },
    /// A boolean extra.
    Bool {
        /// The flag.
        value: bool,
    },
    /// An integer extra.
    Int {
        /// The number or its template.
        value: IntTemplate,
    },
    /// A string array extra.
    StringArray {
        /// Template of each element.
        value: Vec<String>,
    },
}

/// One extra of an intent template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtraTemplate {
    /// Name of the extra.
    pub key: String,
    /// Its typed value.
    #[serde(flatten)]
    pub value: ExtraTemplateValue,
}

/// ClipData attached to the intent, so the target may read the URI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipTemplate {
    /// URI template.
    pub uri: String,
    /// MIME type of the clip item.
    pub mime: String,
}

/// A URI the launcher must grant to the target package before starting it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantTemplate {
    /// URI template.
    pub uri: String,
    /// Grant read access.
    pub read: bool,
    /// Grant write access.
    pub write: bool,
    /// Skip the grant (instead of failing) when the caller has no value for the variable.
    #[serde(default)]
    pub optional: bool,
}

/// Android intent of a profile, with `{variables}` in the strings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentTemplate {
    /// Intent action; absent means none.
    #[serde(default)]
    pub action: Option<String>,
    /// Intent categories.
    #[serde(default)]
    pub category: Vec<String>,
    /// Data URI template.
    #[serde(default)]
    pub data: Option<String>,
    /// MIME type.
    #[serde(default)]
    pub mime: Option<String>,
    /// Extras, in order.
    #[serde(default)]
    pub extras: Vec<ExtraTemplate>,
    /// Flags by name (see `KNOWN_FLAGS` in the spec).
    #[serde(default)]
    pub flags: Vec<String>,
    /// ClipData to attach.
    #[serde(default)]
    pub clip_data: Option<ClipTemplate>,
    /// URI permissions to grant to the target package before the launch.
    #[serde(default)]
    pub uri_grants: Vec<GrantTemplate>,
}

/// One way to launch games of some platforms with an emulator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Id of the profile, unique inside its emulator.
    pub id: String,
    /// Label shown to the user, in English.
    pub label: String,
    /// Platforms this profile launches.
    pub platforms: Vec<String>,
    /// How the ROM must be handed over.
    pub rom_access: RomAccess,
    /// The profile needs a RetroArch core (`{core}`, `{core_path}`).
    #[serde(default)]
    pub uses_core: bool,
    /// The intent to build.
    pub intent: IntentTemplate,
    /// Whether a source confirms the intent.
    pub verified: bool,
    /// Where the intent comes from (`es-de`, `es-de+device`, `upstream`, `manual`).
    pub source: String,
    /// Date a real device launched a game with this intent, when one did.
    #[serde(default)]
    pub verified_on: Option<String>,
    /// Free note on differences from the source or open questions.
    #[serde(default)]
    pub note: Option<String>,
}

/// A RetroArch core for one platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Core {
    /// Id of the core, stable across builds (`mednafen_psx`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// File name of the shared library inside RetroArch's core folder.
    pub file: String,
}

/// An emulator with its packages, profiles and (for RetroArch) cores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emulator {
    /// Id of the emulator.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Standalone app or RetroArch.
    pub kind: EmulatorKind,
    /// Packages that provide it, most common first.
    pub packages: Vec<Package>,
    /// Platforms any profile of this emulator launches.
    pub platforms: Vec<String>,
    /// Launch profiles, preferred first.
    pub profiles: Vec<Profile>,
    /// RetroArch cores by platform id, preferred first.
    #[serde(default)]
    pub cores: BTreeMap<String, Vec<Core>>,
}

/// An entry of a preference list: an emulator, and for RetroArch which core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefEntry {
    /// Id of the emulator.
    pub emulator: String,
    /// Id of a core, for RetroArch; absent means the platform's first core.
    #[serde(default)]
    pub core: Option<String>,
}

/// A Home folder that shows the games of several platforms together (Steam and Windows under
/// "PC"). Each game keeps its own platform, so launching and settings do not change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    /// Id of the folder; it must not be the id or alias of a platform.
    pub id: String,
    /// Display name, in English; the apps may translate it.
    pub name: String,
    /// Canonical ids of the member platforms, in the order their folder art is tried.
    pub platforms: Vec<String>,
}

/// Contents of `platforms.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformsFile {
    /// Format version the file follows.
    pub schema: u32,
    /// Version of the data; the higher copy wins.
    pub version: u32,
    /// The platforms.
    pub platforms: Vec<Platform>,
    /// Folders that group several platforms on the Home.
    #[serde(default)]
    pub folders: Vec<Folder>,
}

/// Contents of `emulators.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorsFile {
    /// Format version the file follows.
    pub schema: u32,
    /// Version of the data; the higher copy wins.
    pub version: u32,
    /// Preference order used for platforms without a list of their own.
    #[serde(default)]
    pub default_preference: Vec<PrefEntry>,
    /// Preference order per platform id.
    #[serde(default)]
    pub preferences: BTreeMap<String, Vec<PrefEntry>>,
    /// The emulators.
    pub emulators: Vec<Emulator>,
}
