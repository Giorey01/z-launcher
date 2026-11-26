//! Encrypted zip (AES and the classic ZipCrypto) and 7z archives: no password or a wrong one
//! fails as `ArchiveFailure::Password` and leaves nothing behind; the right one extracts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::{AesMode, ZipWriter};
use zl_archive::{ArchiveExtractor, ArchiveFailure};

/// The password of every archive here.
const PASSWORD: &str = "Secret123";
/// What the one file of every archive holds.
const CONTENT: &[u8] = b"hello password";

/// A zip made by Info-ZIP (`zip -P Secret123`) with ZipCrypto, which the `zip` crate cannot
/// write: one entry, `secret.txt`.
const ZIPCRYPTO_HEX: &str = concat!(
    "504b03040a00090000008c4e765d01e088f81a0000000e0000000a0000007365637265742e747874",
    "0a94f5678d1126bf1a15795d88029aced3e5bd6750b76fd8cefe504b070801e088f81a0000000e00",
    "0000504b01021e030a00090000008c4e765d01e088f81a0000000e0000000a000000000000000100",
    "0000a481000000007365637265742e747874504b0506000000000100010038000000520000000000"
);

/// Writes the ZipCrypto zip into `dir`.
fn zipcrypto(dir: &Path) -> PathBuf {
    let path = dir.join("old.zip");
    fs::write(&path, hex::decode(ZIPCRYPTO_HEX).unwrap()).unwrap();
    path
}

/// Writes a zip with one AES-256 entry, `secret.txt`, into `dir`.
fn aes_zip(dir: &Path) -> PathBuf {
    let path = dir.join("aes.zip");
    let mut zip = ZipWriter::new(File::create(&path).unwrap());
    let options = SimpleFileOptions::default().with_aes_encryption(AesMode::Aes256, PASSWORD);
    zip.start_file("secret.txt", options).unwrap();
    zip.write_all(CONTENT).unwrap();
    zip.finish().unwrap();
    path
}

/// Writes a 7z with its data encrypted (one file, `secret.txt`) into `dir`.
fn sevenz(dir: &Path) -> PathBuf {
    let src = dir.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("secret.txt"), CONTENT).unwrap();
    let path = dir.join("enc.7z");
    sevenz_rust2::compress_to_path_encrypted(&src, &path, PASSWORD.into()).unwrap();
    path
}

/// Checks the three outcomes on `archive`: none, wrong and right password.
fn check_passwords(archive: &Path) {
    let out = archive.parent().unwrap().join("out");
    for wrong in [None, Some("nope")] {
        let err = ArchiveExtractor::extract(archive, &out, wrong).unwrap_err();
        assert_eq!(err.failure(), ArchiveFailure::Password, "{wrong:?}: {err}");
        assert!(
            !out.join("secret.txt").exists(),
            "{wrong:?} left a file behind"
        );
    }
    let files = ArchiveExtractor::extract(archive, &out, Some(PASSWORD)).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(fs::read(&files[0]).unwrap(), CONTENT);
}

#[test]
fn a_zipcrypto_zip_needs_its_password() {
    let dir = tempdir().unwrap();
    check_passwords(&zipcrypto(dir.path()));
}

#[test]
fn an_aes_zip_needs_its_password() {
    let dir = tempdir().unwrap();
    check_passwords(&aes_zip(dir.path()));
}

#[test]
fn an_encrypted_7z_needs_its_password() {
    let dir = tempdir().unwrap();
    check_passwords(&sevenz(dir.path()));
}

#[test]
fn a_plain_zip_ignores_a_password() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("plain.zip");
    let mut zip = ZipWriter::new(File::create(&path).unwrap());
    zip.start_file("a.txt", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"a").unwrap();
    zip.finish().unwrap();
    let files = ArchiveExtractor::extract(&path, &dir.path().join("out"), Some("x")).unwrap();
    assert_eq!(files.len(), 1);
}
