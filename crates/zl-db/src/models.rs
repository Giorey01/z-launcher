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
