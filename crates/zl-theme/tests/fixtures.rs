//! Runs every case of `fixtures/themes/cocoon/expected.json` through the reader and validator.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use zl_theme::{
    find_themes, read_package, validate_package, ColorKey, Settings, StringEnum, ThemePackage,
};

/// Directory with the shared theme fixtures.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/themes/cocoon")
}

/// Loads the list of expected cases.
fn load_cases() -> Vec<Value> {
    let text = std::fs::read_to_string(fixtures_dir().join("expected.json")).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    doc["cases"].as_array().cloned().unwrap_or_default()
}

/// Reads the package of a case, checking the folder layout is the exported one.
fn package_of(case: &Value) -> ThemePackage {
    let name = case["case"].as_str().unwrap_or("?");
    let roots = find_themes(&fixtures_dir().join(name));
    assert_eq!(roots.len(), 1, "{name}: exactly one theme folder");
    let pkg = read_package(&roots[0]).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert_eq!(
        pkg.id,
        case["theme_id"].as_str().unwrap_or(""),
        "{name}: theme id"
    );
    pkg
}

/// File name of an optional path, as JSON.
fn file_name(path: &Option<PathBuf>) -> Value {
    path.as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map_or(Value::Null, |s| json!(s))
}

/// Value of a setting by key, as JSON (`null` when unset).
fn setting(s: &Settings, key: &str) -> Value {
    match key {
        "highlight_thickness" => json!(s.highlight_thickness),
        "glass_blur" => json!(s.glass_blur),
        "icon_scale" => json!(s.icon_scale),
        "icon_roundness" => json!(s.icon_roundness),
        "hero_vignette_size" => json!(s.hero_vignette_size),
        "glass_on_tiles" => json!(s.glass_on_tiles),
        "hero_display_style" => json!(s.hero_display_style.map(|v| v.as_str())),
        "hero_gradient_style" => json!(s.hero_gradient_style.map(|v| v.as_str())),
        "surface_material" => json!(s.surface_material.map(|v| v.as_str())),
        "dock_icon_shape" => json!(s.dock_icon_shape.map(|v| v.as_str())),
        "icon_style" => json!(s.icon_style.map(|v| v.as_str())),
        "cursor_style" => json!(s.cursor_style.map(|v| v.as_str())),
        other => panic!("test does not know setting {other}"),
    }
}

/// Checks the expected `settings` block.
fn check_settings(case: &str, pkg: &ThemePackage, expected: &Value) {
    let settings = pkg.manifest.as_ref().and_then(|m| m.settings.as_ref());
    for (key, want) in expected.as_object().into_iter().flatten() {
        let got = settings.map_or(Value::Null, |s| setting(s, key));
        assert_eq!(&got, want, "{case}: settings.{key}");
    }
}

/// Checks the expected `music` block.
fn check_music(case: &str, pkg: &ThemePackage, expected: &Value) {
    let music = &pkg
        .manifest
        .as_ref()
        .map(|m| m.music.clone())
        .unwrap_or_default();
    let schedule = music.schedule.as_ref().map(|v| {
        v.iter()
            .map(|e| json!([e.hour, e.minute, e.file]))
            .collect::<Vec<_>>()
    });
    for (key, want) in expected.as_object().into_iter().flatten() {
        let got = match key.as_str() {
            "jingle_volume" => json!(music.jingle_volume),
            "sfx_volume" => json!(music.sfx_volume),
            "music_volume" => json!(music.music_volume),
            "mode" => json!(music.mode.map(|v| v.as_str())),
            "playback_mode" => json!(music.playback_mode.map(|v| v.as_str())),
            "playlist" => json!(music.playlist),
            "schedule" => json!(schedule),
            other => panic!("test does not know music key {other}"),
        };
        assert_eq!(&got, want, "{case}: music.{key}");
    }
}

/// Checks name, mode, description, colours and extras of the manifest.
fn check_manifest(case: &Value, pkg: &ThemePackage) {
    let name = case["case"].as_str().unwrap_or("?");
    assert_eq!(pkg.manifest.is_some(), case["parsed"], "{name}: parsed");
    let Some(m) = &pkg.manifest else { return };
    assert_eq!(json!(m.name), case["name"], "{name}: name");
    assert_eq!(
        json!(m.theme_mode.map(|v| v.as_str())),
        case["theme_mode"],
        "{name}: theme_mode"
    );
    if let Some(want) = case.get("description") {
        assert_eq!(&json!(m.description), want, "{name}: description");
    }
    let colors = m.color_scheme.clone().unwrap_or_default();
    for (key, want) in case["colors"].as_object().into_iter().flatten() {
        let got = ColorKey::parse_text(key)
            .and_then(|k| colors.get(k))
            .map(|c| c.to_array());
        assert_eq!(json!(got), *want, "{name}: colour {key}");
    }
    for key in case["absent_colors"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        assert!(
            ColorKey::parse_text(key)
                .and_then(|k| colors.get(k))
                .is_none(),
            "{name}: {key} should be absent"
        );
    }
    check_extras(name, pkg, case);
}

/// Checks that unknown keys were preserved where the case says so.
fn check_extras(name: &str, pkg: &ThemePackage, case: &Value) {
    let Some(m) = &pkg.manifest else { return };
    let keys = |v: Vec<&String>| json!(v);
    if let Some(want) = case.get("unknown_colors") {
        let got = m
            .color_scheme
            .as_ref()
            .map(|c| c.unknown.keys().collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(&keys(got), want, "{name}: unknown colours");
    }
    if let Some(want) = case.get("extras") {
        assert_eq!(&keys(m.extras.keys().collect()), want, "{name}: extras");
    }
    if let Some(want) = case.get("settings_extras") {
        let got = m
            .settings
            .as_ref()
            .map(|s| s.extras.keys().collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(&keys(got), want, "{name}: settings extras");
    }
}

/// Checks the layer listing of a package.
fn check_layers(name: &str, pkg: &ThemePackage, want: &Value) {
    assert_eq!(
        json!(pkg.preview.is_some()),
        want["preview"],
        "{name}: preview"
    );
    let overlays: BTreeMap<_, _> = pkg
        .icon_overlays
        .iter()
        .map(|(k, a)| {
            (
                k.clone(),
                json!([file_name(&a.mask), file_name(&a.overlay)]),
            )
        })
        .collect();
    assert_eq!(
        json!(overlays),
        want["icon_overlays"],
        "{name}: icon_overlays"
    );
    let global = pkg
        .icon_overlay_global
        .as_ref()
        .map(|a| json!([file_name(&a.mask), file_name(&a.overlay)]));
    assert_eq!(
        global.unwrap_or(Value::Null),
        want.get("icon_overlay_global")
            .cloned()
            .unwrap_or(Value::Null),
        "{name}: icon_overlay_global"
    );
    let art = |m: &BTreeMap<String, zl_theme::FolderArt>| -> Value {
        json!(m
            .iter()
            .map(|(k, a)| (
                k.clone(),
                json!([file_name(&a.icon), file_name(&a.hero), file_name(&a.logo)])
            ))
            .collect::<BTreeMap<_, _>>())
    };
    assert_eq!(
        art(&pkg.smart_folders),
        want["smart_folders"],
        "{name}: smart_folders"
    );
    assert_eq!(
        art(&pkg.smart_folders_by_platform),
        want["smart_folders_by_platform"],
        "{name}: by platform"
    );
    let sounds: BTreeMap<_, _> = pkg
        .sounds
        .iter()
        .map(|(a, p)| (a.as_str().to_string(), file_name(&Some(p.clone()))))
        .collect();
    assert_eq!(json!(sounds), want["sounds"], "{name}: sounds");
    let music: Vec<Value> = pkg
        .music
        .iter()
        .map(|p| file_name(&Some(p.clone())))
        .collect();
    assert_eq!(Value::Array(music), want["music"], "{name}: music files");
}

/// Compares the validator output with the expected issues, ignoring order.
fn check_issues(name: &str, pkg: &ThemePackage, case: &Value) {
    let installed: Vec<String> = case["installed_packs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let mut got: Vec<String> = validate_package(pkg, &installed)
        .iter()
        .map(|i| format!("{}|{}|{}", i.severity.as_str(), i.code.as_str(), i.location))
        .collect();
    let mut want: Vec<String> = case["issues"]
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
    got.sort();
    want.sort();
    assert_eq!(got, want, "{name}: issues");
}

/// Every case of `expected.json` is read and validated as expected.
#[test]
fn every_case_matches_expected() {
    let cases = load_cases();
    assert!(
        cases.len() >= 10,
        "expected.json must list the cases, found {}",
        cases.len()
    );
    for case in &cases {
        let name = case["case"].as_str().unwrap_or("?");
        let pkg = package_of(case);
        check_manifest(case, &pkg);
        assert_eq!(
            file_name(&pkg.wallpaper_main),
            case["wallpaper_main"],
            "{name}: wallpaper_main"
        );
        assert_eq!(
            file_name(&pkg.wallpaper_external),
            case["wallpaper_external"],
            "{name}: wallpaper_external"
        );
        check_settings(name, &pkg, &case["settings"]);
        check_music(name, &pkg, &case["music"]);
        if let Some(layers) = case.get("layers") {
            check_layers(name, &pkg, layers);
        }
        if let Some(cats) = case.get("asset_ref_categories") {
            let got: Vec<&str> = pkg
                .manifest
                .iter()
                .flat_map(|m| m.asset_refs.iter().flatten())
                .map(|r| r.category.as_str())
                .collect();
            assert_eq!(json!(got), *cats, "{name}: asset ref categories");
        }
        check_issues(name, &pkg, case);
    }
}
