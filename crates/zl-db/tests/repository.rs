//! Tests of migrations, WAL and the repository API.

use zl_db::{
    Db, DbError, GameFile, GameKind, GameMeta, GameQuery, MediaKind, MediaSource, MetaField,
    NewGame, NewMedia, NewSession, SessionSource,
};

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

/// A media row to store.
fn media(game_id: i64, path: &str, source: MediaSource, edit: Option<u32>) -> NewMedia {
    NewMedia {
        game_id,
        kind: MediaKind::Icon,
        path: path.to_string(),
        source,
        edit,
    }
}

/// Best file: protected, then Cocoon edit (highest number), plain Cocoon, ES-DE; a re-import
/// replaces unprotected rows only; a game's deletion takes its media along.
#[test]
fn media_precedence_and_protection() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let id = db.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;
    let items = [
        media(id, "/m/esde.png", MediaSource::Esde, None),
        media(id, "/m/cocoon.png", MediaSource::Cocoon, None),
        media(id, "/m/edit1.png", MediaSource::Cocoon, Some(1)),
        media(id, "/m/edit9.png", MediaSource::Cocoon, Some(9)),
    ];
    db.replace_media(&[MediaSource::Cocoon, MediaSource::Esde], &items)?;
    let best = |db: &Db| -> Result<String, DbError> {
        Ok(db.best_media(&[MediaKind::Icon])?.remove(0).path)
    };
    assert_eq!(best(&db)?, "/m/edit9.png");
    db.set_user_media(id, MediaKind::Icon, Some("/u/mine.png"))?;
    db.replace_media(&[MediaSource::Cocoon, MediaSource::Esde], &items[..1])?;
    assert_eq!(best(&db)?, "/u/mine.png");
    let rows = db.game_media(id)?;
    assert_eq!(
        rows.len(),
        2,
        "unprotected Cocoon rows were replaced: {rows:?}"
    );
    db.set_user_media(id, MediaKind::Icon, None)?;
    assert_eq!(best(&db)?, "/m/esde.png");
    db.delete_root(root)?;
    assert!(db.game_media(id)?.is_empty());
    Ok(())
}

/// Metadata CRUD, user overrides and protected field behavior.
#[test]
fn meta_crud_and_overrides() -> TestResult {
    let db = Db::open_in_memory()?;
    let root = db.upsert_root("/roms", true)?;
    let id = db.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;

    let meta = GameMeta {
        game_id: id,
        description: Some("Scraped synopsis".to_string()),
        release_date: Some("2001-03-21".to_string()),
        developer: Some("Original Dev".to_string()),
        publisher: Some("Original Pub".to_string()),
        genres: Some("Action, Platformer".to_string()),
        players: Some(1),
        rating: Some(4.5),
        source: "esde".to_string(),
        scraped_at: 0,
    };
    db.save_game_meta(&meta, false)?;

    let loaded = db.game_meta(id)?.ok_or("missing meta")?;
    assert_eq!(loaded.description.as_deref(), Some("Scraped synopsis"));
    assert_eq!(loaded.developer.as_deref(), Some("Original Dev"));

    db.set_meta_override(id, MetaField::Description, "User custom synopsis")?;
    db.set_meta_override(id, MetaField::Developer, "User Dev")?;

    let merged = db.game_meta(id)?.ok_or("missing merged meta")?;
    assert_eq!(merged.description.as_deref(), Some("User custom synopsis"));
    assert_eq!(merged.developer.as_deref(), Some("User Dev"));
    assert_eq!(merged.publisher.as_deref(), Some("Original Pub"));

    let raw = db.raw_game_meta(id)?.ok_or("missing raw meta")?;
    assert_eq!(raw.description.as_deref(), Some("Scraped synopsis"));

    db.clear_meta_override(id, MetaField::Developer)?;
    let cleared = db.game_meta(id)?.ok_or("missing cleared meta")?;
    assert_eq!(cleared.developer.as_deref(), Some("Original Dev"));
    Ok(())
}

/// A database created at version 3 with games and media is migrated to the latest version
/// preserving data; the rebuilt media table then takes SteamGridDB rows, ranked after ES-DE, and
/// Steam store screenshots.
#[test]
fn migration_from_v3_preserves_data() -> TestResult {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("library.db");
    {
        let conn = rusqlite::Connection::open(&db_path)?;
        for sql in &zl_db::migrations::MIGRATIONS[..3] {
            conn.execute_batch(sql)?;
        }
        conn.execute_batch("PRAGMA user_version = 3")?;
        assert_eq!(zl_db::migrations::current_version(&conn)?, 3);

        conn.execute(
            "INSERT INTO rom_roots (path, enabled, scan_subfolders) VALUES ('/roms', 1, 1)",
            [],
        )?;
        conn.execute("INSERT INTO platforms (id) VALUES ('gba')", [])?;
        conn.execute(
            "INSERT INTO games (root_id, platform_id, title, path, kind, size, mtime, fingerprint, missing, added_at, last_seen_scan)
             VALUES (1, 'gba', 'Pokemon', '/roms/gba/p.gba', 'rom', 100, 10, 55, 0, 1234, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO game_media (game_id, kind, path, source, edit, protected)
             VALUES (1, 'icon', '/m/p.png', 'esde', NULL, 0)",
            [],
        )?;
    }

    let db = Db::open(&db_path)?;
    assert_eq!(
        db.schema_version()?,
        zl_db::migrations::MIGRATIONS.len() as u32
    );

    let g = db.game(1)?.ok_or("game missing after migration")?;
    assert_eq!(g.title, "Pokemon");
    assert_eq!(g.path, "/roms/gba/p.gba");

    let media = db.game_media(1)?;
    assert_eq!(media.len(), 1);
    assert_eq!(media[0].path, "/m/p.png");

    let meta = db.game_meta(1)?;
    assert!(meta.is_none());

    db.add_media(&zl_db::NewMedia {
        game_id: 1,
        kind: zl_db::MediaKind::Icon,
        path: "/data/sgdb/p.png".into(),
        source: zl_db::MediaSource::SteamGridDb,
        edit: None,
    })?;
    let media = db.game_media(1)?;
    assert_eq!(media.len(), 2);
    assert_eq!(media[0].path, "/m/p.png", "ES-DE art stays first");
    assert_eq!(media[1].source, zl_db::MediaSource::SteamGridDb);
    db.add_media(&zl_db::NewMedia {
        game_id: 1,
        kind: zl_db::MediaKind::Screenshot,
        path: "/data/steam/400/0.jpg".into(),
        source: zl_db::MediaSource::Steam,
        edit: None,
    })?;
    let shots: Vec<_> = db
        .game_media(1)?
        .into_iter()
        .filter(|m| m.kind == zl_db::MediaKind::Screenshot)
        .collect();
    assert_eq!(shots.len(), 1, "step 6 takes store screenshots");
    assert_eq!(shots[0].source, zl_db::MediaSource::Steam);
    Ok(())
}

/// Flags default to off and round-trip; sessions sum by source and the Steam total is replaced
/// on every import.
#[test]
fn flags_and_play_sessions() -> TestResult {
    use zl_db::GameFlags;
    let dir = tempfile::tempdir()?;
    let db = Db::open(&dir.path().join("lib.db"))?;
    let root = db.upsert_root("/roms", true)?;
    let a = db.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;
    let b = db.upsert_game(&game(root, "gba", "B", "/roms/gba/b.gba"), 1)?;
    assert_eq!(db.game_flags(a)?, GameFlags::default());
    let fav = GameFlags {
        favorite: true,
        hidden: false,
    };
    db.set_game_flags(a, fav)?;
    assert_eq!(db.game_flags(a)?, fav);
    assert_eq!(db.all_game_flags()?.len(), 1);
    for (start, end, secs, source) in [
        (100, 400, 250, SessionSource::Usage),
        (1000, 1600, 600, SessionSource::Estimated),
    ] {
        db.add_session(&NewSession {
            game_id: a,
            started_at: start,
            ended_at: end,
            seconds: secs,
            source,
        })?;
    }
    db.set_steam_playtime(a, 3600, 50)?;
    db.set_steam_playtime(a, 7200, 60)?;
    let stats = db.play_stats(a)?;
    assert_eq!(
        (
            stats.device_seconds,
            stats.steam_seconds,
            stats.sessions,
            stats.last_played
        ),
        (850, 7200, 2, 1600)
    );
    db.set_steam_playtime(a, 0, 0)?;
    assert_eq!(
        db.play_stats(a)?.steam_seconds,
        0,
        "zero removes the Steam total"
    );
    assert_eq!(db.play_stats(b)?, zl_db::PlayStats::default());
    Ok(())
}

/// Steam's total is not a session: the list leaves it out and the edits refuse to touch it.
#[test]
fn session_edits_leave_the_steam_total_alone() -> TestResult {
    let dir = tempfile::tempdir()?;
    let db = Db::open(&dir.path().join("lib.db"))?;
    let root = db.upsert_root("/roms", true)?;
    let a = db.upsert_game(&game(root, "gba", "A", "/roms/gba/a.gba"), 1)?;
    let manual = db.add_session(&NewSession {
        game_id: a,
        started_at: 1000,
        ended_at: 1600,
        seconds: 600,
        source: SessionSource::Manual,
    })?;
    db.set_steam_playtime(a, 7200, 60)?;
    let listed = db.game_sessions(a)?;
    assert_eq!(listed.len(), 1);
    assert_eq!(
        (listed[0].id, listed[0].source),
        (manual, SessionSource::Manual)
    );
    let steam_id: i64 = rusqlite::Connection::open(dir.path().join("lib.db"))?.query_row(
        "SELECT id FROM game_sessions WHERE source = 'steam'",
        [],
        |r| r.get(0),
    )?;
    assert!(!db.set_session_seconds(steam_id, 5)?);
    assert!(!db.delete_session(steam_id)?);
    assert_eq!(db.play_stats(a)?.steam_seconds, 7200);
    assert!(db.set_session_seconds(manual, 900)?);
    assert_eq!(db.game_sessions(a)?[0].ended_at, 1900);
    assert!(db.delete_session(manual)?);
    assert!(db.game_sessions(a)?.is_empty());
    Ok(())
}

/// The exclusion of an app is stored, undone and repeated without error.
#[test]
fn excluded_apps_are_stored() -> TestResult {
    let dir = tempfile::tempdir()?;
    let db = Db::open(&dir.path().join("lib.db"))?;
    assert!(!db.is_app_excluded("com.emu")?);
    db.set_app_excluded("com.emu", true)?;
    db.set_app_excluded("com.emu", true)?;
    assert!(db.is_app_excluded("com.emu")?);
    assert!(!db.is_app_excluded("com.other")?);
    db.set_app_excluded("com.emu", false)?;
    assert!(!db.is_app_excluded("com.emu")?);
    Ok(())
}

/// A database at the step before sessions could be entered by hand keeps its sessions when the
/// table is rebuilt, and then takes `manual` ones.
#[test]
fn migration_to_manual_sessions_preserves_sessions() -> TestResult {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("library.db");
    {
        let conn = rusqlite::Connection::open(&db_path)?;
        for sql in &zl_db::migrations::MIGRATIONS[..8] {
            conn.execute_batch(sql)?;
        }
        conn.execute_batch("PRAGMA user_version = 8")?;
        conn.execute(
            "INSERT INTO rom_roots (path, enabled, scan_subfolders) VALUES ('/roms', 1, 1)",
            [],
        )?;
        conn.execute("INSERT INTO platforms (id) VALUES ('gba')", [])?;
        conn.execute(
            "INSERT INTO games (root_id, platform_id, title, path, kind, size, mtime, fingerprint, missing, added_at, last_seen_scan)
             VALUES (1, 'gba', 'A', '/roms/gba/a.gba', 'rom', 1, 1, 1, 0, 1, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, seconds, source)
             VALUES (1, 10, 70, 60, 'usage')",
            [],
        )?;
    }
    let db = Db::open(&db_path)?;
    assert_eq!(
        db.schema_version()?,
        zl_db::migrations::MIGRATIONS.len() as u32
    );
    let kept = db.game_sessions(1)?;
    assert_eq!(
        (kept.len(), kept[0].seconds, kept[0].source),
        (1, 60, SessionSource::Usage)
    );
    db.add_session(&NewSession {
        game_id: 1,
        started_at: 100,
        ended_at: 160,
        seconds: 60,
        source: SessionSource::Manual,
    })?;
    assert_eq!(db.play_stats(1)?.sessions, 2);
    Ok(())
}
