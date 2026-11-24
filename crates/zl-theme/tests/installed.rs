//! Runs the `data_dirs` cases of `fixtures/themes/cocoon/expected.json`: a data folder with
//! installed themes and asset packs in both layouts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use zl_theme::{
    list_installed, read_pack_content, read_package, resolve_music_file, validate_pack,
    validate_package, AssetPack, FolderArt, Issue, MusicConfig, OverlayArt,
};

/// Directory with the shared theme fixtures.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/themes/cocoon")
}

/// Loads the `data_dirs` cases.
fn load_cases() -> Vec<Value> {
    let text = std::fs::read_to_string(fixtures_dir().join("expected.json")).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    doc["data_dirs"].as_array().cloned().unwrap_or_default()
}

/// File name of an optional path, as JSON.
fn name_of(path: Option<&PathBuf>) -> Value {
    path.and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or(Value::Null, |s| json!(s))
}

/// Issues as `severity|code|location` strings, sorted.
fn issue_keys(issues: &[Issue]) -> Vec<String> {
    let mut keys: Vec<String> = issues
        .iter()
        .map(|i| format!("{}|{}|{}", i.severity.as_str(), i.code.as_str(), i.location))
        .collect();
    keys.sort();
    keys
}

/// Expected issues as the same sorted strings.
fn expected_keys(list: &Value) -> Vec<String> {
    let mut keys: Vec<String> = list
        .as_array()
        .into_iter()
        .flatten()
        .map(|i| {
            format!(
                "{}|{}|{}",
                i["severity"].as_str().unwrap_or(""),
                i["code"].as_str().unwrap_or(""),
                i["location"].as_str().unwrap_or("")
            )
        })
        .collect();
    keys.sort();
    keys
}

/// `[mask, overlay]` file names of an overlay art.
fn overlay_json(a: &OverlayArt) -> Value {
    json!([name_of(a.mask.as_ref()), name_of(a.overlay.as_ref())])
}

/// `[icon, hero, logo]` file names of a folder art.
fn folder_json(a: &FolderArt) -> Value {
    json!([
        name_of(a.icon.as_ref()),
        name_of(a.hero.as_ref()),
        name_of(a.logo.as_ref())
    ])
}

/// Checks the expected `settings` of a pack's metadata.
fn check_settings(id: &str, music: &MusicConfig, want: &Value) {
    let schedule = music.schedule.as_ref().map(|v| {
        v.iter()
            .map(|e| json!([e.hour, e.minute, e.file]))
            .collect::<Vec<_>>()
    });
    let got = json!({
        "mode": music.mode.map(|v| v.as_str()),
        "playback_mode": music.playback_mode.map(|v| v.as_str()),
        "playlist": music.playlist,
        "schedule": schedule,
    });
    for (key, value) in want.as_object().into_iter().flatten() {
        assert_eq!(&got[key], value, "{id}: settings.{key}");
    }
}

/// Checks the metadata block of one pack.
fn check_meta(pack: &AssetPack, want: &Value) {
    let id = &pack.id;
    let meta = pack
        .meta
        .as_ref()
        .unwrap_or_else(|| panic!("{id}: metadata.json not read"));
    assert_eq!(json!(meta.name), want["name"], "{id}: name");
    for (key, got) in [
        ("author", json!(meta.author)),
        ("version", json!(meta.version)),
        ("file_count", json!(meta.file_count)),
    ] {
        if let Some(w) = want.get(key) {
            assert_eq!(&got, w, "{id}: {key}");
        }
    }
    match (&meta.settings, want.get("settings")) {
        (None, Some(Value::Null)) => {}
        (Some(m), Some(w)) if !w.is_null() => check_settings(id, m, w),
        (got, want) => panic!("{id}: settings mismatch: {got:?} vs {want:?}"),
    }
}

/// Checks the layers a pack provides against the expected block.
fn check_layers(pack: &AssetPack, want: &Value) {
    let id = &pack.id;
    let set = read_pack_content(pack);
    let overlays: BTreeMap<_, _> = set
        .icon_overlays
        .iter()
        .map(|(k, a)| (k, overlay_json(a)))
        .collect();
    let folders: BTreeMap<_, _> = set
        .smart_folders
        .iter()
        .map(|(k, a)| (k, folder_json(a)))
        .collect();
    let by_platform: BTreeMap<_, _> = set
        .smart_folders_by_platform
        .iter()
        .map(|(k, a)| (k, folder_json(a)))
        .collect();
    let sounds: BTreeMap<_, _> = set
        .sounds
        .iter()
        .map(|(a, p)| (a.as_str(), name_of(Some(p))))
        .collect();
    let music: Vec<Value> = set.music.iter().map(|p| name_of(Some(p))).collect();
    let global = set
        .icon_overlay_global
        .as_ref()
        .map_or(Value::Null, overlay_json);
    let got = json!({
        "icon_overlays": overlays, "icon_overlay_global": global, "smart_folders": folders,
        "smart_folders_by_platform": by_platform, "sounds": sounds,
        "unknown_sounds": set.unknown_sounds, "music": music,
    });
    for key in got.as_object().into_iter().flat_map(|o| o.keys()) {
        let expected = want.get(key).cloned().unwrap_or_else(|| default_of(key));
        assert_eq!(got[key], expected, "{id}: {key}");
    }
}

/// What a layer is expected to hold when the case does not mention it: nothing.
fn default_of(key: &str) -> Value {
    match key {
        "icon_overlay_global" => Value::Null,
        "icon_overlays" | "smart_folders" | "smart_folders_by_platform" | "sounds" => json!({}),
        _ => json!([]),
    }
}

/// Checks one pack in full.
fn check_pack(case: &str, pack: &AssetPack, want: &Value) {
    let id = &pack.id;
    assert_eq!(pack.category.as_str(), want["category"], "{id}: category");
    let pack_dir = fixtures_dir()
        .join(case)
        .join("assets")
        .join(pack.category.as_str())
        .join(id);
    let expected_root = match want["content_root"].as_str() {
        Some(".") | None => pack_dir,
        Some(sub) => pack_dir.join(sub),
    };
    assert_eq!(pack.root_of_content, expected_root, "{id}: content root");
    check_meta(pack, &want["meta"]);
    check_layers(pack, want);
    for (reference, file) in want["resolved_music"].as_object().into_iter().flatten() {
        let got = name_of(resolve_music_file(pack, reference).as_ref());
        assert_eq!(&got, file, "{id}: resolve {reference}");
    }
    assert_eq!(
        issue_keys(&validate_pack(pack)),
        expected_keys(&want["issues"]),
        "{id}: issues"
    );
}

/// Installed themes and packs of every data folder are listed and read as expected.
#[test]
fn data_dirs_match_expected() {
    let cases = load_cases();
    assert!(!cases.is_empty(), "expected.json must list data_dirs");
    for case in &cases {
        let name = case["case"].as_str().unwrap_or("?");
        let installed = list_installed(&fixtures_dir().join(name));
        let themes: Vec<_> = installed
            .themes
            .iter()
            .filter_map(|p| p.file_name()?.to_str().map(String::from))
            .collect();
        assert_eq!(json!(themes), case["themes"], "{name}: themes");
        let mut ids = installed.pack_ids();
        let mut want_ids: Vec<String> =
            serde_json::from_value(case["pack_ids"].clone()).unwrap_or_default();
        ids.sort();
        want_ids.sort();
        assert_eq!(ids, want_ids, "{name}: pack ids");
        for pack in &installed.packs {
            let want = &case["packs"][&pack.id];
            assert!(want.is_object(), "{name}: pack {} not expected", pack.id);
            check_pack(name, pack, want);
        }
        let theme = read_package(&installed.themes[0]).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            issue_keys(&validate_package(&theme, &installed.pack_ids())),
            expected_keys(&case["theme_issues"]),
            "{name}: theme issues"
        );
    }
}
