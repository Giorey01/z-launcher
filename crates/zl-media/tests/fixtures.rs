//! Runs the scan cases of `fixtures/media/expected.json` against the readers, and the
//! `MediaDirectory` setting of ES-DE on a throwaway folder.

use std::path::{Path, PathBuf};

use serde_json::Value;
use zl_media::{esde_media_dir, scan_cocoon, scan_esde, FoundMedia};

/// The shared media fixtures folder.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/media")
}

/// Loads `expected.json`.
fn expected() -> Value {
    let text = std::fs::read_to_string(fixtures().join("expected.json")).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or(Value::Null)
}

/// A sorted list of strings from an array of the expected document.
fn strings(doc: &Value, key: &str) -> Vec<String> {
    let mut out: Vec<String> = doc[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    out.sort();
    out
}

/// Renders found media as `platform|kind|stem` plus `|edit` (or `|-`) when `with_edit` is set.
fn keys(found: &[FoundMedia], with_edit: bool) -> Vec<String> {
    let mut out: Vec<String> = found
        .iter()
        .map(|f| {
            let base = format!("{}|{}|{}", f.platform, f.kind.as_str(), f.stem);
            match (with_edit, f.edit) {
                (false, _) => base,
                (true, Some(n)) => format!("{base}|{n}"),
                (true, None) => format!("{base}|-"),
            }
        })
        .collect();
    out.sort();
    out
}

/// The ES-DE fixture folder lists exactly the expected files (case-insensitive stems, wrong
/// types and unknown folders left out, sub-folders entered).
#[test]
fn esde_scan_matches_expected() {
    let found = scan_esde(&fixtures().join("esde"));
    assert_eq!(keys(&found, false), strings(&expected(), "esde_scan"));
}

/// The Cocoon fixture folder lists exactly the expected files, edits numbered and the edit
/// suffix removed from the stem.
#[test]
fn cocoon_scan_matches_expected() {
    let found = scan_cocoon(&fixtures().join("cocoon"));
    assert_eq!(keys(&found, true), strings(&expected(), "cocoon_scan"));
}

/// A non-empty `MediaDirectory` moves the media folder; an empty one or none keeps the default.
#[test]
fn esde_media_directory_setting_is_honoured() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let root = dir.path();
    assert_eq!(esde_media_dir(root), root.join("downloaded_media"));
    std::fs::create_dir_all(root.join("settings"))?;
    let write = |value: &str| -> std::io::Result<()> {
        let xml = format!("<string name=\"MediaDirectory\" value=\"{value}\" />");
        std::fs::write(root.join("settings/es_settings.xml"), xml)
    };
    write("")?;
    assert_eq!(esde_media_dir(root), root.join("downloaded_media"));
    let elsewhere = root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("gba/covers"))?;
    std::fs::write(elsewhere.join("gba/covers/Game.png"), b"x")?;
    write(&elsewhere.to_string_lossy())?;
    assert_eq!(esde_media_dir(root), elsewhere);
    assert_eq!(scan_esde(root).len(), 1);
    Ok(())
}
