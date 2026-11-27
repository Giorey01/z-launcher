//! Organisation of the library (RF-13, RF-15, RF-16, RF-26): the time-based and app Smart Folders,
//! manual folders, sort orders and the hand-made order.

use std::error::Error;
use std::fs;

use zl_config::organization::{NEW_GAME_DAYS, SECONDS_PER_DAY};
use zl_core::{CoreError, FolderKind, InstalledApp, Service, SortOrder};
use zl_library::NoProgress;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A service over a library of four GBA games, with their ids in the order Alpha, Beta, Gamma, Delta.
fn library() -> Result<(tempfile::TempDir, Service, Vec<i64>), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("gba"))?;
    for name in ["Alpha", "Beta", "Gamma", "Delta"] {
        fs::write(root.join(format!("gba/{name}.gba")), b"x")?;
    }
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    let games = svc.games(Some("gba"))?;
    let mut ids = Vec::new();
    for name in ["Alpha", "Beta", "Gamma", "Delta"] {
        ids.push(games.iter().find(|g| g.title == name).ok_or("missing")?.id);
    }
    Ok((dir, svc, ids))
}

/// Titles of a folder in order.
fn titles(svc: &Service, folder: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(svc
        .games(Some(folder))?
        .into_iter()
        .map(|g| g.title)
        .collect())
}

/// Plays `game` for `seconds`, ending at `at_ms` (a session is a launch then a return).
fn play(svc: &Service, game: i64, seconds: i64, at_ms: i64) -> Res {
    svc.begin_session(game, "com.emu", at_ms - seconds * 1000)?;
    svc.end_session(None, at_ms)?;
    Ok(())
}

/// Most played lists the games by play time and only the played ones.
#[test]
fn most_played_orders_by_play_time() -> Res {
    let (_dir, svc, ids) = library()?;
    play(&svc, ids[0], 100, 1_000_000)?;
    play(&svc, ids[2], 900, 2_000_000)?;
    play(&svc, ids[1], 300, 3_000_000)?;
    assert_eq!(titles(&svc, "@mostplayed")?, ["Gamma", "Beta", "Alpha"]);
    let entry = svc
        .platforms()?
        .into_iter()
        .find(|p| p.kind == FolderKind::MostPlayed);
    assert_eq!(entry.map(|p| p.games), Some(3));
    Ok(())
}

/// New holds the games added within the configured days, and goes away with time.
#[test]
fn new_folder_follows_the_added_time() -> Res {
    let (_dir, svc, ids) = library()?;
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    )?;
    assert_eq!(svc.games_at(Some("@new"), now)?.len(), ids.len());
    let later = now + (NEW_GAME_DAYS + 1) * SECONDS_PER_DAY;
    assert!(svc.games_at(Some("@new"), later)?.is_empty());
    assert!(
        !svc.platforms()?.iter().any(|p| p.kind == FolderKind::New),
        "New is not shown while it is the whole library"
    );
    Ok(())
}

/// Apps and Emulators come from the installed packages: an emulator is not an app, a game is
/// neither, and a folder with nothing in it is not listed.
#[test]
fn app_folders_split_the_installed_packages() -> Res {
    let (_dir, svc, _) = library()?;
    let app = |package: &str, is_game: bool| InstalledApp {
        package: package.into(),
        is_game,
    };
    let installed = [
        app("com.explusalpha.A2600Emu", false),
        app("org.example.notes", false),
        app("org.example.puzzle", true),
    ];
    assert_eq!(
        svc.app_folder_packages("@apps", &installed)?,
        ["org.example.notes"]
    );
    assert_eq!(
        svc.app_folder_packages("@emulators", &installed)?,
        ["com.explusalpha.A2600Emu"]
    );
    let kinds: Vec<_> = svc
        .app_folder_entries(&installed)?
        .into_iter()
        .map(|e| (e.kind, e.games))
        .collect();
    assert_eq!(kinds, [(FolderKind::Apps, 1), (FolderKind::Emulators, 1)]);
    assert!(svc.app_folder_entries(&[app("x.y", true)])?.is_empty());
    assert_eq!(
        svc.app_folder_packages("@favorites", &installed),
        Err(CoreError::InvalidInput)
    );
    Ok(())
}

/// A manual folder is created, filled, shown on the Home with its count, renamed and deleted;
/// the same game can be in two folders.
#[test]
fn manual_folders_share_games_and_show_on_the_home() -> Res {
    let (_dir, svc, ids) = library()?;
    let co_op = svc.create_collection("  Co-op ")?;
    let kids = svc.create_collection("Kids")?;
    assert_eq!(co_op.name, "Co-op");
    assert_eq!(
        svc.create_collection("co-op").err(),
        Some(CoreError::InvalidInput)
    );
    assert_eq!(
        svc.create_collection("   ").err(),
        Some(CoreError::InvalidInput)
    );
    svc.add_to_collection(co_op.id, &[ids[1], ids[0]])?;
    svc.add_to_collection(kids.id, &[ids[0]])?;
    assert_eq!(
        titles(&svc, &co_op.folder_id)?,
        ["Beta", "Alpha"],
        "added order"
    );
    assert_eq!(svc.game_collections(ids[0])?, [co_op.id, kids.id]);
    let home: Vec<_> = svc
        .platforms()?
        .into_iter()
        .map(|p| (p.kind, p.name, p.games))
        .collect();
    assert_eq!(
        home,
        [
            (FolderKind::Collection, "Co-op".into(), 2),
            (FolderKind::Collection, "Kids".into(), 1),
            (FolderKind::Platform, "Nintendo Game Boy Advance".into(), 4),
        ]
    );
    svc.set_hidden(ids[0], true)?;
    assert_eq!(
        titles(&svc, &co_op.folder_id)?,
        ["Beta"],
        "hidden games drop out"
    );
    svc.set_hidden(ids[0], false)?;
    svc.rename_collection(kids.id, "Family")?;
    assert_eq!(
        svc.rename_collection(kids.id, "CO-OP"),
        Err(CoreError::InvalidInput)
    );
    svc.delete_collection(co_op.id)?;
    assert_eq!(svc.delete_collection(co_op.id), Err(CoreError::NotFound));
    assert_eq!(titles(&svc, &kids.folder_id)?, ["Alpha"]);
    assert_eq!(svc.games(None)?.len(), 4, "the games stay in the library");
    Ok(())
}

/// Move and remove act on the folder only, and a game that is not in the library is refused.
#[test]
fn move_and_remove_between_folders() -> Res {
    let (_dir, svc, ids) = library()?;
    let a = svc.create_collection("A")?;
    let b = svc.create_collection("B")?;
    svc.add_to_collection(a.id, &ids)?;
    svc.move_between_collections(a.id, b.id, &[ids[1], ids[3]])?;
    assert_eq!(titles(&svc, &a.folder_id)?, ["Alpha", "Gamma"]);
    assert_eq!(titles(&svc, &b.folder_id)?, ["Beta", "Delta"]);
    svc.remove_from_collection(b.id, &[ids[1]])?;
    assert_eq!(titles(&svc, &b.folder_id)?, ["Delta"]);
    assert_eq!(svc.games(None)?.len(), 4);
    assert_eq!(
        svc.add_to_collection(a.id, &[9999]),
        Err(CoreError::NotFound)
    );
    assert_eq!(
        svc.add_to_collection(9999, &[ids[0]]),
        Err(CoreError::NotFound)
    );
    Ok(())
}

/// The hand-made order of a manual folder: a game is picked up and put down at another place.
#[test]
fn manual_order_moves_a_game() -> Res {
    let (_dir, svc, ids) = library()?;
    let f = svc.create_collection("F")?;
    svc.add_to_collection(f.id, &ids)?;
    assert_eq!(svc.sort_order(Some(&f.folder_id))?, SortOrder::Manual);
    svc.move_in_collection(f.id, ids[3], 0)?;
    assert_eq!(
        titles(&svc, &f.folder_id)?,
        ["Delta", "Alpha", "Beta", "Gamma"]
    );
    svc.move_in_collection(f.id, ids[3], 2)?;
    assert_eq!(
        titles(&svc, &f.folder_id)?,
        ["Alpha", "Beta", "Delta", "Gamma"]
    );
    svc.move_in_collection(f.id, ids[0], 99)?;
    assert_eq!(
        titles(&svc, &f.folder_id)?,
        ["Beta", "Delta", "Gamma", "Alpha"]
    );
    svc.remove_from_collection(f.id, &[ids[2]])?;
    assert_eq!(
        svc.move_in_collection(f.id, ids[2], 0),
        Err(CoreError::NotFound)
    );
    svc.set_sort_order(Some(&f.folder_id), Some(SortOrder::Name))?;
    assert_eq!(titles(&svc, &f.folder_id)?, ["Alpha", "Beta", "Delta"]);
    svc.set_sort_order(Some(&f.folder_id), Some(SortOrder::Manual))?;
    assert_eq!(
        titles(&svc, &f.folder_id)?,
        ["Beta", "Delta", "Alpha"],
        "order kept"
    );
    Ok(())
}

/// Sort orders: by folder, global for the folders that sort by title, and unrated games last.
#[test]
fn sort_orders_by_folder_and_global() -> Res {
    let (_dir, svc, ids) = library()?;
    play(&svc, ids[0], 60, 1_000_000)?;
    play(&svc, ids[1], 600, 2_000_000)?;
    svc.set_game_rating(ids[2], Some(5.0))?;
    svc.set_game_rating(ids[0], Some(3.0))?;
    assert_eq!(titles(&svc, "gba")?, ["Alpha", "Beta", "Delta", "Gamma"]);
    svc.set_sort_order(Some("gba"), Some(SortOrder::Rating))?;
    assert_eq!(titles(&svc, "gba")?, ["Gamma", "Alpha", "Beta", "Delta"]);
    svc.set_sort_order(Some("gba"), Some(SortOrder::PlayTime))?;
    assert_eq!(titles(&svc, "gba")?[..2], ["Beta", "Alpha"]);
    svc.set_sort_order(Some("gba"), Some(SortOrder::LastPlayed))?;
    assert_eq!(titles(&svc, "gba")?[..2], ["Beta", "Alpha"]);
    svc.set_sort_order(Some("gba"), None)?;
    svc.set_sort_order(None, Some(SortOrder::Rating))?;
    assert_eq!(
        svc.sort_order(Some("gba"))?,
        SortOrder::Rating,
        "global applies"
    );
    assert_eq!(
        svc.sort_order(Some("@recent"))?,
        SortOrder::LastPlayed,
        "Recent keeps its own"
    );
    assert_eq!(titles(&svc, "gba")?[..2], ["Gamma", "Alpha"]);
    svc.set_sort_order(Some("gba"), Some(SortOrder::Name))?;
    assert_eq!(
        titles(&svc, "gba")?[0],
        "Alpha",
        "the folder's own choice wins"
    );
    assert_eq!(
        svc.set_sort_order(Some("gba"), Some(SortOrder::Manual)),
        Err(CoreError::InvalidInput)
    );
    Ok(())
}
