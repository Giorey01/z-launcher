//! Classification of a tag file: which kind of game it points to.

use crate::parse::TagFile;

/// The kinds of file tag the launcher understands (one per extension and key pair).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagKind {
    /// `.steamappid` file with a `steamappid` key: a Steam app id.
    SteamAppId,
    /// `.localgameid` file with a `localgameid` key: an Android package name.
    LocalGameId,
    /// `.dpt` file with a `scummvm_id` key: a ScummVM game id.
    ScummvmId,
    /// `.dpt` file with a `vita_game_id` key: a Vita game id.
    VitaGameId,
    /// `.moonlight` file with a `moonlight_id` key: the app name on the Moonlight host.
    MoonlightId,
    /// `.xcloud` file with a `titleId` key: an Xbox title id.
    XcloudTitleId,
    /// Anything else: unknown extension, or the expected key is missing.
    Other,
}

/// Platform folder that disambiguates a `.dpt` file holding a Vita id.
const PLATFORM_PSVITA: &str = "psvita";

/// Extensions (lower-case, no dot) of the files that are tag files.
pub const TAG_EXTENSIONS: [&str; 5] = ["steamappid", "localgameid", "dpt", "moonlight", "xcloud"];

/// Whether `extension` (lower-case, no dot) belongs to a tag file.
pub fn is_tag_extension(extension: &str) -> bool {
    TAG_EXTENSIONS.contains(&extension)
}

impl TagKind {
    /// Returns the key (as the format spells it) whose value identifies the game, if any.
    pub fn key(self) -> Option<&'static str> {
        match self {
            TagKind::SteamAppId => Some("steamappid"),
            TagKind::LocalGameId => Some("localgameid"),
            TagKind::ScummvmId => Some("scummvm_id"),
            TagKind::VitaGameId => Some("vita_game_id"),
            TagKind::MoonlightId => Some("moonlight_id"),
            TagKind::XcloudTitleId => Some("titleId"),
            TagKind::Other => None,
        }
    }
}

impl TagFile {
    /// Classifies the file. The extension picks the candidate kind and the key must be present.
    /// For the shared `.dpt` extension the present key decides; if both keys are there, the
    /// platform folder name (`scummvm` or `psvita`) breaks the tie, else ScummVM wins.
    pub fn kind(&self, platform: Option<&str>) -> TagKind {
        match self.extension.as_str() {
            "steamappid" => self.require(TagKind::SteamAppId),
            "localgameid" => self.require(TagKind::LocalGameId),
            "moonlight" => self.require(TagKind::MoonlightId),
            "xcloud" => self.require(TagKind::XcloudTitleId),
            "dpt" => self.dpt_kind(platform),
            _ => TagKind::Other,
        }
    }

    /// Returns `kind` when its key is present in the file, otherwise [`TagKind::Other`].
    fn require(&self, kind: TagKind) -> TagKind {
        match kind.key() {
            Some(key) if self.get(key).is_some() => kind,
            _ => TagKind::Other,
        }
    }

    /// Resolves the shared `.dpt` extension by key, then by platform folder.
    fn dpt_kind(&self, platform: Option<&str>) -> TagKind {
        let scummvm = self.require_key("scummvm_id");
        let vita = self.require_key("vita_game_id");
        match (scummvm, vita) {
            (true, false) => TagKind::ScummvmId,
            (false, true) => TagKind::VitaGameId,
            (true, true) => match platform.map(str::to_ascii_lowercase).as_deref() {
                Some(PLATFORM_PSVITA) => TagKind::VitaGameId,
                _ => TagKind::ScummvmId,
            },
            (false, false) => TagKind::Other,
        }
    }

    /// Whether the key is present (case-insensitive).
    fn require_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}
