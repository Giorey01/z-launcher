#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use zl_archive::ArchiveExtractor;

#[test]
fn test_extract_valid_zip() {
    let dir = tempdir().expect("tempdir");
    let zip_path = dir.path().join("test.zip");
    let dest_dir = dir.path().join("extracted");

    // Create a valid zip
    {
        let file = File::create(&zip_path).expect("create zip");
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("hello.txt", options).expect("start file");
        zip.write_all(b"Hello from Rust zip!").expect("write file");

        zip.start_file("sub/nested.txt", options)
            .expect("start nested");
        zip.write_all(b"Nested content").expect("write nested");

        zip.finish().expect("finish zip");
    }

    // Extract
    let extracted =
        ArchiveExtractor::extract(&zip_path, &dest_dir, None).expect("extract should succeed");

    assert_eq!(extracted.len(), 2);
    let hello = std::fs::read_to_string(dest_dir.join("hello.txt")).expect("read hello");
    assert_eq!(hello, "Hello from Rust zip!");

    let nested = std::fs::read_to_string(dest_dir.join("sub/nested.txt")).expect("read nested");
    assert_eq!(nested, "Nested content");
}

#[test]
fn test_unsupported_format() {
    let dir = tempdir().expect("tempdir");
    let fake_path = dir.path().join("test.unknown");
    File::create(&fake_path).expect("create file");

    let res = ArchiveExtractor::extract(&fake_path, dir.path(), None);
    assert!(res.is_err());
}
