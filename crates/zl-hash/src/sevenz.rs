//! Hashing every entry of a 7z archive, decompressed on the fly (a solid archive is decoded
//! once, in order).

use std::fs::File;

use sevenz_rust2::{ArchiveReader, EntryDecision, Error as SevenError, Password};
use zl_archive::{ArchiveError, ArchiveFailure};

use crate::error::HashError;
use crate::stream::{hash_reader, non_empty, Limits};
use crate::{Container, FileHashes};

/// Maps the failure of the 7z reader to the code the user can act on.
fn sevenz_error(err: SevenError) -> HashError {
    if let SevenError::Io(io, _) | SevenError::FileOpen(io, _) = err {
        return HashError::from_read(io);
    }
    match ArchiveError::from(err).failure() {
        ArchiveFailure::Password => HashError::Password,
        ArchiveFailure::Damaged => HashError::Damaged,
        _ => HashError::Unsupported,
    }
}

/// The state shared with the callback that visits the entries.
struct Walk<'a> {
    /// What was hashed so far.
    out: FileHashes,
    /// Why the walk was cut short, kept apart because the reader's own error type has no room
    /// for it.
    failure: Option<HashError>,
    /// The bounds.
    limits: &'a Limits,
    /// The stop request.
    stop: &'a dyn Fn() -> bool,
}

impl Walk<'_> {
    /// Hashes one entry of the archive; says whether the walk goes on.
    fn visit(&mut self, name: &str, size: u64, reader: &mut dyn std::io::Read) -> EntryDecision {
        if self.out.entries.len() >= self.limits.max_entries {
            self.out.truncated = true;
            return EntryDecision::Stop;
        }
        if size > self.limits.max_entry_bytes {
            self.out.skipped += 1;
            return EntryDecision::Continue;
        }
        let sizes = (size, self.limits.max_entry_bytes);
        match hash_reader(reader, name, sizes, self.limits, self.stop) {
            Ok(item) => self.out.entries.push(item),
            Err(HashError::TooLarge) => self.out.skipped += 1,
            Err(other) => {
                self.failure = Some(other);
                return EntryDecision::Stop;
            }
        }
        EntryDecision::Continue
    }
}

/// Hashes the files of a 7z (`size` is the archive's own size). An entry too large is left out
/// and counted; an encrypted or damaged archive fails as a whole.
pub(crate) fn hash_sevenz(
    file: File,
    size: u64,
    limits: &Limits,
    stop: &dyn Fn() -> bool,
) -> Result<FileHashes, HashError> {
    if size > limits.max_archive_bytes {
        return Err(HashError::TooLarge);
    }
    let mut reader =
        ArchiveReader::new_with_memory_limit(file, Password::empty(), limits.max_7z_memory_kb)
            .map_err(sevenz_error)?;
    let mut walk = Walk {
        out: FileHashes {
            container: Container::SevenZ,
            entries: Vec::new(),
            skipped: 0,
            truncated: false,
        },
        failure: None,
        limits,
        stop,
    };
    let result = reader.for_each_entries_with_decision(|entry, data| {
        if entry.is_directory() {
            return Ok(EntryDecision::Continue);
        }
        Ok(walk.visit(entry.name(), entry.size(), data))
    });
    if let Some(failure) = walk.failure {
        return Err(failure);
    }
    result.map_err(sevenz_error)?;
    non_empty(walk.out)
}
