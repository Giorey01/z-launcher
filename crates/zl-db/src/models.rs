//! Plain data types exchanged with the repository.

use crate::error::DbError;

/// How a library entry is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKind {
    /// A single ROM file (a CUE or GDI sheet counts: its tracks are member files).
    Rom,
    /// An `.m3u` playlist that lists the discs of one game.
    M3u,
    /// Several `(Disc N)` files collapsed into one entry.
    Multidisc,
    /// A tag file (Steam app id, Android package, ...).
    TagFile,
    /// A folder whose bare disc files (`Disc 1`, `Disc 2`) form one game.
    Folder,
}

impl GameKind {
    /// Stable spelling stored in the database and shared with the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            GameKind::Rom => "rom",
            GameKind::M3u => "m3u",
            GameKind::Multidisc => "multidisc",
            GameKind::TagFile => "tagfile",
            GameKind::Folder => "folder",
        }
    }

    /// Reads the stored spelling back.
    pub fn parse(text: &str) -> Result<GameKind, DbError> {
        match text {
            "rom" => Ok(GameKind::Rom),
            "m3u" => Ok(GameKind::M3u),
            "multidisc" => Ok(GameKind::Multidisc),
            "tagfile" => Ok(GameKind::TagFile),
            "folder" => Ok(GameKind::Folder),
            other => Err(DbError::InvalidValue(other.to_string())),
        }
    }
}

/// A folder the user added to be scanned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    /// Row id.
    pub id: i64,
    /// Absolute path of the folder.
    pub path: String,
    /// Whether scans include this root.
    pub enabled: bool,
    /// Whether sub-folders of platform folders are scanned.
    pub scan_subfolders: bool,
    /// Storage Access Framework tree URI the user granted for this folder, if any.
    pub saf_tree: Option<String>,
}

/// A member file of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameFile {
    /// Absolute path.
    pub path: String,
    /// Size in bytes.
    pub size: i64,
    /// Modification time, Unix milliseconds.
    pub mtime_ms: i64,
}

/// A game to insert or update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewGame {
    /// Root the game was found under.
    pub root_id: i64,
    /// Platform id (lower-case folder name).
    pub platform_id: String,
    /// Display title derived from the file name.
    pub title: String,
    /// Primary path: the identity of the game.
    pub path: String,
    /// Shape of the entry.
    pub kind: GameKind,
    /// Total size of the member files in bytes.
    pub size: i64,
    /// Latest modification time among the members, Unix milliseconds.
    pub mtime_ms: i64,
    /// Hash of the game's shape, used to skip unchanged games on rescan.
    pub fingerprint: i64,
    /// Every member file, the primary one included.
    pub files: Vec<GameFile>,
    /// Tag file pairs (empty for other kinds).
    pub tags: Vec<(String, String)>,
}

/// A stored game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// Row id.
    pub id: i64,
    /// Root the game belongs to.
    pub root_id: i64,
    /// Platform id.
    pub platform_id: String,
    /// Display title.
    pub title: String,
    /// Primary path.
    pub path: String,
    /// Shape of the entry.
    pub kind: GameKind,
    /// Total size in bytes.
    pub size: i64,
    /// Latest modification time, Unix milliseconds.
    pub mtime_ms: i64,
    /// Not found by the last scan that covered it.
    pub missing: bool,
    /// When the game was first indexed, Unix seconds.
    pub added_at: i64,
    /// Id of the last scan that saw the game.
    pub last_seen_scan: i64,
}

/// The few columns a rescan needs to decide whether a game changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingGame {
    /// Row id.
    pub id: i64,
    /// Platform id.
    pub platform_id: String,
    /// Fingerprint stored at the last write.
    pub fingerprint: i64,
    /// Whether the game is currently flagged missing.
    pub missing: bool,
}

/// Filter for [`crate::Db::query_games`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameQuery {
    /// Only this platform.
    pub platform: Option<String>,
    /// Only titles containing this text, ignoring case.
    pub title_contains: Option<String>,
    /// Also return games flagged missing.
    pub include_missing: bool,
    /// Maximum number of rows.
    pub limit: Option<u32>,
}

/// Number of visible (not missing) games of one platform id as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCount {
    /// Platform id as stored (the folder name in lower case).
    pub platform_id: String,
    /// Games not flagged missing.
    pub games: i64,
}

/// Totals shown in settings and used by tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// All games, missing ones included.
    pub games: i64,
    /// Games flagged missing.
    pub missing: i64,
    /// Platforms seen.
    pub platforms: i64,
}

/// What a media file shows. The spelling is the one stored in the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MediaKind {
    /// The cover (ES-DE `covers`, Cocoon `icon`): the square image of the game in the grid.
    Icon,
    /// The big background image (ES-DE `fanart`, Cocoon `hero`).
    Hero,
    /// The logo or marquee (ES-DE `marquees`, Cocoon `logo`).
    Logo,
    /// A gameplay screenshot (ES-DE `screenshots`, Cocoon `screenshot_gameplay`).
    Screenshot,
    /// A video (ES-DE `videos`).
    Video,
    /// A 3D box render (ES-DE `3dboxes`).
    Box3d,
    /// The title screen (ES-DE `titlescreens`, Cocoon `screenshot_title`).
    TitleScreen,
    /// The back of the box (ES-DE `backcovers`).
    BackCover,
    /// A composed image of screenshot, cover and logo (ES-DE `miximages`).
    MixImage,
    /// The cartridge or disc (ES-DE `physicalmedia`).
    PhysicalMedia,
    /// The manual, a PDF (ES-DE `manuals`).
    Manual,
}

impl MediaKind {
    /// Every kind, in declaration order.
    pub const ALL: [MediaKind; 11] = [
        MediaKind::Icon,
        MediaKind::Hero,
        MediaKind::Logo,
        MediaKind::Screenshot,
        MediaKind::Video,
        MediaKind::Box3d,
        MediaKind::TitleScreen,
        MediaKind::BackCover,
        MediaKind::MixImage,
        MediaKind::PhysicalMedia,
        MediaKind::Manual,
    ];

    /// Stable spelling stored in the database and used in the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::Icon => "icon",
            MediaKind::Hero => "hero",
            MediaKind::Logo => "logo",
            MediaKind::Screenshot => "screenshot",
            MediaKind::Video => "video",
            MediaKind::Box3d => "3dbox",
            MediaKind::TitleScreen => "titlescreen",
            MediaKind::BackCover => "backcover",
            MediaKind::MixImage => "miximage",
            MediaKind::PhysicalMedia => "physicalmedia",
            MediaKind::Manual => "manual",
        }
    }

    /// Reads the stored spelling back.
    pub fn parse(text: &str) -> Result<MediaKind, DbError> {
        MediaKind::ALL
            .into_iter()
            .find(|k| k.as_str() == text)
            .ok_or_else(|| DbError::InvalidValue(text.to_string()))
    }
}

/// Where a media file comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MediaSource {
    /// Cocoon's `downloaded_media` folder.
    Cocoon,
    /// ES-DE's `downloaded_media` folder.
    Esde,
    /// A file the user picked.
    User,
    /// A file downloaded from SteamGridDB.
    SteamGridDb,
}

impl MediaSource {
    /// Stable spelling stored in the database and used in the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            MediaSource::Cocoon => "cocoon",
            MediaSource::Esde => "esde",
            MediaSource::User => "user",
            MediaSource::SteamGridDb => "sgdb",
        }
    }

    /// Reads the stored spelling back.
    pub fn parse(text: &str) -> Result<MediaSource, DbError> {
        match text {
            "cocoon" => Ok(MediaSource::Cocoon),
            "esde" => Ok(MediaSource::Esde),
            "user" => Ok(MediaSource::User),
            "sgdb" => Ok(MediaSource::SteamGridDb),
            other => Err(DbError::InvalidValue(other.to_string())),
        }
    }
}

/// A media file to store for a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMedia {
    /// The game.
    pub game_id: i64,
    /// What the file shows.
    pub kind: MediaKind,
    /// Absolute path of the file.
    pub path: String,
    /// Where it comes from.
    pub source: MediaSource,
    /// Number of the Cocoon edit the file is, `None` for a plain file.
    pub edit: Option<u32>,
}

/// A stored media file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaRow {
    /// The game.
    pub game_id: i64,
    /// What the file shows.
    pub kind: MediaKind,
    /// Absolute path of the file.
    pub path: String,
    /// Where it comes from.
    pub source: MediaSource,
    /// Number of the Cocoon edit, if the file is one.
    pub edit: Option<u32>,
    /// The user's choice: an import never removes or replaces it, and it wins over all others.
    pub protected: bool,
}

/// A field that can be protected from automated scraping by a user override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetaField {
    /// Title override.
    Title,
    /// Synopsis or description.
    Description,
    /// Release date (YYYY-MM-DD).
    ReleaseDate,
    /// Developer studio.
    Developer,
    /// Publisher company.
    Publisher,
    /// Genres (comma-separated or JSON array).
    Genres,
    /// Maximum player count.
    Players,
    /// Critical rating.
    Rating,
}

impl MetaField {
    /// Stable spelling stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            MetaField::Title => "title",
            MetaField::Description => "description",
            MetaField::ReleaseDate => "release_date",
            MetaField::Developer => "developer",
            MetaField::Publisher => "publisher",
            MetaField::Genres => "genres",
            MetaField::Players => "players",
            MetaField::Rating => "rating",
        }
    }

    /// Reads the stored spelling back.
    pub fn parse(text: &str) -> Result<MetaField, DbError> {
        match text {
            "title" => Ok(MetaField::Title),
            "description" => Ok(MetaField::Description),
            "release_date" => Ok(MetaField::ReleaseDate),
            "developer" => Ok(MetaField::Developer),
            "publisher" => Ok(MetaField::Publisher),
            "genres" => Ok(MetaField::Genres),
            "players" => Ok(MetaField::Players),
            "rating" => Ok(MetaField::Rating),
            other => Err(DbError::InvalidValue(other.to_string())),
        }
    }
}

/// A protected user override for a specific metadata field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaOverride {
    /// The game row id.
    pub game_id: i64,
    /// The protected field.
    pub field: MetaField,
    /// The user-defined value.
    pub value: String,
}

/// Stored metadata of a game.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameMeta {
    /// The game row id.
    pub game_id: i64,
    /// Synopsis or description.
    pub description: Option<String>,
    /// Release date (e.g. YYYY-MM-DD or YYYY).
    pub release_date: Option<String>,
    /// Developer name.
    pub developer: Option<String>,
    /// Publisher name.
    pub publisher: Option<String>,
    /// Comma-separated genres.
    pub genres: Option<String>,
    /// Number of players.
    pub players: Option<u32>,
    /// Game rating (0.0 - 5.0 or 0 - 100).
    pub rating: Option<f32>,
    /// Source that populated this row.
    pub source: String,
    /// Timestamp when metadata was saved, Unix seconds.
    pub scraped_at: i64,
}

/// Scraping mode defining which fields to populate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrapeMode {
    /// Only query and populate fields that are currently missing.
    MissingOnly,
    /// Populate specific chosen fields.
    ChosenFields,
    /// Force overwrite of all fields (except user-protected fields unless requested).
    ForceAll,
}

impl ScrapeMode {
    /// String identifier for settings.
    pub fn as_str(self) -> &'static str {
        match self {
            ScrapeMode::MissingOnly => "missing_only",
            ScrapeMode::ChosenFields => "chosen_fields",
            ScrapeMode::ForceAll => "force_all",
        }
    }

    /// Parses string representation.
    pub fn parse(text: &str) -> Result<ScrapeMode, DbError> {
        match text {
            "missing_only" => Ok(ScrapeMode::MissingOnly),
            "chosen_fields" => Ok(ScrapeMode::ChosenFields),
            "force_all" => Ok(ScrapeMode::ForceAll),
            other => Err(DbError::InvalidValue(other.to_string())),
        }
    }
}

/// Metadata source identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum ScraperSource {
    /// SteamGridDB API for artwork.
    #[default]
    SteamGridDb,
    /// EmulationStation ES-DE gamelist.xml.
    Esde,
    /// User manual edit.
    User,
}

impl ScraperSource {
    /// Stable string spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            ScraperSource::SteamGridDb => "steamgriddb",
            ScraperSource::Esde => "esde",
            ScraperSource::User => "user",
        }
    }

    /// Parses source string.
    pub fn parse(text: &str) -> Result<ScraperSource, DbError> {
        match text {
            "steamgriddb" => Ok(ScraperSource::SteamGridDb),
            "esde" => Ok(ScraperSource::Esde),
            "user" => Ok(ScraperSource::User),
            other => Err(DbError::InvalidValue(other.to_string())),
        }
    }
}
