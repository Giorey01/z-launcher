//! Play sessions from usage events, the Steam play time, and the Smart Folders built on them.

use std::error::Error;
use std::fs;

use zl_core::{FolderKind, Service, SessionEnd};
use zl_library::NoProgress;
use zl_sessions::{UsageEvent, UsageKind};

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A service over a library of three GBA games, with their ids by title.
fn library() -> Result<(tempfile::TempDir, Service, Vec<i64>), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("gba"))?;
    for name in ["Alpha", "Beta", "Gamma"] {
        fs::write(root.join(format!("gba/{name}.gba")), b"x")?;
    }
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    let ids = svc.games(Some("gba"))?.into_iter().map(|g| g.id).collect();
    Ok((dir, svc, ids))
}

/// An event.
fn ev(at_ms: i64, kind: UsageKind) -> UsageEvent {
    UsageEvent { at_ms, kind }
}

#[test]
fn sessions_count_foreground_time_and_wait_for_running_games() -> Res {
    let (_dir, svc, ids) = library()?;
    let t0 = 1_700_000_000_000;
    assert_eq!(svc.end_session(Some(&[]), t0)?, SessionEnd::NoneOpen);
    svc.begin_session(ids[0], "com.emu", t0)?;
    assert_eq!(
        svc.pending_session()?.map(|p| p.package),
        Some("com.emu".into())
    );
    let running = [ev(t0 + 1_000, UsageKind::Resumed)];
    assert_eq!(
        svc.end_session(Some(&running), t0 + 600_000)?,
        SessionEnd::StillPlaying
    );
    let played = [
        ev(t0 + 1_000, UsageKind::Resumed),
        ev(t0 + 301_000, UsageKind::Paused),
        ev(t0 + 400_000, UsageKind::Resumed),
        ev(t0 + 500_000, UsageKind::Paused),
    ];
    let end = svc.end_session(Some(&played), t0 + 900_000)?;
    assert_eq!(
        end,
        SessionEnd::Recorded {
            game_id: ids[0],
            seconds: 400
        }
    );
    assert_eq!(svc.pending_session()?, None);
    let stats = svc.play_stats(ids[0])?;
    assert_eq!((stats.device_seconds, stats.sessions), (400, 1));
    svc.begin_session(ids[1], "com.emu", t0)?;
    let short = [
        ev(t0, UsageKind::Resumed),
        ev(t0 + 10_000, UsageKind::Paused),
    ];
    assert_eq!(
        svc.end_session(Some(&short), t0 + 20_000)?,
        SessionEnd::Dropped,
        "10 s is too short"
    );
    svc.begin_session(ids[1], "com.emu", t0)?;
    let estimated = svc.end_session(None, t0 + 10 * 60 * 60 * 1000)?;
    assert_eq!(
        estimated,
        SessionEnd::Recorded {
            game_id: ids[1],
            seconds: 4 * 60 * 60
        },
        "capped"
    );
    Ok(())
}

#[test]
fn smart_folders_follow_flags_and_sessions() -> Res {
    let (_dir, svc, ids) = library()?;
    let kinds = |svc: &Service| -> Result<Vec<(FolderKind, u32)>, Box<dyn Error>> {
        Ok(svc
            .platforms()?
            .into_iter()
            .map(|p| (p.kind, p.games))
            .collect())
    };
    assert_eq!(
        kinds(&svc)?,
        [(FolderKind::Platform, 3)],
        "nothing played, nothing marked"
    );
    svc.set_favorite(ids[1], true)?;
    svc.begin_session(ids[2], "com.emu", 0)?;
    svc.end_session(None, 120_000)?;
    svc.set_hidden(ids[0], true)?;
    assert_eq!(
        kinds(&svc)?,
        [
            (FolderKind::Favorites, 1),
            (FolderKind::Recent, 1),
            (FolderKind::MostPlayed, 1),
            (FolderKind::NeverPlayed, 1),
            (FolderKind::Platform, 2),
            (FolderKind::Hidden, 1),
        ]
    );
    let titles = |folder: &str| -> Result<Vec<String>, Box<dyn Error>> {
        Ok(svc
            .games(Some(folder))?
            .into_iter()
            .map(|g| g.title)
            .collect())
    };
    assert_eq!(titles("@favorites")?, ["Beta"]);
    assert_eq!(titles("@recent")?, ["Gamma"]);
    assert_eq!(
        titles("@neverplayed")?,
        ["Beta"],
        "the hidden Alpha is left out"
    );
    assert_eq!(titles("@hidden")?, ["Alpha"]);
    assert_eq!(titles("gba")?, ["Beta", "Gamma"]);
    assert!(svc.games(Some("gba"))?[0].favorite);
    svc.set_hidden(ids[0], false)?;
    assert_eq!(titles("gba")?.len(), 3);
    assert_eq!(
        svc.set_favorite(999, true),
        Err(zl_core::CoreError::NotFound)
    );
    Ok(())
}
