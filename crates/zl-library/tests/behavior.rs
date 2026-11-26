//! Behaviour of incremental scans: missing flags, restore, unavailable roots, symlink loops,
//! options and progress events.

mod common;

use std::sync::Mutex;

use common::{basic_rules, quiet_scan, root, write, TestResult};
use zl_db::{Db, GameQuery};
use zl_library::{scan, IssueCode, RootSpec, ScanEvent};

/// Unchanged files are skipped, a changed file is rewritten, and counts add up.
#[test]
fn rescan_is_incremental() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "gba/a.gba", "1")?;
    write(dir.path(), "gba/b.gba", "2")?;
    let db = Db::open_in_memory()?;
    let first = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!((first.added, first.unchanged), (2, 0));
    let second = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!((second.added, second.updated, second.unchanged), (0, 0, 2));
    write(dir.path(), "gba/a.gba", "changed size")?;
    let third = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!((third.updated, third.unchanged), (1, 1));
    let games = db.list_games_by_platform("gba")?;
    assert_eq!(
        games.iter().find(|g| g.title == "a").map(|g| g.size),
        Some(12)
    );
    Ok(())
}

/// A deleted game is flagged missing (row kept), and clears the flag when it comes back.
#[test]
fn missing_then_restored() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "gba/a.gba", "1")?;
    write(dir.path(), "gba/b.gba", "2")?;
    let db = Db::open_in_memory()?;
    quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    let id = db
        .list_games_by_platform("gba")?
        .iter()
        .find(|g| g.title == "b")
        .map(|g| g.id)
        .ok_or("b")?;
    db.set_game_emulator(id, Some("mgba"))?;
    std::fs::remove_file(dir.path().join("gba/b.gba"))?;
    let gone = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!(gone.missing, 1);
    assert_eq!(db.counts()?.games, 2, "never deleted");
    assert_eq!(db.counts()?.missing, 1);
    assert_eq!(db.list_games_by_platform("gba")?.len(), 1);
    assert_eq!(
        quiet_scan(&db, &[root(dir.path())], &basic_rules())?.missing,
        0,
        "flagged once"
    );
    write(dir.path(), "gba/b.gba", "2")?;
    let back = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!(back.restored, 1);
    assert_eq!(db.counts()?.missing, 0);
    assert_eq!(
        db.game_emulator(id)?,
        Some(Some("mgba".to_string())),
        "override survived"
    );
    Ok(())
}

/// A root that is gone (removed SD card) is reported and its games are not flagged.
#[test]
fn unavailable_root_is_left_alone() -> TestResult {
    let dir = tempfile::tempdir()?;
    let card = dir.path().join("sd");
    write(&card, "gba/a.gba", "1")?;
    let db = Db::open_in_memory()?;
    quiet_scan(&db, &[root(&card)], &basic_rules())?;
    std::fs::remove_dir_all(&card)?;
    let summary = quiet_scan(&db, &[root(&card)], &basic_rules())?;
    assert_eq!(summary.missing, 0);
    assert_eq!(
        summary.issues.first().map(|i| i.code),
        Some(IssueCode::RootUnavailable)
    );
    assert_eq!(db.counts()?.missing, 0);
    Ok(())
}

/// Several roots (internal + card) are scanned in one run.
#[test]
fn several_roots() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "internal/gba/a.gba", "1")?;
    write(dir.path(), "card/GBA/b.gba", "2")?;
    let db = Db::open_in_memory()?;
    let roots = [
        root(&dir.path().join("internal")),
        root(&dir.path().join("card")),
    ];
    let summary = quiet_scan(&db, &roots, &basic_rules())?;
    assert_eq!(summary.added, 2);
    assert_eq!(db.list_roots()?.len(), 2);
    Ok(())
}

/// With sub-folders off only the platform folder itself is read.
#[test]
fn subfolder_option() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "gba/a.gba", "1")?;
    write(dir.path(), "gba/hacks/b.gba", "2")?;
    let flat = RootSpec {
        path: dir.path().to_path_buf(),
        scan_subfolders: false,
    };
    let db = Db::open_in_memory()?;
    assert_eq!(quiet_scan(&db, &[flat], &basic_rules())?.added, 1);
    let deep = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!((deep.added, deep.unchanged), (1, 1));
    Ok(())
}

/// A symlink pointing back at an ancestor does not loop, and games are not duplicated.
#[cfg(unix)]
#[test]
fn symlink_loop_terminates() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "gba/a.gba", "1")?;
    std::os::unix::fs::symlink(dir.path().join("gba"), dir.path().join("gba/loop"))?;
    std::os::unix::fs::symlink(dir.path(), dir.path().join("gba/up"))?;
    let db = Db::open_in_memory()?;
    let summary = quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    assert_eq!(db.counts()?.games, 1);
    assert_eq!(summary.added, 1);
    Ok(())
}

/// Only enabled roots are scanned.
#[test]
fn disabled_root_is_skipped() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "gba/a.gba", "1")?;
    let db = Db::open_in_memory()?;
    let id = db.upsert_root(&dir.path().to_string_lossy(), true)?;
    db.set_root_enabled(id, false)?;
    assert_eq!(
        quiet_scan(&db, &[root(dir.path())], &basic_rules())?.added,
        0
    );
    Ok(())
}

/// Titles keep region tags; a lone disc keeps its marker; two discs merge and lose it.
#[test]
fn titles_and_disc_groups() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(dir.path(), "psx/Alone (Disc 1) (Europe).chd", "1")?;
    write(dir.path(), "psx/Pair (Europe) (Disc 1).chd", "1")?;
    write(dir.path(), "psx/Pair (Europe) (Disc 2).chd", "1")?;
    let db = Db::open_in_memory()?;
    quiet_scan(&db, &[root(dir.path())], &basic_rules())?;
    let titles: Vec<String> = db
        .query_games(&GameQuery::default())?
        .into_iter()
        .map(|g| g.title)
        .collect();
    assert_eq!(titles, ["Alone (Disc 1) (Europe)", "Pair (Europe)"]);
    Ok(())
}

/// Switch updates, DLC and a second dump of a game join the base game they name by title id;
/// an update without its base stays a game of its own.
#[test]
fn switch_addons_join_their_game() -> TestResult {
    let dir = tempfile::tempdir()?;
    write(
        dir.path(),
        "switch/Hollow Knight/Hollow Knight [0100633007D48000][v0].nsp",
        "b",
    )?;
    write(
        dir.path(),
        "switch/Hollow Knight/Hollow Knight [0100633007D48800][v589824].nsp",
        "u",
    )?;
    write(
        dir.path(),
        "switch/Updates/Boomerang Fu [DLC][010081A00EE63002].nsp",
        "d",
    )?;
    write(
        dir.path(),
        "switch/Boomerang Fu [010081A00EE62000][v0][Base].nsp",
        "b",
    )?;
    write(
        dir.path(),
        "switch/Boomerang Fu [010081A00EE62000][v0].nsz",
        "b2",
    )?;
    write(
        dir.path(),
        "switch/Only Update [0100AAAA0000A800][v65536].nsp",
        "u",
    )?;
    let db = Db::open_in_memory()?;
    let mut rules = basic_rules();
    rules.insert("switch", &["nsp", "nsz", "xci"]);
    quiet_scan(&db, &[root(dir.path())], &rules)?;
    let mut titles: Vec<String> = db
        .query_games(&GameQuery::default())?
        .into_iter()
        .map(|g| g.title)
        .collect();
    titles.sort();
    assert_eq!(titles.len(), 3, "{titles:?}");
    assert!(titles.iter().any(|t| t.starts_with("Hollow Knight")));
    assert!(titles.iter().any(|t| t.starts_with("Boomerang Fu")));
    assert!(titles.iter().any(|t| t.starts_with("Only Update")));
    Ok(())
}

/// Events arrive in order: started, platforms 1..N of N, files seen, finished.
#[test]
fn progress_events() -> TestResult {
    let dir = tempfile::tempdir()?;
    for i in 0..1200 {
        write(dir.path(), &format!("gba/g{i}.gba"), "1")?;
    }
    write(dir.path(), "psx/p.chd", "1")?;
    let events = Mutex::new(Vec::new());
    let sink = |e: ScanEvent| {
        if let Ok(mut list) = events.lock() {
            list.push(e);
        }
    };
    let db = Db::open_in_memory()?;
    scan(&db, &[root(dir.path())], &basic_rules(), &sink)?;
    let events = events.into_inner().map_err(|_| "poisoned")?;
    assert_eq!(
        events.first(),
        Some(&ScanEvent::Started {
            roots: 1,
            platforms: 2
        })
    );
    let platforms: Vec<(u32, u32)> = events
        .iter()
        .filter_map(|e| match e {
            ScanEvent::PlatformStarted { index, total, .. } => Some((*index, *total)),
            _ => None,
        })
        .collect();
    assert_eq!(platforms, [(1, 2), (2, 2)]);
    let counts: Vec<u64> = events
        .iter()
        .filter_map(|e| match e {
            ScanEvent::FilesSeen { count } => Some(*count),
            _ => None,
        })
        .collect();
    assert_eq!(counts, [500, 1000]);
    assert!(
        matches!(events.last(), Some(ScanEvent::Finished { summary }) if summary.added == 1201)
    );
    Ok(())
}
