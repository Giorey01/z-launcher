//! What was added over the extraction: zip entries that name no file or start with `/`, and the
//! progress and stop request of zip and 7z.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use zl_archive::{ArchiveError, ArchiveExtractor, ExtractOptions};

/// A zip at `path` with the entries `(name, content)`; a name ending in `/` is a folder.
fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let mut zip = ZipWriter::new(File::create(path).unwrap());
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        if name.ends_with('/') {
            zip.add_directory(*name, opts).unwrap();
        } else {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
    }
    zip.finish().unwrap();
}

#[test]
fn a_root_entry_is_skipped_and_a_leading_slash_dropped() {
    let dir = tempdir().unwrap();
    let archive = dir.path().join("mod.zip");
    write_zip(
        &archive,
        &[
            ("/", b""),
            ("/Mod/romfs/a.bin", b"abc"),
            ("Mod/exefs/", b""),
        ],
    );
    let out = dir.path().join("out");
    let files = ArchiveExtractor::extract(&archive, &out, None).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(fs::read(out.join("Mod/romfs/a.bin")).unwrap(), b"abc");
    assert!(out.join("Mod/exefs").is_dir());
}

#[test]
fn an_entry_that_climbs_out_is_still_refused() {
    let dir = tempdir().unwrap();
    let archive = dir.path().join("bad.zip");
    write_zip(&archive, &[("ok.txt", b"1"), ("../evil.txt", b"2")]);
    let out = dir.path().join("out");
    let err = ArchiveExtractor::extract(&archive, &out, None).unwrap_err();
    assert!(matches!(err, ArchiveError::SecurityError(_)), "{err}");
    assert!(!dir.path().join("evil.txt").exists());
    assert!(!out.join("ok.txt").exists(), "rolled back");
}

/// Options whose progress keeps the last count in `seen`, with the stop flag `cancel`.
fn watched(seen: &Arc<AtomicU64>, cancel: &Arc<AtomicBool>) -> ExtractOptions {
    let s = seen.clone();
    ExtractOptions {
        progress: Some(Arc::new(move |done, total| {
            assert!(done <= total, "{done} > {total}");
            s.store(done, Ordering::Relaxed);
        })),
        cancel: Some(cancel.clone()),
        ..ExtractOptions::default()
    }
}

#[test]
fn a_zip_tells_its_progress_and_stops_when_asked() {
    let dir = tempdir().unwrap();
    let archive = dir.path().join("mod.zip");
    write_zip(
        &archive,
        &[("a.bin", &[1u8; 5000]), ("b.bin", &[2u8; 3000])],
    );
    let (seen, cancel) = (
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicBool::new(false)),
    );
    let out = dir.path().join("out");
    ArchiveExtractor::extract_with(&archive, &out, &watched(&seen, &cancel)).unwrap();
    assert_eq!(seen.load(Ordering::Relaxed), 8000);
    cancel.store(true, Ordering::Relaxed);
    let again = dir.path().join("again");
    let err =
        ArchiveExtractor::extract_with(&archive, &again, &watched(&seen, &cancel)).unwrap_err();
    assert!(matches!(err, ArchiveError::Cancelled), "{err}");
    assert!(!again.join("a.bin").exists());
}

#[test]
fn a_7z_tells_its_progress_and_stops_when_asked() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src/Mod");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("a.bin"), vec![3u8; 4000]).unwrap();
    fs::write(src.join("b.bin"), vec![4u8; 1000]).unwrap();
    let archive = dir.path().join("mod.7z");
    sevenz_rust2::compress_to_path(dir.path().join("src"), &archive).unwrap();
    let (seen, cancel) = (
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicBool::new(false)),
    );
    let out = dir.path().join("out");
    ArchiveExtractor::extract_with(&archive, &out, &watched(&seen, &cancel)).unwrap();
    assert_eq!(seen.load(Ordering::Relaxed), 5000);
    assert_eq!(fs::read(out.join("Mod/a.bin")).unwrap().len(), 4000);
    cancel.store(true, Ordering::Relaxed);
    let again = dir.path().join("again");
    let err =
        ArchiveExtractor::extract_with(&archive, &again, &watched(&seen, &cancel)).unwrap_err();
    assert!(matches!(err, ArchiveError::Cancelled), "{err}");
    assert!(!again.join("Mod/a.bin").exists());
}
