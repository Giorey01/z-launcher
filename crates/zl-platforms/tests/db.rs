//! Loading, validation, updates, templates and extension rules.

use std::error::Error;

use zl_platforms::{
    build_launch, DataFile, DbError, LaunchContext, LaunchError, LaunchRequest, PlatformDb,
    ProfileRef, Resolution, ResolveInput,
};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// Embedded platforms text, for building variants.
const PLATFORMS: &str = include_str!("../../../data/platforms.json");
/// Embedded emulators text, for building variants.
const EMULATORS: &str = include_str!("../../../data/emulators.json");

#[test]
fn embedded_copy_loads_and_covers_the_handheld_platforms() -> Res {
    let db = PlatformDb::embedded()?;
    let rules = db.extension_rules();
    for id in [
        "nes",
        "snes",
        "n64",
        "gb",
        "gbc",
        "gba",
        "nds",
        "n3ds",
        "gc",
        "wii",
        "switch",
        "psx",
        "ps2",
        "psp",
        "psvita",
        "genesis",
        "mastersystem",
        "gamegear",
        "segacd",
        "saturn",
        "dreamcast",
        "mame",
        "fbneo",
        "neogeo",
        "tg16",
        "scummvm",
        "dos",
        "atari2600",
        "atarilynx",
        "ngp",
        "ngpc",
        "wonderswan",
        "pico8",
        "ports",
        "steam",
        "windows",
        "moonlight",
        "xcloud",
    ] {
        assert!(rules.contains_key(id), "missing platform {id}");
    }
    assert!(rules["pico8"].contains(&"p8.png".to_string()));
    assert!(rules["psx"].contains(&"cue".to_string()));
    assert_eq!(rules["steam"], vec!["steamappid".to_string()]);
    assert_eq!(db.platform("PSX").map(|p| p.id.as_str()), Some("psx"));
    assert_eq!(db.platform("Sfc").map(|p| p.id.as_str()), Some("snes"));
    Ok(())
}

#[test]
fn higher_version_on_disk_wins_and_lower_does_not() -> Res {
    let embedded = PlatformDb::embedded()?;
    let (pv, ev) = embedded.versions();
    let dir = tempfile::tempdir()?;
    let newer = EMULATORS.replacen(
        &format!("\"version\": {ev}"),
        &format!("\"version\": {}", ev + 1),
        1,
    );
    let older = PLATFORMS.replacen(&format!("\"version\": {pv}"), "\"version\": 0", 1);
    std::fs::write(dir.path().join("emulators.json"), newer)?;
    std::fs::write(dir.path().join("platforms.json"), older)?;
    let db = PlatformDb::load_with_updates(dir.path())?;
    assert_eq!(db.versions(), (pv, ev + 1));
    Ok(())
}

#[test]
fn missing_update_folder_files_are_fine() -> Res {
    let dir = tempfile::tempdir()?;
    let db = PlatformDb::load_with_updates(dir.path())?;
    assert_eq!(db.versions(), PlatformDb::embedded()?.versions());
    Ok(())
}

#[test]
fn foreign_schema_and_broken_json_are_structured_errors() -> Res {
    let bad = EMULATORS.replacen("\"schema\": 1", "\"schema\": 2", 1);
    assert_eq!(
        PlatformDb::from_json(PLATFORMS, &bad).err(),
        Some(DbError::SchemaMismatch {
            file: DataFile::Emulators,
            found: 2,
            expected: 1
        })
    );
    let err = PlatformDb::from_json("{\n \"schema\": }", EMULATORS).err();
    assert!(matches!(
        err,
        Some(DbError::Parse {
            file: DataFile::Platforms,
            line: 2,
            ..
        })
    ));
    Ok(())
}

#[test]
fn validation_rejects_an_unknown_variable_and_a_wrong_rom_access() -> Res {
    let bad = EMULATORS.replacen("{rom_saf}", "{rom_sav}", 1);
    let err = PlatformDb::from_json(PLATFORMS, &bad).err();
    assert!(
        matches!(
            err,
            Some(DbError::Invalid {
                code: "bad_template",
                ..
            })
        ),
        "{err:?}"
    );
    let bad = EMULATORS.replacen("\"rom_access\": \"saf\"", "\"rom_access\": \"path\"", 1);
    let err = PlatformDb::from_json(PLATFORMS, &bad).err();
    assert!(
        matches!(
            err,
            Some(DbError::Invalid {
                code: "rom_access_mismatch",
                ..
            })
        ),
        "{err:?}"
    );
    let bad = PLATFORMS.replacen("\"id\": \"psx\"", "\"id\": \"snes\"", 1);
    let err = PlatformDb::from_json(&bad, EMULATORS).err();
    assert!(
        matches!(
            err,
            Some(DbError::Invalid {
                code: "bad_or_duplicate_platform_name",
                ..
            })
        ),
        "{err:?}"
    );
    Ok(())
}

#[test]
fn build_launch_rejects_unknown_variables_and_wrong_platform() -> Res {
    let db = PlatformDb::embedded()?;
    let emu = db.emulator("ppsspp").ok_or("no ppsspp")?;
    let mut profile = emu.profiles[0].clone();
    profile.intent.data = Some("{nope}".into());
    let ctx = LaunchContext {
        platform: "psp".into(),
        ..LaunchContext::default()
    };
    assert_eq!(
        build_launch(emu, &profile, &ctx).err(),
        Some(LaunchError::UnknownVariable("nope".into()))
    );
    profile.intent.data = Some("{rom".into());
    assert!(matches!(
        build_launch(emu, &profile, &ctx).err(),
        Some(LaunchError::BadTemplate(_))
    ));
    let ctx = LaunchContext {
        platform: "psx".into(),
        ..ctx
    };
    assert!(matches!(
        build_launch(emu, &emu.profiles[0], &ctx).err(),
        Some(LaunchError::ProfilePlatformMismatch { .. })
    ));
    Ok(())
}

#[test]
fn rom_name_dir_and_file_variables_derive_from_the_path() -> Res {
    let db = PlatformDb::embedded()?;
    let emu = db.emulator("ppsspp").ok_or("no ppsspp")?;
    let mut profile = emu.profiles[0].clone();
    profile.rom_access = zl_platforms::RomAccess::None;
    profile.intent.data = Some("{rom_dir}|{rom_name}|{rom_file}|{platform}".into());
    profile.intent.clip_data = None;
    profile.intent.uri_grants.clear();
    let ctx = LaunchContext {
        platform: "psp".into(),
        rom_path: "/a/b/Game.v1.iso".into(),
        ..LaunchContext::default()
    };
    let intent = build_launch(emu, &profile, &ctx)?;
    assert_eq!(intent.data.as_deref(), Some("/a/b|Game.v1|Game.v1.iso|psp"));
    Ok(())
}

#[test]
fn multi_part_extension_rule_matches_the_longest_suffix() -> Res {
    let db = PlatformDb::embedded()?;
    let installed = db
        .detect_installed(&["com.retroarch.aarch64", "io.wip.pico8"])
        .installed;
    let rule = ProfileRef::new("retroarch", "core", Some("fake08"));
    let rules = [("p8.png".to_string(), rule.clone())].into_iter().collect();
    let input = ResolveInput {
        platform: "pico8",
        file_name: "Celeste.P8.PNG",
        game_override: None,
        platform_default: None,
        extension_defaults: &rules,
        installed: &installed,
    };
    match db.resolve_profile(&input)? {
        Resolution::Resolved { profile, .. } => assert_eq!(profile, rule),
        other => return Err(format!("not resolved: {other:?}").into()),
    }
    let request = LaunchRequest::default();
    assert!(db.plan_launch(&installed, &request).is_err());
    Ok(())
}

/// Steam and Windows share the "PC" folder; aliases find it, other platforms have none.
#[test]
fn pc_folder_groups_steam_and_windows() -> Res {
    let db = PlatformDb::embedded()?;
    let folder = db.folder("PCGames").ok_or("no pc folder")?;
    assert_eq!(folder.name, "PC");
    assert_eq!(folder.platforms, ["windows", "steam"]);
    assert_eq!(
        db.folder_of("steam").map(|f| f.id.as_str()),
        Some("pcgames")
    );
    assert_eq!(
        db.folder_of("WINDOWS").map(|f| f.id.as_str()),
        Some("pcgames")
    );
    assert_eq!(db.folder_of("psx"), None);
    assert_eq!(db.folder_of("nope"), None);
    Ok(())
}

/// Builds a platforms text whose folder list is `folders` (raw JSON).
fn with_folders(folders: &str) -> Result<String, Box<dyn Error>> {
    let start = PLATFORMS.find("\"folders\"").ok_or("no folders")?;
    Ok(format!(
        "{}\"folders\": {folders}\n}}\n",
        &PLATFORMS[..start]
    ))
}

/// The code of the error a platforms text with `folders` gets.
fn folder_error(folders: &str) -> Result<Option<&'static str>, Box<dyn Error>> {
    Ok(
        match PlatformDb::from_json(&with_folders(folders)?, EMULATORS) {
            Err(DbError::Invalid { code, .. }) => Some(code),
            _ => None,
        },
    )
}

#[test]
fn validation_rejects_bad_folders() -> Res {
    let cases = [
        (
            r#"[{"id": "dos", "name": "X", "platforms": ["steam"]}]"#,
            "bad_or_duplicate_folder_id",
        ),
        (
            r#"[{"id": "pc", "name": "X", "platforms": ["steam"]}]"#,
            "bad_or_duplicate_folder_id",
        ),
        (
            r#"[{"id": "Pc2", "name": "X", "platforms": ["steam"]}]"#,
            "bad_or_duplicate_folder_id",
        ),
        (
            r#"[{"id": "a", "name": "X", "platforms": []}]"#,
            "empty_folder",
        ),
        (
            r#"[{"id": "a", "name": "X", "platforms": ["sfc"]}]"#,
            "unknown_folder_platform",
        ),
        (
            r#"[{"id": "a", "name": "X", "platforms": ["steam"]},
                {"id": "b", "name": "Y", "platforms": ["steam"]}]"#,
            "platform_in_two_folders",
        ),
    ];
    for (folders, code) in cases {
        assert_eq!(folder_error(folders)?, Some(code), "{folders}");
    }
    assert_eq!(folder_error("[]")?, None, "no folders is fine");
    Ok(())
}
