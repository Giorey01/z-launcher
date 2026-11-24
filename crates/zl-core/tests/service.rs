//! End-to-end tests of the service on a real folder tree and a real database file.

use std::error::Error;
use std::fs;
use std::path::Path;

use tempfile::TempDir;
use zl_core::{CoreError, LibraryMode, Plan, Service};
use zl_library::NoProgress;
use zl_platforms::{ProfileRef, ResolvedBy};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A service over a fresh data folder, with a ROM root laid out like a handheld's `Games` folder.
struct Fixture {
    /// Keeps the folders alive.
    _dir: TempDir,
    /// The ROM root.
    root: String,
    /// The service under test.
    svc: Service,
}

/// Writes an empty file, creating its folders.
fn touch(path: &Path) -> Res {
    fs::create_dir_all(path.parent().ok_or("no parent")?)?;
    fs::write(path, b"x")?;
    Ok(())
}

/// Builds the tree: Ps2, Psp, a Wii U folder game, an aliased folder and a Switch update.
fn fixture(saf_tree: Option<&str>) -> Result<Fixture, Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    touch(&root.join("Ps2/God of War (Europe).iso"))?;
    touch(&root.join("Ps2/systeminfo.txt"))?;
    touch(&root.join("psp/Patapon (Europe).iso"))?;
    touch(&root.join("wiiu/Mario Kart 8 [00050000]/code/MK8.rpx"))?;
    touch(&root.join("wiiu/Mario Kart 8 [00050000]/meta/meta.xml"))?;
    touch(&root.join("sfc/Zelda.sfc"))?;
    touch(&root.join("snes/Mario.sfc"))?;
    touch(&root.join("Switch/Games/Game A.nsp"))?;
    touch(&root.join("Switch/Updates_DLC/Update.nsp"))?;
    let svc = Service::open(&dir.path().join("data"))?;
    let root = root.to_string_lossy().into_owned();
    svc.add_root(&root, saf_tree)?;
    svc.scan(&NoProgress)?;
    Ok(Fixture {
        _dir: dir,
        root,
        svc,
    })
}

/// Package list of a device with the given emulator packages.
fn pkgs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// Platforms are merged by alias, counted without updates, and a Wii U folder is one game.
#[test]
fn scan_lists_platforms_and_games() -> Res {
    let f = fixture(None)?;
    let platforms = f.svc.platforms()?;
    let summary: Vec<(&str, u32)> = platforms.iter().map(|p| (p.id.as_str(), p.games)).collect();
    assert_eq!(
        summary,
        [
            ("snes", 2),
            ("switch", 1),
            ("wiiu", 1),
            ("ps2", 1),
            ("psp", 1)
        ],
        "sorted by display name"
    );
    let wiiu = f.svc.games(Some("wiiu"))?;
    assert_eq!(wiiu.len(), 1);
    assert_eq!(wiiu[0].title, "Mario Kart 8 [00050000]");
    assert_eq!(
        f.svc.games(Some("sfc"))?.len(),
        2,
        "alias asks for the platform"
    );
    assert_eq!(f.svc.games(None)?.len(), 6);
    Ok(())
}

/// A game planned with a root that has a SAF tree gets the document and tree URIs.
#[test]
fn plan_launch_hands_the_saf_document_to_the_emulator() -> Res {
    let tree = "content://com.android.externalstorage.documents/tree/Games";
    let f = fixture(Some(tree))?;
    let game = &f.svc.games(Some("psp"))?[0];
    let Plan::Launch { intent, by, .. } =
        f.svc.plan_launch(game.id, &pkgs(&["org.ppsspp.ppsspp"]))?
    else {
        return Err("expected a launch".into());
    };
    assert_eq!(by, ResolvedBy::OnlyCandidate);
    assert_eq!(intent.package, "org.ppsspp.ppsspp");
    let data = intent.data.ok_or("no data")?;
    assert!(
        data.starts_with(&format!(
            "{tree}/document/Games%2Fpsp%2FPatapon%20(Europe).iso"
        )),
        "{data}"
    );
    assert!(
        intent.uri_grants.iter().any(|g| g.uri == tree),
        "tree grant"
    );
    Ok(())
}

/// Without a SAF tree a profile that needs one reports the missing variable as a code.
#[test]
fn plan_without_a_tree_reports_a_code() -> Res {
    let f = fixture(None)?;
    let game = &f.svc.games(Some("psp"))?[0];
    let err = f
        .svc
        .plan_launch(game.id, &pkgs(&["org.ppsspp.ppsspp"]))
        .err();
    assert!(
        matches!(&err, Some(CoreError::Launch { code, detail }) if code == "missing_variable" && detail == "rom_saf"),
        "{err:?}"
    );
    Ok(())
}

/// Two installed PS2 emulators ask the user; the choice is remembered and a game override wins.
#[test]
fn choice_default_and_override_order() -> Res {
    let tree = "content://com.android.externalstorage.documents/tree/Games";
    let f = fixture(Some(tree))?;
    let both = pkgs(&["xyz.aethersx2.android", "com.armsx2"]);
    let game = f.svc.games(Some("ps2"))?[0].id;
    let Plan::NeedsChoice { candidates } = f.svc.plan_launch(game, &both)? else {
        return Err("expected a choice".into());
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().all(|c| !c.emulator_name.is_empty()));
    let first = candidates[0].profile.clone();
    let second = candidates[1].profile.clone();
    f.svc.set_platform_default("ps2", Some(&first))?;
    assert_eq!(f.svc.platform_default("ps2")?, Some(first.clone()));
    let Plan::Launch { profile, by, .. } = f.svc.plan_launch(game, &both)? else {
        return Err("expected a launch".into());
    };
    assert_eq!((profile, by), (first, ResolvedBy::PlatformDefault));
    f.svc.set_game_emulator(game, Some(&second))?;
    let Plan::Launch { profile, by, .. } = f.svc.plan_launch(game, &both)? else {
        return Err("expected a launch".into());
    };
    assert_eq!((profile, by), (second, ResolvedBy::GameOverride));
    f.svc.set_game_emulator(game, None)?;
    assert_eq!(f.svc.game_emulator(game)?, None);
    assert_eq!(
        f.svc.set_game_emulator(9999, None),
        Err(CoreError::NotFound)
    );
    Ok(())
}

/// No installed emulator means an empty choice; a Wii U game launches its `.rpx` file.
#[test]
fn no_emulator_and_wiiu_launch_file() -> Res {
    let tree = "content://com.android.externalstorage.documents/tree/Games";
    let f = fixture(Some(tree))?;
    let ps2 = f.svc.games(Some("ps2"))?[0].id;
    assert_eq!(
        f.svc.plan_launch(ps2, &[])?,
        Plan::NeedsChoice { candidates: vec![] }
    );
    let wiiu = f.svc.games(Some("wiiu"))?[0].id;
    let Plan::Launch { intent, .. } = f.svc.plan_launch(wiiu, &pkgs(&["info.cemu.cemu"]))? else {
        return Err("expected a launch".into());
    };
    let data = intent.data.ok_or("no data")?;
    assert!(data.ends_with("%2Fcode%2FMK8.rpx"), "{data}");
    Ok(())
}

/// Detection proposes defaults only for platforms in the library, and accepting stores them.
#[test]
fn detection_is_limited_to_the_library() -> Res {
    let f = fixture(None)?;
    let packages = pkgs(&[
        "org.ppsspp.ppsspp",
        "org.dolphinemu.dolphinemu",
        "info.cemu.cemu",
    ]);
    let det = f.svc.detect(&packages)?;
    let ids: Vec<&str> = det
        .proposed
        .iter()
        .map(|p| p.platform_id.as_str())
        .collect();
    assert_eq!(ids, ["wiiu", "psp"], "gc and wii are not in the library");
    assert_eq!(det.installed.len(), 3);
    assert_eq!(f.svc.accept_proposed_defaults(&packages)?, 2);
    assert_eq!(f.svc.accept_proposed_defaults(&packages)?, 0, "already set");
    assert_eq!(
        f.svc.platform_default("psp")?,
        Some(ProfileRef::new("ppsspp", "ppsspp", None))
    );
    Ok(())
}

/// Library mode, onboarding and the dock persist across a reopen; the dock has a limit.
#[test]
fn preferences_persist() -> Res {
    let f = fixture(None)?;
    assert_eq!(f.svc.library_mode()?, LibraryMode::Smart);
    assert!(!f.svc.onboarding_done()?);
    f.svc.set_library_mode(LibraryMode::Simple)?;
    f.svc.set_onboarding_done(true)?;
    for p in ["a.one", "b.two", "c.three", "d.four"] {
        f.svc.toggle_dock(p)?;
    }
    assert_eq!(f.svc.toggle_dock("e.five"), Err(CoreError::DockFull));
    assert_eq!(f.svc.toggle_dock("b.two")?, ["a.one", "c.three", "d.four"]);
    let data = f._dir.path().join("data");
    let again = Service::open(&data)?;
    assert_eq!(again.library_mode()?, LibraryMode::Simple);
    assert!(again.onboarding_done()?);
    assert_eq!(again.dock()?, ["a.one", "c.three", "d.four"]);
    assert_eq!(again.roots()?.len(), 1);
    assert_eq!(
        again.add_root("relative", None),
        Err(CoreError::InvalidInput)
    );
    let _ = &f.root;
    Ok(())
}

/// Every profile and core of the database survives the stored text form of a reference.
#[test]
fn every_profile_reference_round_trips() -> Res {
    let db = zl_platforms::PlatformDb::embedded()?;
    let mut checked = 0;
    for e in db.emulators() {
        for p in &e.profiles {
            let core = e.cores.values().flatten().next().map(|c| c.id.as_str());
            let r = ProfileRef::new(&e.id, &p.id, core);
            assert_eq!(ProfileRef::from_key(&r.to_key()), Some(r));
            checked += 1;
        }
    }
    assert!(checked > 50);
    Ok(())
}
