//! Hashing every entry of a zip archive, decompressed on the fly.

use std::fs::File;
use std::io::BufReader;

use zip::result::ZipError;
use zip::ZipArchive;
use zl_archive::{ArchiveError, ArchiveFailure};

use crate::error::HashError;
use crate::stream::{hash_reader, non_empty, Limits};
use crate::{Container, FileHashes};

/// Maps the failure of the zip reader to the code the user can act on.
fn zip_error(err: ZipError) -> HashError {
    match err {
        ZipError::Io(io) => HashError::from_read(io),
        other => match ArchiveError::from(other).failure() {
            ArchiveFailure::Password => HashError::Password,
            _ => HashError::Damaged,
        },
    }
}

/// Hashes the files of a zip (`size` is the archive's own size). An entry too large is left out
/// and counted; an encrypted or damaged entry fails the whole archive.
pub(crate) fn hash_zip(
    file: File,
    size: u64,
    limits: &Limits,
    stop: &dyn Fn() -> bool,
) -> Result<FileHashes, HashError> {
    if size > limits.max_archive_bytes {
        return Err(HashError::TooLarge);
    }
    let mut zip = ZipArchive::new(BufReader::new(file)).map_err(zip_error)?;
    let mut out = FileHashes {
        container: Container::Zip,
        entries: Vec::new(),
        skipped: 0,
        truncated: false,
    };
    for index in 0..zip.len() {
        if stop() {
            return Err(HashError::Cancelled);
        }
        let mut entry = zip.by_index(index).map_err(zip_error)?;
        if entry.is_dir() {
            continue;
        }
        if out.entries.len() >= limits.max_entries {
            out.truncated = true;
            break;
        }
        if entry.size() > limits.max_entry_bytes {
            out.skipped += 1;
            continue;
        }
        let name = entry.name().to_string();
        let sizes = (entry.size(), limits.max_entry_bytes);
        match hash_reader(&mut entry, &name, sizes, limits, stop) {
            Ok(item) => out.entries.push(item),
            Err(HashError::TooLarge) => out.skipped += 1,
            Err(other) => return Err(other),
        }
    }
    non_empty(out)
}
