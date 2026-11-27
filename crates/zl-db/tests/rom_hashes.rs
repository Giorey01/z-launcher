//! Tests of the `rom_hashes` repository: which files are pending, replacement, staleness.

use zl_db::{Db, GameFile, GameKind, NewGame, NewRomHash};

/// Result type of the tests: any error fails the test.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A game with the given member files (path, size, mtime).
fn game(path: &str, files: &[(&str, i64, i64)]) -> NewGame {
    NewGame {
        root_id: 1,
        platform_id: "nes".into(),
        title: path.into(),
        path: path.into(),
        kind: GameKind::Rom,
        size: 10,
        mtime_ms: 5,
        fingerprint: 1,
        files: files
            .iter()
            .map(|(p, size, mtime)| GameFile {
                path: (*p).into(),
                size: *size,
                mtime_ms: *mtime,
            })
            .collect(),
        tags: vec![],
    }
}

/// A row for a plain file hashed at the given size and mtime.
fn row(size: i64, mtime: i64, crc: &str) -> NewRomHash {
    NewRomHash {
        file_size: size,
        file_mtime: mtime,
        container: "raw".into(),
        size,
        crc32: Some(crc.into()),
        md5: Some("m".repeat(32)),
        sha1: Some("s".repeat(40)),
        ..NewRomHash::default()
    }
}

/// A database with one root and the games `a` (one file) and `b` (a disc and a playlist).
fn library() -> Result<(Db, i64, i64), Box<dyn std::error::Error>> {
    let db = Db::open_in_memory()?;
    db.upsert_root("/roms", true)?;
    let a = db.upsert_game(&game("/roms/a.nes", &[("/roms/a.nes", 100, 7)]), 1)?;
    let files = [("/roms/b.cue", 10, 1), ("/roms/b.bin", 500, 2)];
    let b = db.upsert_game(&game("/roms/b.cue", &files), 1)?;
    Ok((db, a, b))
}

/// Files without a current row are pending; the skipped extensions are not counted.
#[test]
fn pending_files_and_coverage() -> TestResult {
    let (db, a, b) = library()?;
    let all = db.files_to_hash(&[])?;
    assert_eq!(all.len(), 3);
    let pending = db.files_to_hash(&["cue"])?;
    let paths: Vec<&str> = pending.iter().map(|(_, f)| f.path.as_str()).collect();
    assert_eq!(paths, ["/roms/a.nes", "/roms/b.bin"]);
    assert_eq!(db.hash_coverage(&["cue"])?, (2, 0));
    db.replace_file_hashes(a, "/roms/a.nes", &[row(100, 7, "aaaaaaaa")])?;
    assert_eq!(db.hash_coverage(&["cue"])?, (2, 1));
    db.replace_file_hashes(b, "/roms/b.bin", &[row(500, 2, "bbbbbbbb")])?;
    assert_eq!(db.hash_coverage(&["cue"])?, (2, 2));
    assert!(db.files_to_hash(&["cue"])?.is_empty());
    Ok(())
}

/// A file whose size or mtime changed is pending again, and its old row is not returned.
#[test]
fn changed_file_is_pending_again() -> TestResult {
    let (db, a, _) = library()?;
    db.replace_file_hashes(a, "/roms/a.nes", &[row(100, 7, "aaaaaaaa")])?;
    assert_eq!(db.game_hashes(a)?.len(), 1);
    // The scanner sees a new mtime for the same file.
    db.upsert_game(&game("/roms/a.nes", &[("/roms/a.nes", 100, 8)]), 2)?;
    assert!(db.game_hashes(a)?.is_empty());
    let pending = db.files_to_hash(&["cue"])?;
    assert!(pending.iter().any(|(id, f)| *id == a && f.mtime_ms == 8));
    Ok(())
}

/// Replacing a file's rows swaps its entries, and games_with_digest finds either digest kind.
#[test]
fn replace_and_lookup() -> TestResult {
    let (db, a, b) = library()?;
    let mut inner = row(100, 7, "11111111");
    inner.entry = "x/one.nes".into();
    inner.hl_crc32 = Some("22222222".into());
    inner.header_bytes = 16;
    let mut other = row(100, 7, "33333333");
    other.entry = "two.nes".into();
    db.replace_file_hashes(a, "/roms/a.nes", &[inner, other])?;
    assert_eq!(db.game_hashes(a)?.len(), 2);
    assert_eq!(db.games_with_digest("crc32", "11111111")?, [a]);
    assert_eq!(
        db.games_with_digest("crc32", "22222222")?,
        [a],
        "headerless"
    );
    assert!(db.games_with_digest("crc32", "44444444")?.is_empty());
    assert!(db.games_with_digest("size", "1").is_err());
    db.replace_file_hashes(a, "/roms/a.nes", &[row(100, 7, "55555555")])?;
    let rows = db.game_hashes(a)?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].crc32.as_deref(), Some("55555555"));
    assert!(db.game_hashes(b)?.is_empty());
    Ok(())
}

/// A file that failed for a lasting reason is not pending again until it changes.
#[test]
fn lasting_error_is_remembered() -> TestResult {
    let (db, _, b) = library()?;
    let failed = NewRomHash {
        file_size: 500,
        file_mtime: 2,
        container: "zip".into(),
        error: Some("password".into()),
        ..NewRomHash::default()
    };
    db.replace_file_hashes(b, "/roms/b.bin", &[failed])?;
    let pending = db.files_to_hash(&["cue"])?;
    assert!(pending.iter().all(|(_, f)| f.path != "/roms/b.bin"));
    let stored = db.game_hashes(b)?;
    assert_eq!(stored[0].error.as_deref(), Some("password"));
    assert_eq!(stored[0].crc32, None);
    Ok(())
}
