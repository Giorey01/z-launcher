//! Play sessions (RF-40) and the user's flags on games (favourite, hidden, RF-17).
//!
//! A launch is noted as pending (game, package, time). When the launcher is in front again the
//! app hands over the usage events of that package, or none when it may not read them; the
//! foreground time becomes a `usage` session, or the launch-to-return time an `estimated` one
//! (capped). A game still in front, as on the other screen of a dual-screen device, stays
//! pending. Sessions under the minimum are dropped.

use serde::{Deserialize, Serialize};
use zl_config::sessions as cfg;
use zl_db::{GameFlags, NewSession, PlayStats, SessionSource};
use zl_sessions::{foreground, UsageEvent};

use crate::error::CoreError;
use crate::Service;

/// Settings key of the pending launch.
const KEY_PENDING: &str = "session.pending";

/// A game started by the launcher and not yet counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSession {
    /// The game.
    pub game_id: i64,
    /// Package that runs it (the emulator, the client, or the app itself).
    pub package: String,
    /// Launch time, Unix milliseconds.
    pub started_ms: i64,
}

/// What closing a session did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEnd {
    /// Nothing was pending.
    NoneOpen,
    /// The game is still in front; the session stays open.
    StillPlaying,
    /// Too short, or too old, to count.
    Dropped,
    /// Recorded for `game_id`, `seconds` long.
    Recorded {
        /// The game.
        game_id: i64,
        /// Seconds counted.
        seconds: i64,
    },
}

impl Service {
    /// Notes that `game_id` was started through `package` at `now_ms` (replaces any older one).
    /// An app left out of the tracking (RF-41) notes nothing: no session will be recorded.
    pub fn begin_session(&self, game_id: i64, package: &str, now_ms: i64) -> Result<(), CoreError> {
        self.remember_tracking_package(game_id, package)?;
        if self.db().is_app_excluded(package)? {
            self.db().set_setting(KEY_PENDING, "")?;
            return Ok(());
        }
        let pending = PendingSession {
            game_id,
            package: package.to_string(),
            started_ms: now_ms,
        };
        let json = serde_json::to_string(&pending).map_err(|_| CoreError::Database)?;
        self.db().set_setting(KEY_PENDING, &json)?;
        Ok(())
    }

    /// The launch waiting to be counted, if any (its package is what the app reads events of).
    pub fn pending_session(&self) -> Result<Option<PendingSession>, CoreError> {
        let text = self.db().setting(KEY_PENDING)?.filter(|t| !t.is_empty());
        Ok(text.and_then(|t| serde_json::from_str(&t).ok()))
    }

    /// Closes the pending session at `now_ms` from the usage `events` of its package, or from
    /// the launch time when `events` is `None` (the app may not read them).
    pub fn end_session(
        &self,
        events: Option<&[UsageEvent]>,
        now_ms: i64,
    ) -> Result<SessionEnd, CoreError> {
        let Some(p) = self.pending_session()? else {
            return Ok(SessionEnd::NoneOpen);
        };
        let age = (now_ms - p.started_ms) / 1000;
        let (seconds, source) = match events {
            Some(events) => {
                let f = foreground(events, p.started_ms, now_ms);
                if f.still_running && age < cfg::MAX_PENDING_SECONDS {
                    return Ok(SessionEnd::StillPlaying);
                }
                (f.ms / 1000, SessionSource::Usage)
            }
            None => (
                age.min(cfg::MAX_ESTIMATED_SECONDS),
                SessionSource::Estimated,
            ),
        };
        self.db().set_setting(KEY_PENDING, "")?;
        if seconds < cfg::MIN_SESSION_SECONDS || age > cfg::MAX_PENDING_SECONDS {
            return Ok(SessionEnd::Dropped);
        }
        self.db().add_session(&NewSession {
            game_id: p.game_id,
            started_at: p.started_ms / 1000,
            ended_at: now_ms / 1000,
            seconds,
            source,
        })?;
        Ok(SessionEnd::Recorded {
            game_id: p.game_id,
            seconds,
        })
    }

    /// Play time of one game.
    pub fn play_stats(&self, game_id: i64) -> Result<PlayStats, CoreError> {
        Ok(self.db().play_stats(game_id)?)
    }

    /// The flags of one game.
    pub fn game_flags(&self, game_id: i64) -> Result<GameFlags, CoreError> {
        Ok(self.db().game_flags(game_id)?)
    }

    /// Marks or unmarks a game as a favourite.
    pub fn set_favorite(&self, game_id: i64, favorite: bool) -> Result<(), CoreError> {
        let db = self.db();
        db.game(game_id)?.ok_or(CoreError::NotFound)?;
        let flags = GameFlags {
            favorite,
            ..db.game_flags(game_id)?
        };
        db.set_game_flags(game_id, flags)?;
        Ok(())
    }

    /// Hides a game from every list but the Hidden folder, or shows it again.
    pub fn set_hidden(&self, game_id: i64, hidden: bool) -> Result<(), CoreError> {
        let db = self.db();
        db.game(game_id)?.ok_or(CoreError::NotFound)?;
        let flags = GameFlags {
            hidden,
            ..db.game_flags(game_id)?
        };
        db.set_game_flags(game_id, flags)?;
        Ok(())
    }
}
