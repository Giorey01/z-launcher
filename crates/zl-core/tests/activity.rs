//! The Activity tab of a game (RF-31) and the manual changes to sessions (RF-41).

use std::error::Error;
use std::fs;

use zl_config::sessions as cfg;
use zl_core::{CoreError, Service, SessionEnd};
use zl_db::SessionSource;
use zl_library::NoProgress;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// Monday 2024-01-01 00:00 UTC.
const MONDAY: i64 = 1_704_067_200;

/// A service over a library of two GBA games, with their ids.
fn library() -> Result<(tempfile::TempDir, Service, Vec<i64>), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let root = dir.path().join("Games");
    fs::create_dir_all(root.join("gba"))?;
    for name in ["Alpha", "Beta"] {
        fs::write(root.join(format!("gba/{name}.gba")), b"x")?;
    }
    let svc = Service::open(&dir.path().join("data"))?;
    svc.add_root(&root.to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    let ids = svc.games(Some("gba"))?.into_iter().map(|g| g.id).collect();
    Ok((dir, svc, ids))
}

#[test]
fn the_tab_sums_sessions_by_week_and_lists_them_newest_first() -> Res {
    let (_dir, svc, ids) = library()?;
    svc.add_manual_session(ids[0], MONDAY + 3_600, 3_600)?;
    svc.add_manual_session(ids[0], MONDAY - 86_400, 1_800)?;
    svc.add_manual_session(ids[1], MONDAY, 999)?;
    let now = MONDAY + 2 * 86_400;
    let tab = svc.game_activity(ids[0], now, 0)?;
    assert_eq!(tab.summary.total_seconds, 5_400);
    assert_eq!(tab.summary.session_count, 2);
    assert_eq!(tab.summary.average_seconds, 2_700);
    assert_eq!(tab.summary.last_played, MONDAY + 7_200);
    assert_eq!(tab.summary.weeks.len(), cfg::ACTIVITY_WEEKS);
    let last_weeks: Vec<i64> = tab
        .summary
        .weeks
        .iter()
        .rev()
        .take(2)
        .map(|w| w.seconds)
        .collect();
    assert_eq!(last_weeks, [3_600, 1_800]);
    let starts: Vec<i64> = tab.sessions.iter().map(|s| s.started_at).collect();
    assert_eq!(starts, [MONDAY + 3_600, MONDAY - 86_400]);
    assert!(tab
        .sessions
        .iter()
        .all(|s| s.source == SessionSource::Manual));
    Ok(())
}

#[test]
fn a_session_can_be_corrected_and_deleted() -> Res {
    let (_dir, svc, ids) = library()?;
    let id = svc.add_manual_session(ids[0], MONDAY, 600)?;
    svc.set_session_seconds(id, 1_200)?;
    let tab = svc.game_activity(ids[0], MONDAY, 0)?;
    assert_eq!(
        (tab.sessions[0].seconds, tab.sessions[0].ended_at),
        (1_200, MONDAY + 1_200)
    );
    assert_eq!(svc.play_stats(ids[0])?.device_seconds, 1_200);
    svc.delete_session(id)?;
    assert_eq!(
        svc.game_activity(ids[0], MONDAY, 0)?.summary.session_count,
        0
    );
    assert_eq!(svc.delete_session(id), Err(CoreError::NotFound));
    assert_eq!(svc.set_session_seconds(id, 60), Err(CoreError::NotFound));
    Ok(())
}

#[test]
fn entered_durations_are_checked() -> Res {
    let (_dir, svc, ids) = library()?;
    let id = svc.add_manual_session(ids[0], MONDAY, 60)?;
    for bad in [0, -5, cfg::MAX_MANUAL_SESSION_SECONDS + 1] {
        assert_eq!(
            svc.set_session_seconds(id, bad),
            Err(CoreError::InvalidInput)
        );
        assert_eq!(
            svc.add_manual_session(ids[0], MONDAY, bad),
            Err(CoreError::InvalidInput)
        );
    }
    assert_eq!(
        svc.add_manual_session(999, MONDAY, 60),
        Err(CoreError::NotFound)
    );
    svc.set_session_seconds(id, cfg::MAX_MANUAL_SESSION_SECONDS)?;
    Ok(())
}

#[test]
fn an_excluded_app_records_nothing_until_put_back() -> Res {
    let (_dir, svc, ids) = library()?;
    assert_eq!(svc.tracking_package(ids[0])?, None);
    svc.set_app_tracking_excluded("com.emu", true)?;
    assert!(svc.is_app_tracking_excluded("com.emu")?);
    svc.begin_session(ids[0], "com.emu", 0)?;
    assert_eq!(svc.pending_session()?, None);
    assert_eq!(svc.end_session(None, 600_000)?, SessionEnd::NoneOpen);
    assert_eq!(svc.play_stats(ids[0])?.sessions, 0);
    assert_eq!(svc.tracking_package(ids[0])?.as_deref(), Some("com.emu"));
    svc.begin_session(ids[0], "com.other", 0)?;
    assert!(svc.pending_session()?.is_some(), "another app still counts");
    svc.set_app_tracking_excluded("com.emu", false)?;
    svc.begin_session(ids[0], "com.emu", 0)?;
    assert!(matches!(
        svc.end_session(None, 600_000)?,
        SessionEnd::Recorded { .. }
    ));
    assert_eq!(
        svc.set_app_tracking_excluded(" ", true),
        Err(CoreError::InvalidInput)
    );
    Ok(())
}

#[test]
fn a_launch_of_an_excluded_app_drops_the_pending_one() -> Res {
    let (_dir, svc, ids) = library()?;
    svc.begin_session(ids[0], "com.other", 0)?;
    svc.set_app_tracking_excluded("com.emu", true)?;
    svc.begin_session(ids[1], "com.emu", 1_000)?;
    assert_eq!(
        svc.pending_session()?,
        None,
        "an older launch is not counted for the new game"
    );
    Ok(())
}
