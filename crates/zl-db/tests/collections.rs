//! Tests of the manual folders repository.

use zl_db::{Db, GameFile, GameKind, NewGame};

/// Result type of the tests: any error fails the test.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A database with `n` games named `g0`, `g1`, ...; returns their ids.
fn library(n: usize) -> Result<(Db, Vec<i64>), Box<dyn std::error::Error>> {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let mut ids = Vec::new();
    for i in 0..n {
        let path = format!("/roms/gba/g{i}.gba");
        let game = NewGame {
            root_id: root,
            platform_id: "gba".into(),
            title: format!("g{i}"),
            path: path.clone(),
            kind: GameKind::Rom,
            size: 1,
            mtime_ms: 1,
            fingerprint: 1,
            files: vec![GameFile {
                path,
                size: 1,
                mtime_ms: 1,
            }],
            tags: vec![],
        };
        ids.push(db.upsert_game(&game, 1)?);
    }
    Ok((db, ids))
}

/// A game can be in several folders, in the order it was added, and names are unique ignoring case.
#[test]
fn folders_hold_games_in_order_and_share_them() -> TestResult {
    let (db, ids) = library(3)?;
    let a = db.create_collection("Co-op")?;
    let b = db.create_collection("Kids")?;
    assert!(db.create_collection("co-OP").is_err(), "names are unique");
    db.add_to_collection(a, &[ids[2], ids[0]])?;
    db.add_to_collection(a, &[ids[0], ids[1]])?;
    db.add_to_collection(b, &[ids[0]])?;
    assert_eq!(db.collection_game_ids(a)?, [ids[2], ids[0], ids[1]]);
    assert_eq!(db.collections_of_game(ids[0])?, [a, b]);
    assert_eq!(db.collection_counts()?.get(&a), Some(&3));
    db.remove_from_collection(a, &[ids[0]])?;
    assert_eq!(db.collection_game_ids(a)?, [ids[2], ids[1]]);
    assert_eq!(
        db.collection_game_ids(b)?,
        [ids[0]],
        "the other folder keeps it"
    );
    Ok(())
}

/// A rewritten order is read back, and deleting a folder keeps the games.
#[test]
fn order_is_rewritten_and_delete_keeps_games() -> TestResult {
    let (db, ids) = library(3)?;
    let a = db.create_collection("A")?;
    db.add_to_collection(a, &ids)?;
    db.set_collection_order(a, &[ids[1], ids[2], ids[0]])?;
    assert_eq!(db.collection_game_ids(a)?, [ids[1], ids[2], ids[0]]);
    assert!(db.rename_collection(a, "B")?);
    assert_eq!(db.list_collections()?[0].name, "B");
    assert!(db.delete_collection(a)?);
    assert!(db.collection_game_ids(a)?.is_empty());
    assert_eq!(db.counts()?.games, 3);
    assert!(!db.delete_collection(a)?, "already gone");
    Ok(())
}

/// A game that does not exist cannot be put in a folder.
#[test]
fn unknown_game_is_refused() -> TestResult {
    let (db, _) = library(1)?;
    let a = db.create_collection("A")?;
    assert!(db.add_to_collection(a, &[9999]).is_err());
    Ok(())
}
