//! The result of reading a media folder.

use std::path::PathBuf;

use zl_db::{MediaKind, MediaSource};

/// One media file found on disk, not yet tied to a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundMedia {
    /// Name of the system or platform folder it was in, lower case (an ES-DE id or a Cocoon id;
    /// the caller maps it to a platform with the aliases).
    pub platform: String,
    /// File stem (the ROM name the file belongs to), lower case, without the Cocoon edit suffix.
    pub stem: String,
    /// What the file shows.
    pub kind: MediaKind,
    /// Absolute path.
    pub path: PathBuf,
    /// Which reader found it.
    pub source: MediaSource,
    /// Number of the Cocoon edit, `None` for a plain file.
    pub edit: Option<u32>,
}
