//! The Activity tab of a game (RF-31) and the manual changes to its sessions (RF-41): the
//! totals and the weekly chart, the list of sessions, and correcting, deleting or adding one by
//! hand. Seconds typed by the user are checked against [`cfg::MAX_MANUAL_SESSION_SECONDS`].

use zl_config::sessions as cfg;
use zl_db::{NewSession, SessionRow, SessionSource};
use zl_sessions::{summarize, ActivitySummary, SessionRecord};

use crate::error::CoreError;
use crate::Service;

/// Settings key prefix of the package a game was last started through.
const KEY_PACKAGE: &str = "session.package.";

/// What the Activity tab of a game shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameActivity {
    /// Totals and the time of each of the last weeks.
    pub summary: ActivitySummary,
    /// The latest sessions, newest first, at most [`cfg::ACTIVITY_SESSION_LIMIT`].
    pub sessions: Vec<SessionRow>,
    /// Seconds Steam reports for the account, 0 when none (not part of the sessions).
    pub steam_seconds: i64,
}

/// Whether `seconds` is a duration a person may enter.
fn valid_seconds(seconds: i64) -> bool {
    (1..=cfg::MAX_MANUAL_SESSION_SECONDS).contains(&seconds)
}

impl Service {
    /// The Activity tab of a game at `now` (Unix seconds) for a time zone `utc_offset` seconds
    /// east of UTC: the weeks of the chart open on Monday there.
    pub fn game_activity(
        &self,
        game_id: i64,
        now: i64,
        utc_offset: i64,
    ) -> Result<GameActivity, CoreError> {
        let db = self.db();
        let mut sessions = db.game_sessions(game_id)?;
        let records: Vec<SessionRecord> = sessions
            .iter()
            .map(|s| SessionRecord {
                started_at: s.started_at,
                ended_at: s.ended_at,
                seconds: s.seconds,
            })
            .collect();
        sessions.truncate(cfg::ACTIVITY_SESSION_LIMIT);
        Ok(GameActivity {
            summary: summarize(&records, now, utc_offset, cfg::ACTIVITY_WEEKS),
            sessions,
            steam_seconds: db.play_stats(game_id)?.steam_seconds,
        })
    }

    /// Corrects the duration of a session; its end moves with it.
    pub fn set_session_seconds(&self, session_id: i64, seconds: i64) -> Result<(), CoreError> {
        if !valid_seconds(seconds) {
            return Err(CoreError::InvalidInput);
        }
        match self.db().set_session_seconds(session_id, seconds)? {
            true => Ok(()),
            false => Err(CoreError::NotFound),
        }
    }

    /// Deletes a session.
    pub fn delete_session(&self, session_id: i64) -> Result<(), CoreError> {
        match self.db().delete_session(session_id)? {
            true => Ok(()),
            false => Err(CoreError::NotFound),
        }
    }

    /// Adds a session by hand: it started at `started_at` (Unix seconds) and lasted `seconds`.
    /// Its id.
    pub fn add_manual_session(
        &self,
        game_id: i64,
        started_at: i64,
        seconds: i64,
    ) -> Result<i64, CoreError> {
        if !valid_seconds(seconds) || started_at < 0 {
            return Err(CoreError::InvalidInput);
        }
        let db = self.db();
        db.game(game_id)?.ok_or(CoreError::NotFound)?;
        Ok(db.add_session(&NewSession {
            game_id,
            started_at,
            ended_at: started_at + seconds,
            seconds,
            source: SessionSource::Manual,
        })?)
    }

    /// Leaves an app out of the play time tracking, or puts it back (RF-41).
    pub fn set_app_tracking_excluded(
        &self,
        package: &str,
        excluded: bool,
    ) -> Result<(), CoreError> {
        if package.trim().is_empty() {
            return Err(CoreError::InvalidInput);
        }
        Ok(self.db().set_app_excluded(package, excluded)?)
    }

    /// Whether an app's play time is left out.
    pub fn is_app_tracking_excluded(&self, package: &str) -> Result<bool, CoreError> {
        Ok(self.db().is_app_excluded(package)?)
    }

    /// The package the game was last started through, which is what the exclusion is about;
    /// `None` when it was never started by the launcher.
    pub fn tracking_package(&self, game_id: i64) -> Result<Option<String>, CoreError> {
        let text = self.db().setting(&format!("{KEY_PACKAGE}{game_id}"))?;
        Ok(text.filter(|t| !t.is_empty()))
    }

    /// Notes the package a game was started through, for [`Service::tracking_package`].
    pub(crate) fn remember_tracking_package(
        &self,
        game_id: i64,
        package: &str,
    ) -> Result<(), CoreError> {
        Ok(self
            .db()
            .set_setting(&format!("{KEY_PACKAGE}{game_id}"), package)?)
    }
}
