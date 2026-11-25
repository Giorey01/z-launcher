//! The play half of the [`Core`] object: sessions from the system's usage events, play time,
//! and the favourite and hidden flags.

use zl_core as core;
use zl_db as db;
use zl_sessions as sessions;

use crate::core_object::Core;
use crate::error::CoreError;

/// Result of a core call.
type Res<T> = Result<T, CoreError>;

mirror_enum! {
    /// What happened to an activity of the game.
    UsageKind <=> sessions::UsageKind { Resumed, Paused }
}
mirror_record! {
    /// One usage event of the game's package (from `UsageStatsManager`).
    UsageEvent <=> sessions::UsageEvent {
        /// When, Unix milliseconds.
        plain at_ms: i64,
        /// What.
        plain kind: UsageKind,
    }
}
mirror_record! {
    /// A game started by the launcher and not yet counted.
    PendingSession <=> core::PendingSession {
        /// The game.
        plain game_id: i64,
        /// Package whose usage events tell the play time.
        plain package: String,
        /// Launch time, Unix milliseconds.
        plain started_ms: i64,
    }
}
mirror_record! {
    /// Play time of a game.
    PlayStats <=> db::PlayStats {
        /// Seconds measured on this device.
        plain device_seconds: i64,
        /// Seconds Steam reports for the account.
        plain steam_seconds: i64,
        /// Sessions on this device.
        plain sessions: i64,
        /// Unix seconds of the last session, 0 when never played.
        plain last_played: i64,
    }
}

/// What closing a session did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SessionEnd {
    /// Nothing was pending.
    NoneOpen,
    /// The game is still in front; the session stays open.
    StillPlaying,
    /// Too short, or too old, to count.
    Dropped,
    /// Recorded.
    Recorded {
        /// The game.
        game_id: i64,
        /// Seconds counted.
        seconds: i64,
    },
}

impl From<core::SessionEnd> for SessionEnd {
    /// One to one.
    fn from(e: core::SessionEnd) -> Self {
        match e {
            core::SessionEnd::NoneOpen => SessionEnd::NoneOpen,
            core::SessionEnd::StillPlaying => SessionEnd::StillPlaying,
            core::SessionEnd::Dropped => SessionEnd::Dropped,
            core::SessionEnd::Recorded { game_id, seconds } => {
                SessionEnd::Recorded { game_id, seconds }
            }
        }
    }
}

#[uniffi::export]
impl Core {
    /// Notes that `game_id` was started through `package` at `now_ms`.
    pub fn begin_session(&self, game_id: i64, package: String, now_ms: i64) -> Res<()> {
        Ok(self.service().begin_session(game_id, &package, now_ms)?)
    }

    /// The launch waiting to be counted, if any.
    pub fn pending_session(&self) -> Res<Option<PendingSession>> {
        Ok(self.service().pending_session()?.map(Into::into))
    }

    /// Closes the pending session at `now_ms` from the usage events of its package, or from
    /// the launch time when `events` is `None` (no usage access).
    pub fn end_session(&self, events: Option<Vec<UsageEvent>>, now_ms: i64) -> Res<SessionEnd> {
        let events: Option<Vec<sessions::UsageEvent>> =
            events.map(|list| list.into_iter().map(Into::into).collect());
        Ok(self
            .service()
            .end_session(events.as_deref(), now_ms)?
            .into())
    }

    /// Play time of one game.
    pub fn play_stats(&self, game_id: i64) -> Res<PlayStats> {
        Ok(self.service().play_stats(game_id)?.into())
    }

    /// Marks or unmarks a game as a favourite.
    pub fn set_favorite(&self, game_id: i64, favorite: bool) -> Res<()> {
        Ok(self.service().set_favorite(game_id, favorite)?)
    }

    /// Hides a game (only the Hidden folder lists it) or shows it again.
    pub fn set_hidden(&self, game_id: i64, hidden: bool) -> Res<()> {
        Ok(self.service().set_hidden(game_id, hidden)?)
    }

    /// Whether a game is hidden.
    pub fn is_hidden(&self, game_id: i64) -> Res<bool> {
        Ok(self.service().game_flags(game_id)?.hidden)
    }
}
