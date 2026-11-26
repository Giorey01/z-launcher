#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use zl_archive::ArchiveExtractor;

#[test]
fn test_extract_zip_extracts_contents_properly() {
    let dir = tempdir().expect("tempdir");
    let zip_path = dir.path().join("game.zip");

    {
        let file = File::create(&zip_path).expect("create zip");
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("game.wux", options).expect("start file");
        zip.write_all(b"wux content 123").expect("write bytes");
        zip.finish().expect("finish zip");
    }

    let extracted = ArchiveExtractor::extract(&zip_path, dir.path(), None).expect("extract");
    assert_eq!(extracted.len(), 1);
    assert_eq!(
        extracted[0].file_name().and_then(|s| s.to_str()),
        Some("game.wux")
    );
    let content = fs::read_to_string(&extracted[0]).expect("read content");
    assert_eq!(content, "wux content 123");
}

#[test]
fn test_extract_zip_keeps_folders_and_rollback_on_broken() {
    let dir = tempdir().expect("tempdir");
    let zip_path = dir.path().join("game.zip");

    {
        let file = File::create(&zip_path).expect("create zip");
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("Disc 1/track.bin", options).expect("disc 1");
        zip.write_all(b"one").expect("write disc 1");
        zip.start_file("Disc 2/track.bin", options).expect("disc 2");
        zip.write_all(&vec![b'2'; 10000]).expect("write disc 2");
        zip.finish().expect("finish");
    }

    // First: valid extraction keeps folders
    let extracted = ArchiveExtractor::extract(&zip_path, dir.path(), None).expect("extract");
    assert_eq!(extracted.len(), 2);
    let disc1 = dir.path().join("Disc 1/track.bin");
    assert_eq!(fs::read_to_string(&disc1).expect("read disc 1"), "one");

    // Clean up extracted files
    let _ = fs::remove_dir_all(dir.path().join("Disc 1"));
    let _ = fs::remove_dir_all(dir.path().join("Disc 2"));

    // Corrupt the zip file
    let mut bytes = fs::read(&zip_path).expect("read zip");
    let corrupt_start = bytes.len() / 2;
    for b in &mut bytes[corrupt_start..] {
        *b = 0xff;
    }
    fs::write(&zip_path, &bytes).expect("write corrupted zip");

    // Extraction must fail and leave no partial files behind
    let res = ArchiveExtractor::extract(&zip_path, dir.path(), None);
    assert!(res.is_err(), "extraction of corrupted zip must fail");
    assert!(!disc1.exists(), "partial files must be removed on rollback");
}

#[test]
fn test_auto_extract_success_deletes_archive() {
    let dir = tempdir().expect("tempdir");
    let zip_path = dir.path().join("captain-toad.zip");

    {
        let file = File::create(&zip_path).expect("create zip");
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("Captain Toad.wux", options)
            .expect("start file");
        zip.write_all(b"Captain Toad WUX Rom Data")
            .expect("write bytes");
        zip.finish().expect("finish zip");
    }

    assert!(zip_path.exists());
    let extracted =
        ArchiveExtractor::auto_extract(&zip_path, dir.path(), true).expect("auto extract");
    assert_eq!(extracted.len(), 1);
    assert!(
        !zip_path.exists(),
        "zip file must be deleted when configured"
    );
    let extracted_wux = dir.path().join("Captain Toad.wux");
    assert!(extracted_wux.exists());
    assert_eq!(
        fs::read_to_string(extracted_wux).expect("read wux"),
        "Captain Toad WUX Rom Data"
    );
}

#[test]
fn test_auto_extract_failure_keeps_archive() {
    let dir = tempdir().expect("tempdir");
    let broken_zip = dir.path().join("broken.zip");
    fs::write(&broken_zip, b"PK\x03\x04 not really a zip archive").expect("write broken");

    let res = ArchiveExtractor::auto_extract(&broken_zip, dir.path(), true);
    assert!(res.is_err(), "broken zip extraction must fail");
    assert!(
        broken_zip.exists(),
        "broken zip must be kept even with delete_archive = true"
    );
}
