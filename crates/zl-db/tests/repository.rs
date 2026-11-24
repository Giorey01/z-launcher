//! Tests of migrations, WAL and the repository API.

use zl_db::{Db, DbError, GameFile, GameKind, GameQuery, NewGame};

/// Result type of the tests: any error fails the test.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Builds a game with one member file.
fn game(root_id: i64, platform: &str, title: &str, path: &str) -> NewGame {
    NewGame {
        root_id,
        platform_id: platform.to_string(),
        title: title.to_string(),
        path: path.to_string(),
        kind: GameKind::Rom,
        size: 10,
        mtime_ms: 5,
        fingerprint: 77,
        files: vec![GameFile {
            path: path.to_string(),
            size: 10,
            mtime_ms: 5,
        }],
        tags: vec![],
    }
}

/// Migrating an empty database reaches the latest version and reopening changes nothing.
#[test]
fn migration_is_idempotent_on_a_file() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("lib.db");
    let first = Db::open(&path)?;
    let version = first.schema_version()?;
    assert_eq!(version as usize, zl_db::migrations::MIGRATIONS.len());
    let root = first.upsert_root("/roms", true)?;
    first.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;
    drop(first);
    let second = Db::open(&path)?;
    assert_eq!(second.schema_version()?, version);
    assert_eq!(second.counts()?.games, 1, "data survives a reopen");
    Ok(())
}

/// A file database runs in WAL mode.
#[test]
fn file_database_uses_wal() -> TestResult {
    let dir = tempfile::tempdir()?;
    let db = Db::open(&dir.path().join("lib.db"))?;
    drop(db);
    let conn = rusqlite::Connection::open(dir.path().join("lib.db"))?;
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    assert_eq!(mode, "wal");
    Ok(())
}

/// A database from a newer app is refused instead of corrupted.
#[test]
fn newer_schema_is_refused() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("lib.db");
    drop(Db::open(&path)?);
    rusqlite::Connection::open(&path)?.pragma_update(None, "user_version", 999)?;
    match Db::open(&path) {
        Err(DbError::NewerSchema { found: 999, .. }) => Ok(()),
        other => Err(format!("expected NewerSchema, got ok={}", other.is_ok()).into()),
    }
}

/// Upsert keeps the id and `added_at`, replaces members and tags, and clears the missing flag.
#[test]
fn upsert_updates_in_place_and_clears_missing() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let mut g = game(root, "gba", "Alpha", "/roms/gba/a.gba");
    let id = db.upsert_game(&g, 1)?;
    db.mark_missing(&[id])?;
    assert_eq!(db.counts()?.missing, 1);
    assert!(
        db.list_games_by_platform("gba")?.is_empty(),
        "missing hidden by default"
    );
    g.title = "Alpha 2".into();
    g.tags = vec![("steamappid".into(), "42".into())];
    g.files.push(GameFile {
        path: "/roms/gba/a.sav".into(),
        size: 1,
        mtime_ms: 1,
    });
    assert_eq!(db.upsert_game(&g, 2)?, id);
    let stored = db.list_games_by_platform("gba")?;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].title, "Alpha 2");
    assert_eq!(stored[0].last_seen_scan, 2);
    assert_eq!(db.game_files(id)?.len(), 2);
    assert_eq!(
        db.game_tags(id)?,
        vec![("steamappid".to_string(), "42".to_string())]
    );
    Ok(())
}

/// Queries filter by platform, title text and missing flag, in case-insensitive title order.
#[test]
fn query_filters_and_orders() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    db.upsert_game(&game(root, "gba", "zelda", "/r/gba/z.gba"), 1)?;
    db.upsert_game(&game(root, "gba", "Advance Wars", "/r/gba/a.gba"), 1)?;
    db.upsert_game(&game(root, "nes", "Zelda NES", "/r/nes/z.nes"), 1)?;
    let titles = |q: &GameQuery| -> Result<Vec<String>, DbError> {
        Ok(db.query_games(q)?.into_iter().map(|g| g.title).collect())
    };
    let all = GameQuery::default();
    assert_eq!(titles(&all)?, ["Advance Wars", "zelda", "Zelda NES"]);
    let zelda = GameQuery {
        title_contains: Some("ZELDA".into()),
        ..GameQuery::default()
    };
    assert_eq!(titles(&zelda)?.len(), 2);
    let nes = GameQuery {
        platform: Some("nes".into()),
        ..GameQuery::default()
    };
    assert_eq!(titles(&nes)?, ["Zelda NES"]);
    let one = GameQuery {
        limit: Some(1),
        ..GameQuery::default()
    };
    assert_eq!(titles(&one)?.len(), 1);
    assert_eq!(db.list_platforms()?, ["gba", "nes"]);
    Ok(())
}

/// A failing transaction body leaves no trace.
#[test]
fn transaction_rolls_back_on_error() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let result: Result<(), DbError> = db.transaction(|d| {
        d.upsert_game(&game(root, "gba", "A", "/r/a.gba"), 1)?;
        Err(DbError::InvalidValue("boom".into()))
    });
    assert!(result.is_err());
    assert_eq!(db.counts()?.games, 0);
    Ok(())
}

/// Emulator overrides distinguish "no override" from "explicitly none".
#[test]
fn emulator_overrides_roundtrip() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let id = db.upsert_game(&game(root, "gba", "A", "/r/a.gba"), 1)?;
    assert_eq!(db.platform_emulator("gba")?, None);
    db.set_platform_emulator("gba", Some("mgba"))?;
    assert_eq!(db.platform_emulator("gba")?, Some(Some("mgba".to_string())));
    assert_eq!(db.game_emulator(id)?, None);
    db.set_game_emulator(id, None)?;
    assert_eq!(db.game_emulator(id)?, Some(None));
    Ok(())
}

/// A root keeps the SAF tree it was added with, and a later upsert can clear it.
#[test]
fn root_stores_its_saf_tree() -> TestResult {
    let db = Db::open_in_memory()?;
    let tree = "content://com.android.externalstorage.documents/tree/primary%3AGames";
    let id = db.upsert_root_with_saf("/storage/emulated/0/Games", true, Some(tree))?;
    assert_eq!(db.root(id)?.and_then(|r| r.saf_tree).as_deref(), Some(tree));
    db.upsert_root_with_saf("/storage/emulated/0/Games", true, None)?;
    assert_eq!(db.root(id)?.map(|r| r.saf_tree), Some(None));
    assert_eq!(db.root(id + 100)?, None);
    Ok(())
}

/// Settings read back what was written, the last write wins, and unknown keys are `None`.
#[test]
fn settings_round_trip() -> TestResult {
    let db = Db::open_in_memory()?;
    assert_eq!(db.setting("mode")?, None);
    db.set_setting("mode", "smart")?;
    db.set_setting("mode", "simple")?;
    assert_eq!(db.setting("mode")?.as_deref(), Some("simple"));
    Ok(())
}

/// Platform counts ignore missing games and one game can be fetched by id.
#[test]
fn platform_counts_skip_missing_games() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let a = db.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;
    db.upsert_game(&game(root, "gba", "B", "/roms/gba/b.gba"), 1)?;
    let n = db.upsert_game(&game(root, "nes", "N", "/roms/nes/n.nes"), 1)?;
    db.mark_missing(&[n])?;
    let counts = db.platform_counts()?;
    assert_eq!(counts.len(), 1);
    assert_eq!(
        (counts[0].platform_id.as_str(), counts[0].games),
        ("gba", 2)
    );
    assert_eq!(db.game(a)?.map(|g| g.title), Some("A".to_string()));
    assert!(db.game(9999)?.is_none());
    Ok(())
}
