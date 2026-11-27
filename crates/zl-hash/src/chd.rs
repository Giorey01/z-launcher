//! Reading the header of a CHD (Compressed Hunks of Data) disc image: it declares the SHA1 of
//! the raw data, so the image is identified without decompressing a byte.
//!
//! Facts of the format, from the public description published by the MAME project (BSD-3-Clause
//! licensed; only the layout is used here, written in our own words). Every number is big-endian.
//! All versions start with the 8 bytes `MComprHD`, then a 32-bit header length and a 32-bit
//! version. What changes is where the fields sit:
//!
//! | version | header length | logical size (u64) | SHA1 of the raw data | other digests |
//! | --- | --- | --- | --- | --- |
//! | 3 | 120 | at 28 | at 80 | MD5 at 44 |
//! | 4 | 108 | at 28 | at 88 (`rawsha1`) | SHA1 with metadata at 48 |
//! | 5 | 124 | at 32 | at 64 (`rawsha1`) | SHA1 with metadata at 84 |
//!
//! Version 1 and 2 headers are older than any image in use today and are reported unsupported.

use std::io::Read;

use crate::digest::{to_hex, Digests};
use crate::error::HashError;
use crate::{Container, EntryHash, FileHashes};

/// The first 8 bytes of every CHD.
const TAG: &[u8; 8] = b"MComprHD";

/// Size of the longest header read (version 5).
const MAX_HEADER: usize = 124;

/// Where the fields sit in one version of the header.
struct Layout {
    /// Version number.
    version: u32,
    /// Declared header length in bytes.
    length: u32,
    /// Offset of the logical size (u64).
    logical: usize,
    /// Offset of the SHA1 of the raw data.
    raw_sha1: usize,
    /// Offset of the MD5 of the raw data, in the one version that has it.
    md5: Option<usize>,
}

/// The layouts of the versions that are read.
const LAYOUTS: [Layout; 3] = [
    Layout {
        version: 3,
        length: 120,
        logical: 28,
        raw_sha1: 80,
        md5: Some(44),
    },
    Layout {
        version: 4,
        length: 108,
        logical: 28,
        raw_sha1: 88,
        md5: None,
    },
    Layout {
        version: 5,
        length: 124,
        logical: 32,
        raw_sha1: 64,
        md5: None,
    },
];

/// Reads a big-endian u32 at `at`.
fn be32(head: &[u8], at: usize) -> u32 {
    let mut b = [0u8; 4];
    b.copy_from_slice(&head[at..at + 4]);
    u32::from_be_bytes(b)
}

/// Reads a big-endian u64 at `at`.
fn be64(head: &[u8], at: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&head[at..at + 8]);
    u64::from_be_bytes(b)
}

/// Reads as many bytes as the file has, up to the buffer.
fn read_head(file: &mut dyn Read, buf: &mut [u8]) -> Result<usize, HashError> {
    let mut total = 0;
    while total < buf.len() {
        match file.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(HashError::Io(e)),
        }
    }
    Ok(total)
}

/// Reads the header of a CHD file and returns the SHA1 of its raw data, with the logical size.
pub(crate) fn hash_chd(file: &mut dyn Read) -> Result<FileHashes, HashError> {
    let mut head = [0u8; MAX_HEADER];
    let got = read_head(file, &mut head)?;
    if got < 16 || &head[..8] != TAG {
        return Err(HashError::Damaged);
    }
    let (length, version) = (be32(&head, 8), be32(&head, 12));
    let layout = LAYOUTS
        .iter()
        .find(|l| l.version == version)
        .ok_or(HashError::Unsupported)?;
    if length != layout.length || got < layout.length as usize {
        return Err(HashError::Damaged);
    }
    let sha1 = &head[layout.raw_sha1..layout.raw_sha1 + 20];
    if sha1.iter().all(|b| *b == 0) {
        return Err(HashError::Damaged);
    }
    let md5 = layout.md5.map(|at| to_hex(&head[at..at + 16]));
    Ok(FileHashes {
        container: Container::Chd,
        entries: vec![EntryHash {
            entry: String::new(),
            size: be64(&head, layout.logical),
            full: Digests {
                crc32: None,
                md5,
                sha1: Some(to_hex(sha1)),
            },
            headerless: None,
        }],
        skipped: 0,
        truncated: false,
    })
}
