//! Runs `hash_file` on the shared fixtures and compares with the digests Python computed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use serde::Deserialize;
use zl_hash::{hash_file, Digests, EntryHash, HashError, Limits};

/// The expected file of the fixtures.
#[derive(Deserialize)]
struct Expected {
    /// One case per file.
    cases: Vec<Case>,
}

/// What `hash_file` must answer for one file.
#[derive(Deserialize)]
struct Case {
    /// File name in `fixtures/hash`.
    file: String,
    /// Container spelling, for a success.
    container: Option<String>,
    /// Entries, for a success.
    entries: Option<Vec<Item>>,
    /// Error code, for a failure.
    error: Option<String>,
}

/// One expected entry.
#[derive(Deserialize)]
struct Item {
    /// Entry name inside the archive.
    entry: String,
    /// Size in bytes.
    size: u64,
    /// Digests of the whole content.
    crc32: Option<String>,
    /// MD5.
    md5: Option<String>,
    /// SHA1.
    sha1: Option<String>,
    /// Digests without the header.
    headerless: Option<Headerless>,
}

/// The expected digests without the header.
#[derive(Deserialize)]
struct Headerless {
    /// Bytes of header.
    header_bytes: u32,
    /// CRC32.
    crc32: String,
    /// MD5.
    md5: String,
    /// SHA1.
    sha1: String,
}

/// Folder of the fixtures.
fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/hash")
}

/// The digests as the fixtures spell them.
fn digests(d: &Digests) -> (Option<&str>, Option<&str>, Option<&str>) {
    (d.crc32.as_deref(), d.md5.as_deref(), d.sha1.as_deref())
}

/// Checks one entry against the expectation.
fn check_entry(file: &str, got: &EntryHash, want: &Item) {
    assert_eq!(got.entry, want.entry, "{file}: entry name");
    assert_eq!(got.size, want.size, "{file} {}: size", want.entry);
    let full = (
        want.crc32.as_deref(),
        want.md5.as_deref(),
        want.sha1.as_deref(),
    );
    assert_eq!(digests(&got.full), full, "{file} {}: digests", want.entry);
    match (&got.headerless, &want.headerless) {
        (None, None) => {}
        (Some(g), Some(w)) => {
            assert_eq!(g.header_bytes, w.header_bytes, "{file}: header bytes");
            let d = (Some(w.crc32.as_str()), Some(w.md5.as_str()));
            assert_eq!((g.digests.crc32.as_deref(), g.digests.md5.as_deref()), d);
            assert_eq!(g.digests.sha1.as_deref(), Some(w.sha1.as_str()));
        }
        _ => panic!("{file}: headerless presence differs"),
    }
}

/// Every fixture hashes to the digests written next to it, or fails with its code.
#[test]
fn fixtures_match_the_expected_digests() {
    let text = std::fs::read_to_string(dir().join("expected.json")).unwrap();
    let expected: Expected = serde_json::from_str(&text).unwrap();
    assert!(expected.cases.len() >= 10);
    for case in expected.cases {
        let result = hash_file(&dir().join(&case.file), &Limits::default(), &|| false);
        match (result, case.error, case.entries) {
            (Ok(mut done), None, Some(mut items)) => {
                // 7z lists its entries in an order of its own: compare by name.
                done.entries.sort_by(|a, b| a.entry.cmp(&b.entry));
                items.sort_by(|a, b| a.entry.cmp(&b.entry));
                assert_eq!(Some(done.container.as_str().to_string()), case.container);
                assert_eq!(done.entries.len(), items.len(), "{}: entries", case.file);
                for (got, want) in done.entries.iter().zip(&items) {
                    check_entry(&case.file, got, want);
                }
            }
            (Err(err), Some(code), _) => assert_eq!(err.code(), code, "{}", case.file),
            (other, _, _) => panic!("{}: unexpected {:?}", case.file, other.map(|_| ())),
        }
    }
}

/// A stop request ends the work with `Cancelled`, whatever the container.
#[test]
fn stop_request_cancels() {
    for name in ["plain.bin", "set.zip", "set.7z"] {
        let err = hash_file(&dir().join(name), &Limits::default(), &|| true).unwrap_err();
        assert!(matches!(err, HashError::Cancelled), "{name}: {err}");
    }
}

/// Limits turn a file or an entry that is too large into `too_large`, and cap the entry count.
#[test]
fn limits_are_enforced() {
    let small = Limits {
        max_file_bytes: 100,
        max_archive_bytes: 100,
        max_entry_bytes: 100,
        ..Limits::default()
    };
    for name in ["plain.bin", "set.zip", "set.7z"] {
        let err = hash_file(&dir().join(name), &small, &|| false).unwrap_err();
        assert_eq!(err.code(), "too_large", "{name}");
    }
    // Only the entry under the limit is hashed; the other one is counted as skipped.
    let one = Limits {
        max_entry_bytes: 100,
        ..Limits::default()
    };
    let zip = hash_file(&dir().join("set.zip"), &one, &|| false).unwrap();
    assert_eq!((zip.entries.len(), zip.skipped), (1, 1));
    assert_eq!(zip.entries[0].entry, "roms/a.nes");
    let cap = Limits {
        max_entries: 1,
        ..Limits::default()
    };
    let seven = hash_file(&dir().join("set.7z"), &cap, &|| false).unwrap();
    assert!(seven.truncated);
    assert_eq!(seven.entries.len(), 1);
}

/// A file that does not exist is an I/O error, which a later run may retry.
#[test]
fn missing_file_is_transient() {
    let err = hash_file(&dir().join("nope.bin"), &Limits::default(), &|| false).unwrap_err();
    assert!(err.is_transient());
    assert_eq!(err.code(), "io");
}
