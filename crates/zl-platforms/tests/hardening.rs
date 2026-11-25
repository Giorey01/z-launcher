//! Regression tests of edge cases: SAF URIs, unresolved placeholders, empty values and the
//! consistency rules of the validator.

use std::error::Error;

use zl_platforms::saf::{saf_document_uri, tree_uri_for_path, uri_decode};
use zl_platforms::{build_launch, DbError, LaunchContext, LaunchError, PlatformDb, RomAccess};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// Embedded platforms text, for building variants.
const PLATFORMS: &str = include_str!("../../../data/platforms.json");
/// Embedded emulators text, for building variants.
const EMULATORS: &str = include_str!("../../../data/emulators.json");
/// Tree granted on `/storage/emulated/0/Games`.
const TREE: &str = "content://com.android.externalstorage.documents/tree/primary%3AGames";
/// Folder of [`TREE`].
const ROOT: &str = "/storage/emulated/0/Games";

/// The code of a validation failure, or `None` when the database loaded.
fn code_of(platforms: &str, emulators: &str) -> Option<&'static str> {
    match PlatformDb::from_json(platforms, emulators) {
        Err(DbError::Invalid { code, .. }) => Some(code),
        _ => None,
    }
}

/// Builds the intent of PPSSPP's first profile with `data` as its only template.
fn render(data: &str, ctx: &LaunchContext) -> Result<Option<String>, Box<dyn Error>> {
    let db = PlatformDb::embedded()?;
    let emu = db.emulator("ppsspp").ok_or("no ppsspp")?;
    let mut profile = emu.profiles[0].clone();
    profile.rom_access = RomAccess::None;
    profile.intent.data = Some(data.into());
    profile.intent.clip_data = None;
    profile.intent.uri_grants.clear();
    Ok(build_launch(emu, &profile, ctx)?.data)
}

/// A context for a PSP game at `rom_path`.
fn psp(rom_path: &str) -> LaunchContext {
    LaunchContext {
        platform: "psp".into(),
        rom_path: rom_path.into(),
        ..LaunchContext::default()
    }
}

#[test]
fn percent_escapes_need_two_real_hex_digits() {
    // `u8::from_str_radix` alone would accept `%+1` as 0x01.
    assert_eq!(uri_decode("a%+1b"), None);
    assert_eq!(uri_decode("a%-1b"), None);
    assert_eq!(uri_decode("a%2fb"), Some("a/b".into()));
}

#[test]
fn saf_paths_with_dot_segments_or_double_slashes() {
    let doc = |file: &str| saf_document_uri(TREE, ROOT, file);
    assert_eq!(doc("/storage/emulated/0/Games/../Secret/a.iso"), None);
    assert_eq!(doc("/storage/emulated/0/Games/./a.iso"), None);
    assert_eq!(
        doc("/storage/emulated/0/Games//psp//a.iso"),
        doc("/storage/emulated/0/Games/psp/a.iso")
    );
    assert!(doc("/storage/emulated/0/Games/psp/a.iso").is_some());
}

#[test]
fn tree_without_a_volume_falls_back_to_the_saved_root() {
    let tree = "content://com.android.externalstorage.documents/tree/%3AGames";
    // No volume name: the base is the saved root, not `/storage/` + the id's path.
    let uri = saf_document_uri(tree, "/mnt/Games", "/mnt/Games/a.iso");
    assert_eq!(
        uri.as_deref(),
        Some(
            "content://com.android.externalstorage.documents/tree/%3AGames/document/\
             %3AGames%2Fa.iso"
        )
    );
    assert_eq!(
        saf_document_uri(tree, "/mnt/Games", "/storage/Games/a.iso"),
        None
    );
}

#[test]
fn secondary_user_storage_is_the_primary_volume() {
    let uri = |p: &str| tree_uri_for_path(p);
    assert_eq!(
        uri("/storage/emulated/10/Games"),
        uri("/storage/emulated/0/Games")
    );
    assert_eq!(uri("/storage/emulated/10"), uri("/storage/emulated/0"));
    // Not a user id: not storage we know.
    assert_ne!(
        uri("/storage/emulated/x/Games"),
        uri("/storage/emulated/0/Games")
    );
}

#[test]
fn an_empty_rom_is_missing_not_an_empty_string() {
    for template in ["{rom}", "{rom_file}", "{rom_name}"] {
        let err = render(template, &psp("")).err().map(|e| e.to_string());
        assert!(
            err.as_deref()
                .is_some_and(|e| e.starts_with("missing_variable")),
            "{template}: {err:?}"
        );
    }
}

#[test]
fn dotfiles_keep_their_name_and_root_files_have_a_root_dir() -> Res {
    let out = render("{rom_name}|{rom_dir}", &psp("/.hidden"))?;
    assert_eq!(out.as_deref(), Some(".hidden|/"));
    let out = render("{rom_name}|{rom_dir}|{rom_file}", &psp("Game.iso"))?;
    assert_eq!(out.as_deref(), Some("Game||Game.iso"));
    Ok(())
}

#[test]
fn a_stray_closing_brace_is_a_bad_template() {
    for text in ["{rom}}", "rom}", "x}{rom}", "}"] {
        let err = render(text, &psp("/a.iso")).err().map(|e| e.to_string());
        assert!(
            err.as_deref()
                .is_some_and(|e| e.starts_with("bad_template")),
            "{text}: {err:?}"
        );
    }
    let ok = render("a{rom}b", &psp("/a.iso"));
    assert_eq!(ok.ok().flatten().as_deref(), Some("a/a.isob"));
}

#[test]
fn an_empty_tag_or_uri_is_missing() {
    let mut ctx = psp("/a.iso");
    ctx.tags.insert("x".into(), " ".into());
    assert_eq!(
        render("{tags.x}", &ctx).err().map(|e| e.to_string()),
        Some(LaunchError::MissingTag("x".into()).to_string())
    );
    ctx.rom_uris.file = Some(String::new());
    assert_eq!(
        render("{rom_uri}", &ctx).err().map(|e| e.to_string()),
        Some(LaunchError::MissingVariable("rom_uri".into()).to_string())
    );
}

#[test]
fn ids_that_would_break_a_stored_profile_key_are_rejected() -> Res {
    let mutate = |f: &dyn Fn(&mut serde_json::Value)| -> Result<String, Box<dyn Error>> {
        let mut value: serde_json::Value = serde_json::from_str(EMULATORS)?;
        f(&mut value["emulators"][0]);
        Ok(serde_json::to_string(&value)?)
    };
    let slash_profile = mutate(&|e| e["profiles"][0]["id"] = "a/b".into())?;
    let space_emulator = mutate(&|e| e["id"] = "a b".into())?;
    let empty_profile = mutate(&|e| e["profiles"][0]["id"] = "".into())?;
    for bad in [&slash_profile, &space_emulator, &empty_profile] {
        assert_eq!(code_of(PLATFORMS, bad), Some("bad_id"));
    }
    assert_eq!(code_of(PLATFORMS, &mutate(&|_| {})?), None);
    Ok(())
}

#[test]
fn preferences_must_name_a_platform_that_exists() {
    let bad = EMULATORS.replacen(
        "\"preferences\": {",
        "\"preferences\": {\n    \"nonexistent\": [],",
        1,
    );
    assert_ne!(bad, EMULATORS, "layout of emulators.json changed");
    assert_eq!(
        code_of(PLATFORMS, &bad),
        Some("unknown_preference_platform")
    );
}

#[test]
fn a_core_profile_needs_a_core_for_each_of_its_platforms() -> Res {
    let mut value: serde_json::Value = serde_json::from_str(EMULATORS)?;
    let emulators = value["emulators"].as_array_mut().ok_or("no emulators")?;
    let ra = emulators
        .iter_mut()
        .find(|e| e["id"] == "retroarch")
        .ok_or("no retroarch")?;
    ra["cores"]["psx"] = serde_json::json!([]);
    let bad = serde_json::to_string(&value)?;
    assert_eq!(code_of(PLATFORMS, &bad), Some("no_core_for_platform"));
    Ok(())
}
