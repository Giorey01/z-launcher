//! ZIP import: accepted layouts and refused archives.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};
use zl_theme::{import_zip, import_zip_with_limits, read_package, ThemeError, ZipLimits};

/// Smallest valid `theme.json`.
const MINIMAL: &str = r#"{"name":"Zipped"}"#;

/// Writes a ZIP with the given `(name, bytes)` entries, deflated.
fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap_or_else(|e| panic!("create zip: {e}"));
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, data) in entries {
        zip.start_file(*name, opts)
            .unwrap_or_else(|e| panic!("start {name}: {e}"));
        zip.write_all(data)
            .unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    zip.finish().unwrap_or_else(|e| panic!("finish: {e}"));
}

/// Makes a temp dir and returns it with the zip path and an output path inside it.
fn sandbox() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let zip = dir.path().join("in.zip");
    let out = dir.path().join("out");
    (dir, zip, out)
}

/// Theme folder names found by an import, sorted.
fn names(roots: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = roots
        .iter()
        .filter_map(|r| r.file_name()?.to_str().map(String::from))
        .collect();
    v.sort();
    v
}

/// The exported layout `themes/<id>/theme.json` is imported and readable afterwards.
#[test]
fn imports_exported_layout() {
    let (_d, zip, out) = sandbox();
    write_zip(
        &zip,
        &[
            ("themes/a-123456/theme.json", MINIMAL.as_bytes()),
            ("themes/a-123456/wallpapers/main.png", b"png"),
        ],
    );
    let roots = import_zip(&zip, &out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&roots), ["a-123456"]);
    let pkg = read_package(&roots[0]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pkg.manifest.map(|m| m.name), Some("Zipped".to_string()));
    assert_eq!(
        pkg.wallpaper_main.and_then(|p| std::fs::read(p).ok()),
        Some(b"png".to_vec())
    );
}

/// `theme.json` at the root and inside a single top folder are accepted too.
#[test]
fn imports_root_and_single_folder_layouts() {
    let (_d, zip, out) = sandbox();
    write_zip(&zip, &[("theme.json", MINIMAL.as_bytes())]);
    let roots = import_zip(&zip, &out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(roots, vec![out.clone()]);
    let (_d2, zip2, out2) = sandbox();
    write_zip(&zip2, &[("only-top/theme.json", MINIMAL.as_bytes())]);
    let roots2 = import_zip(&zip2, &out2).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&roots2), ["only-top"]);
}

/// A ZIP with several themes yields all of them.
#[test]
fn imports_multiple_themes() {
    let (_d, zip, out) = sandbox();
    write_zip(
        &zip,
        &[
            ("themes/one-aaaaaa/theme.json", MINIMAL.as_bytes()),
            ("themes/two-bbbbbb/theme.json", MINIMAL.as_bytes()),
        ],
    );
    let roots = import_zip(&zip, &out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(names(&roots), ["one-aaaaaa", "two-bbbbbb"]);
}

/// A `..` entry is refused and nothing is written outside the destination.
#[test]
fn refuses_parent_traversal() {
    let (d, zip, out) = sandbox();
    write_zip(
        &zip,
        &[
            ("themes/a/theme.json", MINIMAL.as_bytes()),
            ("../evil", b"x"),
        ],
    );
    let err = import_zip(&zip, &out).err();
    assert!(
        matches!(err, Some(ThemeError::UnsafePath(ref n)) if n == "../evil"),
        "got {err:?}"
    );
    assert!(!d.path().join("evil").exists());
    assert!(!out.exists(), "nothing is extracted when a check fails");
}

/// An absolute entry path is refused.
#[test]
fn refuses_absolute_path() {
    let (_d, zip, out) = sandbox();
    write_zip(&zip, &[("/tmp/zl-evil-abs", b"x")]);
    assert!(matches!(
        import_zip(&zip, &out),
        Err(ThemeError::UnsafePath(_))
    ));
}

/// A symbolic link entry is refused.
#[test]
fn refuses_symlink() {
    let (_d, zip, out) = sandbox();
    let file = File::create(&zip).unwrap_or_else(|e| panic!("{e}"));
    let mut w = ZipWriter::new(file);
    w.add_symlink("themes/a/link", "/etc/passwd", SimpleFileOptions::default())
        .unwrap_or_else(|e| panic!("{e}"));
    w.finish().unwrap_or_else(|e| panic!("{e}"));
    assert!(matches!(
        import_zip(&zip, &out),
        Err(ThemeError::SymlinkEntry(_))
    ));
}

/// A highly compressible big entry (zip bomb shape) trips the ratio limit.
#[test]
fn refuses_huge_ratio() {
    let (_d, zip, out) = sandbox();
    let zeros = vec![0u8; 16 * 1024 * 1024];
    write_zip(
        &zip,
        &[
            ("themes/a/theme.json", MINIMAL.as_bytes()),
            ("themes/a/wallpapers/main.gif", &zeros),
        ],
    );
    let err = import_zip(&zip, &out).err();
    assert!(
        matches!(err, Some(ThemeError::CompressionRatio(_))),
        "got {err:?}"
    );
}

/// The entry-count, per-entry and total limits are enforced.
#[test]
fn enforces_count_and_size_limits() {
    let (_d, zip, out) = sandbox();
    let blob = vec![7u8; 1000];
    write_zip(
        &zip,
        &[
            ("a/theme.json", MINIMAL.as_bytes()),
            ("a/x", &blob),
            ("a/y", &blob),
        ],
    );
    let base = ZipLimits::default();
    let few = ZipLimits {
        max_entries: 2,
        ..base
    };
    assert!(matches!(
        import_zip_with_limits(&zip, &out, &few),
        Err(ThemeError::TooManyEntries)
    ));
    let small_entry = ZipLimits {
        max_entry_bytes: 500,
        ..base
    };
    assert!(matches!(
        import_zip_with_limits(&zip, &out, &small_entry),
        Err(ThemeError::TooLarge)
    ));
    let small_total = ZipLimits {
        max_total_bytes: 1500,
        ..base
    };
    assert!(matches!(
        import_zip_with_limits(&zip, &out, &small_total),
        Err(ThemeError::TooLarge)
    ));
    assert!(
        import_zip_with_limits(&zip, &out, &base).is_ok(),
        "same archive passes with the default limits"
    );
}

/// An archive with no `theme.json` is not a theme.
#[test]
fn refuses_archive_without_theme() {
    let (_d, zip, out) = sandbox();
    write_zip(&zip, &[("readme.txt", b"hi")]);
    assert!(matches!(
        import_zip(&zip, &out),
        Err(ThemeError::NoThemeInArchive)
    ));
}

/// A fixture theme zipped up and imported reads back with the same id and wallpapers.
#[test]
fn fixture_survives_round_trip() {
    let (_d, zip, out) = sandbox();
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/themes/cocoon/valid_video");
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    collect(&root, &root, &mut entries);
    let refs: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|(n, d)| (n.as_str(), d.as_slice()))
        .collect();
    write_zip(&zip, &refs);
    let roots = import_zip(&zip, &out).unwrap_or_else(|e| panic!("{e}"));
    let pkg = read_package(&roots[0]).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pkg.id, "neon-drift-3fa9c1");
    assert!(pkg.wallpaper_main.is_some_and(|p| p.ends_with("main.mp4")));
    assert!(pkg
        .wallpaper_external
        .is_some_and(|p| p.ends_with("external.webm")));
}

/// Collects all files under `dir` as `(path relative to root with '/', bytes)`.
fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{e}"))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if let (Ok(rel), Ok(bytes)) = (path.strip_prefix(root), std::fs::read(&path)) {
            out.push((rel.to_string_lossy().replace('\\', "/"), bytes));
        }
    }
}
