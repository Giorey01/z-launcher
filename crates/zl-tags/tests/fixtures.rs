//! Runs every case of `fixtures/tags/expected.json` through the reader.

use std::path::{Path, PathBuf};

use serde_json::Value;
use zl_tags::{parse_tag_file, read_tag_file, TagKind};

/// Directory with the shared tag fixtures.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tags")
}

/// Name of a [`TagKind`] as spelled in `expected.json`.
fn kind_name(kind: TagKind) -> &'static str {
    match kind {
        TagKind::SteamAppId => "SteamAppId",
        TagKind::LocalGameId => "LocalGameId",
        TagKind::ScummvmId => "ScummvmId",
        TagKind::VitaGameId => "VitaGameId",
        TagKind::MoonlightId => "MoonlightId",
        TagKind::XcloudTitleId => "XcloudTitleId",
        TagKind::Other => "Other",
    }
}

/// Loads the expected file.
fn load_expected() -> Vec<Value> {
    let text = std::fs::read_to_string(fixtures_dir().join("expected.json")).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    doc["cases"].as_array().cloned().unwrap_or_default()
}

/// Every expected case matches what the reader returns, for name, tags, kind and issues.
#[test]
fn every_case_matches_expected() {
    let cases = load_expected();
    assert!(
        cases.len() >= 15,
        "expected.json must list the cases, found {}",
        cases.len()
    );
    for case in &cases {
        let rel = case["path"].as_str().unwrap_or("");
        let path = fixtures_dir().join("files").join(rel);
        let parsed = read_tag_file(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert_eq!(parsed.file.name, case["name"], "{rel}: name");
        assert_eq!(parsed.file.extension, case["extension"], "{rel}: extension");
        let tags: Vec<Value> = parsed
            .file
            .tags
            .iter()
            .map(|(k, v)| serde_json::json!([k, v]))
            .collect();
        assert_eq!(Value::Array(tags), case["tags"], "{rel}: tags");
        let issues: Vec<Value> = parsed
            .issues
            .iter()
            .map(|i| serde_json::json!({"line": i.line, "code": i.code.as_str()}))
            .collect();
        assert_eq!(Value::Array(issues), case["issues"], "{rel}: issues");
        let platform = case["platform"].as_str();
        assert_eq!(
            kind_name(parsed.file.kind(platform)),
            case["kind"],
            "{rel}: kind"
        );
        if let Some(lookup) = case["lookup"].as_object() {
            for (key, value) in lookup {
                assert_eq!(parsed.file.get(key), value.as_str(), "{rel}: lookup {key}");
            }
        }
    }
}

/// The shared `.dpt` extension is resolved by the platform folder when both keys exist.
#[test]
fn dpt_tie_is_broken_by_platform() {
    let text = "[vita_game_id] a\n[scummvm_id] b\n";
    let file = parse_tag_file("x.dpt", text).file;
    assert_eq!(file.kind(Some("PSVITA")), TagKind::VitaGameId);
    assert_eq!(file.kind(Some("scummvm")), TagKind::ScummvmId);
    assert_eq!(file.kind(None), TagKind::ScummvmId);
}

/// A file name with several dots keeps all but the last part as the name.
#[test]
fn name_keeps_inner_dots() {
    let parsed = parse_tag_file("Mr. Game v1.2.steamappid", "[steamappid] 1");
    assert_eq!(parsed.file.name, "Mr. Game v1.2");
    assert_eq!(parsed.file.extension, "steamappid");
}
