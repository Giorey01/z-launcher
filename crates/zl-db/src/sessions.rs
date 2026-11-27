//! Repository: the user's flags on games and the play sessions.

use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};

use crate::db::Db;
use crate::error::DbError;

/// Where the time of a session comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionSource {
    /// Foreground time from the system's app usage events.
    Usage,
    /// Launch to return, when the usage events cannot be read.
    Estimated,
    /// The account's total as Steam reports it.
    Steam,
    /// Entered or corrected by hand by the user (RF-41).
    Manual,
}

impl SessionSource {
    /// Stable spelling stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            SessionSource::Usage => "usage",
            SessionSource::Estimated => "estimated",
            SessionSource::Steam => "steam",
            SessionSource::Manual => "manual",
        }
    }

    /// The source stored under `text`, `None` for a spelling this version does not know.
    pub fn parse(text: &str) -> Option<SessionSource> {
        [
            SessionSource::Usage,
            SessionSource::Estimated,
            SessionSource::Steam,
            SessionSource::Manual,
        ]
        .into_iter()
        .find(|s| s.as_str() == text)
    }
}

/// A recorded session as stored, with its id so it can be corrected or removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRow {
    /// Row id.
    pub id: i64,
    /// The game.
    pub game_id: i64,
    /// Start, Unix seconds.
    pub started_at: i64,
    /// End, Unix seconds.
    pub ended_at: i64,
    /// Seconds played.
    pub seconds: i64,
    /// Where the time comes from.
    pub source: SessionSource,
}

/// The favourite and hidden flags of a game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameFlags {
    /// Marked as a favourite.
    pub favorite: bool,
    /// Hidden from every list but the hidden one.
    pub hidden: bool,
}

/// Play time of a game, summed by source.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayStats {
    /// Seconds measured on this device (usage and estimated sessions).
    pub device_seconds: i64,
    /// Seconds Steam reports for the account.
    pub steam_seconds: i64,
    /// Sessions measured on this device.
    pub sessions: i64,
    /// Unix time (seconds) of the last session of any source, 0 when never played.
    pub last_played: i64,
}

/// A play session to record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewSession {
    /// The game.
    pub game_id: i64,
    /// Start, Unix seconds.
    pub started_at: i64,
    /// End, Unix seconds.
    pub ended_at: i64,
    /// Seconds played (foreground time; can be less than end minus start).
    pub seconds: i64,
    /// Where the time comes from.
    pub source: SessionSource,
}

impl Db {
    /// The flags of one game (both off when never set).
    pub fn game_flags(&self, game_id: i64) -> Result<GameFlags, DbError> {
        let row = self
            .conn
            .query_row(
                "SELECT favorite, hidden FROM game_flags WHERE game_id = ?1",
                params![game_id],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
            )
            .optional()?;
        Ok(row.map_or_else(GameFlags::default, |(f, h)| GameFlags {
            favorite: f != 0,
            hidden: h != 0,
        }))
    }

    /// The flags of every game that has one set.
    pub fn all_game_flags(&self) -> Result<HashMap<i64, GameFlags>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT game_id, favorite, hidden FROM game_flags")?;
        let rows = stmt.query_map([], |r| {
            let flags = GameFlags {
                favorite: r.get::<_, i64>(1)? != 0,
                hidden: r.get::<_, i64>(2)? != 0,
            };
            Ok((r.get::<_, i64>(0)?, flags))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Stores the flags of one game.
    pub fn set_game_flags(&self, game_id: i64, flags: GameFlags) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO game_flags (game_id, favorite, hidden) VALUES (?1, ?2, ?3)
             ON CONFLICT(game_id) DO UPDATE SET favorite = excluded.favorite,
                                                hidden = excluded.hidden",
            params![game_id, i64::from(flags.favorite), i64::from(flags.hidden)],
        )?;
        Ok(())
    }

    /// Records one session; its row id.
    pub fn add_session(&self, s: &NewSession) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT INTO game_sessions (game_id, started_at, ended_at, seconds, source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                s.game_id,
                s.started_at,
                s.ended_at,
                s.seconds.max(0),
                s.source.as_str()
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// The sessions measured or entered on this device for a game (Steam's total is not a
    /// session), newest first.
    pub fn game_sessions(&self, game_id: i64) -> Result<Vec<SessionRow>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, game_id, started_at, ended_at, seconds, source FROM game_sessions
             WHERE game_id = ?1 AND source <> 'steam' ORDER BY started_at DESC, id DESC",
        )?;
        let rows = stmt.query_map(params![game_id], |r| {
            let source: String = r.get(5)?;
            Ok(SessionRow {
                id: r.get(0)?,
                game_id: r.get(1)?,
                started_at: r.get(2)?,
                ended_at: r.get(3)?,
                seconds: r.get(4)?,
                source: SessionSource::parse(&source).unwrap_or(SessionSource::Estimated),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Sets the duration of a session that is not Steam's total, moving its end to match.
    /// `false` when there is no such session.
    pub fn set_session_seconds(&self, id: i64, seconds: i64) -> Result<bool, DbError> {
        let changed = self.conn.execute(
            "UPDATE game_sessions SET seconds = ?2, ended_at = started_at + ?2
             WHERE id = ?1 AND source <> 'steam'",
            params![id, seconds.max(0)],
        )?;
        Ok(changed > 0)
    }

    /// Removes a session that is not Steam's total. `false` when there is no such session.
    pub fn delete_session(&self, id: i64) -> Result<bool, DbError> {
        let changed = self.conn.execute(
            "DELETE FROM game_sessions WHERE id = ?1 AND source <> 'steam'",
            params![id],
        )?;
        Ok(changed > 0)
    }

    /// Leaves an app out of the tracking, or puts it back (RF-41).
    pub fn set_app_excluded(&self, package: &str, excluded: bool) -> Result<(), DbError> {
        if excluded {
            self.conn.execute(
                "INSERT OR IGNORE INTO session_excluded_apps (package) VALUES (?1)",
                params![package],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM session_excluded_apps WHERE package = ?1",
                params![package],
            )?;
        }
        Ok(())
    }

    /// Whether the app's play time is left out of the tracking.
    pub fn is_app_excluded(&self, package: &str) -> Result<bool, DbError> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM session_excluded_apps WHERE package = ?1",
                params![package],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    /// Replaces the Steam total of a game (`seconds` 0 removes it).
    pub fn set_steam_playtime(
        &self,
        game_id: i64,
        seconds: i64,
        last_played: i64,
    ) -> Result<(), DbError> {
        self.transaction::<(), DbError>(|db| {
            db.conn.execute(
                "DELETE FROM game_sessions WHERE game_id = ?1 AND source = 'steam'",
                params![game_id],
            )?;
            if seconds > 0 {
                db.add_session(&NewSession {
                    game_id,
                    started_at: last_played,
                    ended_at: last_played,
                    seconds,
                    source: SessionSource::Steam,
                })?;
            }
            Ok(())
        })
    }

    /// Play time of every game that has some, by game.
    pub fn all_play_stats(&self) -> Result<HashMap<i64, PlayStats>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT game_id,
                    sum(CASE WHEN source = 'steam' THEN 0 ELSE seconds END),
                    sum(CASE WHEN source = 'steam' THEN seconds ELSE 0 END),
                    sum(CASE WHEN source = 'steam' THEN 0 ELSE 1 END),
                    max(ended_at)
             FROM game_sessions GROUP BY game_id",
        )?;
        let rows = stmt.query_map([], |r| {
            let stats = PlayStats {
                device_seconds: r.get(1)?,
                steam_seconds: r.get(2)?,
                sessions: r.get(3)?,
                last_played: r.get(4)?,
            };
            Ok((r.get::<_, i64>(0)?, stats))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Play time of one game.
    pub fn play_stats(&self, game_id: i64) -> Result<PlayStats, DbError> {
        Ok(self.all_play_stats()?.remove(&game_id).unwrap_or_default())
    }
}
