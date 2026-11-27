//! Hashing a stream of bytes in one pass: the whole content and, when the format has a header
//! to leave out, the content after it.

use std::io::Read;
use std::path::Path;

use crate::digest::Hasher;
use crate::error::HashError;
use crate::header::{header_len, PROBE_BYTES};
use crate::{Container, EntryHash, FileHashes, Headerless};

/// The bounds of a run; the defaults come from `zl_config::hashing`.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Largest plain file hashed.
    pub max_file_bytes: u64,
    /// Largest archive file opened.
    pub max_archive_bytes: u64,
    /// Largest single archive entry hashed once decompressed.
    pub max_entry_bytes: u64,
    /// Most entries hashed from one archive.
    pub max_entries: usize,
    /// Bytes read at a time.
    pub chunk_bytes: usize,
    /// Memory a 7z decoder may reserve, in KiB.
    pub max_7z_memory_kb: usize,
}

impl Default for Limits {
    /// The limits of `zl_config::hashing`.
    fn default() -> Limits {
        use zl_config::hashing as cfg;
        Limits {
            max_file_bytes: cfg::MAX_FILE_BYTES,
            max_archive_bytes: cfg::MAX_ARCHIVE_BYTES,
            max_entry_bytes: cfg::MAX_ENTRY_BYTES,
            max_entries: cfg::MAX_ARCHIVE_ENTRIES,
            chunk_bytes: cfg::READ_CHUNK_BYTES,
            max_7z_memory_kb: cfg::MAX_7Z_MEMORY_KB,
        }
    }
}

/// Reads until `buf` is full or the stream ends; returns the bytes read.
fn fill(reader: &mut dyn Read, buf: &mut [u8]) -> Result<usize, HashError> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(HashError::from_read(e)),
        }
    }
    Ok(total)
}

/// The two running digests of a stream: the whole content and the part after the header.
struct Pair {
    /// Digest of everything.
    full: Hasher,
    /// Digest of what follows the header, when the format has one.
    after: Option<(u32, Hasher)>,
    /// Bytes fed so far.
    seen: u64,
}

impl Pair {
    /// Feeds one chunk to the digests, leaving the header out of the second one.
    fn feed(&mut self, chunk: &[u8]) {
        self.full.update(chunk);
        if let Some((header, hasher)) = &mut self.after {
            let skip = u64::from(*header).saturating_sub(self.seen) as usize;
            if skip < chunk.len() {
                hasher.update(&chunk[skip..]);
            }
        }
        self.seen += chunk.len() as u64;
    }
}

/// Hashes `reader` to its end. `name` picks the header rule by extension and `size` is the size
/// the container declares (the SNES rule needs it before the end is reached). `entry` is the
/// name recorded in the result. The stream is read up to `max_bytes`; one more byte is
/// [`HashError::TooLarge`] (the declared size of an archive entry can lie).
pub(crate) fn hash_reader(
    reader: &mut dyn Read,
    entry: &str,
    (size, max_bytes): (u64, u64),
    limits: &Limits,
    stop: &dyn Fn() -> bool,
) -> Result<EntryHash, HashError> {
    let mut buf = vec![0u8; limits.chunk_bytes.max(PROBE_BYTES)];
    let mut got = fill(reader, &mut buf)?;
    let header = header_len(entry, size, &buf[..got.min(PROBE_BYTES)]);
    let mut pair = Pair {
        full: Hasher::new(),
        after: (header > 0).then(|| (header, Hasher::new())),
        seen: 0,
    };
    loop {
        if stop() {
            return Err(HashError::Cancelled);
        }
        if got == 0 {
            break;
        }
        pair.feed(&buf[..got]);
        if pair.seen > max_bytes {
            return Err(HashError::TooLarge);
        }
        got = fill(reader, &mut buf)?;
    }
    Ok(EntryHash {
        entry: entry.to_string(),
        size: pair.seen,
        full: pair.full.finish(),
        headerless: pair.after.map(|(header_bytes, hasher)| Headerless {
            header_bytes,
            digests: hasher.finish(),
        }),
    })
}

/// Hashes a plain file.
pub(crate) fn hash_plain(
    reader: &mut dyn Read,
    path: &Path,
    size: u64,
    limits: &Limits,
    stop: &dyn Fn() -> bool,
) -> Result<FileHashes, HashError> {
    if size > limits.max_file_bytes {
        return Err(HashError::TooLarge);
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let mut item = hash_reader(reader, name, (size, limits.max_file_bytes), limits, stop)?;
    item.entry.clear();
    Ok(FileHashes {
        container: Container::Raw,
        entries: vec![item],
        skipped: 0,
        truncated: false,
    })
}

/// An archive with nothing hashed is an error: too large when entries were skipped, else empty.
pub(crate) fn non_empty(out: FileHashes) -> Result<FileHashes, HashError> {
    if !out.entries.is_empty() {
        Ok(out)
    } else if out.skipped > 0 {
        Err(HashError::TooLarge)
    } else {
        Err(HashError::Empty)
    }
}
