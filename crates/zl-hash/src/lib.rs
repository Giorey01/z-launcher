//! CRC32, MD5 and SHA1 of ROM files, to match them with the DATs of No-Intro and Redump and the
//! `libretro-database`. One pass over the data gives all three digests.
//!
//! What is hashed:
//! - a plain file: the whole content, and, for the formats that carry a copier or loader header
//!   the DATs leave out (iNES, Lynx, Atari 7800, SNES copiers), also the content without it;
//! - a `.zip` or `.7z`: every entry, decompressed on the fly (nothing is written to disk);
//! - a `.chd`: only the header is read, which declares the SHA1 of the raw data, so a disc
//!   image of gigabytes costs a few bytes of reading.
//!
//! The crate is synchronous and knows nothing of the library or the database: [`hash_file`]
//! takes a path and returns the digests, with a stop request checked between chunks.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

mod chd;
mod digest;
mod error;
mod header;
mod sevenz;
mod stream;
mod zipfile;

use std::fs::File;
use std::path::Path;

pub use digest::Digests;
pub use error::HashError;
pub use header::header_len;
pub use stream::Limits;

/// How a file stores its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// A plain file.
    Raw,
    /// A zip archive.
    Zip,
    /// A 7z archive.
    SevenZ,
    /// A CHD disc image (only its header is read).
    Chd,
}

impl Container {
    /// Stable spelling stored in the database and shared with the fixtures.
    pub fn as_str(self) -> &'static str {
        match self {
            Container::Raw => "raw",
            Container::Zip => "zip",
            Container::SevenZ => "7z",
            Container::Chd => "chd",
        }
    }

    /// Reads the stored spelling back.
    pub fn parse(text: &str) -> Option<Container> {
        match text {
            "raw" => Some(Container::Raw),
            "zip" => Some(Container::Zip),
            "7z" => Some(Container::SevenZ),
            "chd" => Some(Container::Chd),
            _ => None,
        }
    }

    /// The container a file name suggests, by extension.
    pub fn of_path(path: &Path) -> Container {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match ext.as_str() {
            "zip" => Container::Zip,
            "7z" => Container::SevenZ,
            "chd" => Container::Chd,
            _ => Container::Raw,
        }
    }
}

/// The digests of the content without its leading header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Headerless {
    /// Bytes of header left out.
    pub header_bytes: u32,
    /// Digests of what follows the header.
    pub digests: Digests,
}

/// The digests of one file, or of one entry of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryHash {
    /// Path of the entry inside the archive; empty for a plain file or a CHD.
    pub entry: String,
    /// Size in bytes of the content (of the logical data, for a CHD).
    pub size: u64,
    /// Digests of the whole content. A CHD only carries a SHA1.
    pub full: Digests,
    /// Digests without the header, for the formats that have one to leave out.
    pub headerless: Option<Headerless>,
}

/// Everything [`hash_file`] found in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHashes {
    /// How the file stores its content.
    pub container: Container,
    /// One item per file inside (one for a plain file or a CHD).
    pub entries: Vec<EntryHash>,
    /// Entries left out because they were too large to be hashed.
    pub skipped: u32,
    /// True when the archive holds more entries than the limit and the rest was not read.
    pub truncated: bool,
}

/// Hashes the file at `path` as its extension says (`.zip`, `.7z`, `.chd`, anything else plain).
/// `stop` is asked between chunks; when it answers `true` the call ends with
/// [`HashError::Cancelled`].
pub fn hash_file(
    path: &Path,
    limits: &Limits,
    stop: &dyn Fn() -> bool,
) -> Result<FileHashes, HashError> {
    let container = Container::of_path(path);
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    match container {
        Container::Chd => chd::hash_chd(&mut file),
        Container::Raw => stream::hash_plain(&mut file, path, size, limits, stop),
        Container::Zip => zipfile::hash_zip(file, size, limits, stop),
        Container::SevenZ => sevenz::hash_sevenz(file, size, limits, stop),
    }
}
