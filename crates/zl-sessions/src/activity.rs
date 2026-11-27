//! The numbers of a game's Activity tab (RF-31): total time, number of sessions, average, last
//! session and the time of each of the last weeks, computed from the recorded sessions.
//!
//! Weeks run from Monday to Sunday in the user's time zone, which the app passes as an offset
//! from UTC; a session belongs to the week it started in.

/// Seconds in a day.
const DAY: i64 = 86_400;

/// Days from a Monday to the Unix epoch (a Thursday): the epoch is day 3 of its week, Monday
/// being day 0.
const EPOCH_WEEKDAY: i64 = 3;

/// One recorded session, as the calculations see it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRecord {
    /// Start, Unix seconds.
    pub started_at: i64,
    /// End, Unix seconds.
    pub ended_at: i64,
    /// Seconds played.
    pub seconds: i64,
}

/// The time played in one week.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeekBucket {
    /// The Monday 00:00 (local time) that opens the week, Unix seconds.
    pub week_start: i64,
    /// Seconds played in the week.
    pub seconds: i64,
}

/// What the Activity tab shows besides the list of sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivitySummary {
    /// Seconds played in all sessions.
    pub total_seconds: i64,
    /// Number of sessions.
    pub session_count: i64,
    /// Seconds per session, rounded down; 0 with no session.
    pub average_seconds: i64,
    /// End of the latest session, Unix seconds; 0 when there is none.
    pub last_played: i64,
    /// The last weeks, oldest first, the current week the last one.
    pub weeks: Vec<WeekBucket>,
}

/// Index of the week of `at` (Unix seconds) in a time zone `utc_offset` seconds east of UTC.
fn week_index(at: i64, utc_offset: i64) -> i64 {
    let day = (at + utc_offset).div_euclid(DAY);
    (day + EPOCH_WEEKDAY).div_euclid(7)
}

/// The Monday 00:00 (local time) of the week with this index, as Unix seconds.
fn week_start_of(index: i64, utc_offset: i64) -> i64 {
    (index * 7 - EPOCH_WEEKDAY) * DAY - utc_offset
}

/// The first instant (Unix seconds) of the week `at` falls in, in a time zone `utc_offset` seconds
/// east of UTC.
pub fn week_start(at: i64, utc_offset: i64) -> i64 {
    week_start_of(week_index(at, utc_offset), utc_offset)
}

/// Totals of `sessions` and the time of each of the last `weeks` weeks up to the week of `now`.
pub fn summarize(
    sessions: &[SessionRecord],
    now: i64,
    utc_offset: i64,
    weeks: usize,
) -> ActivitySummary {
    let total_seconds: i64 = sessions.iter().map(|s| s.seconds).sum();
    let session_count = sessions.len() as i64;
    let current = week_index(now, utc_offset);
    let first = current - weeks as i64 + 1;
    let mut buckets: Vec<WeekBucket> = (first..=current)
        .map(|index| WeekBucket {
            week_start: week_start_of(index, utc_offset),
            seconds: 0,
        })
        .collect();
    for s in sessions {
        let index = week_index(s.started_at, utc_offset);
        if (first..=current).contains(&index) {
            buckets[(index - first) as usize].seconds += s.seconds;
        }
    }
    ActivitySummary {
        total_seconds,
        session_count,
        average_seconds: if session_count > 0 {
            total_seconds / session_count
        } else {
            0
        },
        last_played: sessions.iter().map(|s| s.ended_at).max().unwrap_or(0),
        weeks: buckets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monday 2024-01-01 00:00 UTC.
    const MONDAY: i64 = 1_704_067_200;

    /// A session starting at `at` that lasts `seconds`.
    fn session(at: i64, seconds: i64) -> SessionRecord {
        SessionRecord {
            started_at: at,
            ended_at: at + seconds,
            seconds,
        }
    }

    /// Weeks open on Monday: Sunday night belongs to the old week, Monday 00:00 to the new one.
    #[test]
    fn weeks_open_on_monday() {
        assert_eq!(week_start(MONDAY, 0), MONDAY);
        assert_eq!(week_start(MONDAY - 1, 0), MONDAY - 7 * DAY);
        assert_eq!(week_start(MONDAY + 6 * DAY + 3_600, 0), MONDAY);
        assert_eq!(week_start(0, 0), -3 * DAY, "the epoch is a Thursday");
    }

    /// The time zone moves the boundary: 23:00 UTC on Sunday is already Monday in UTC+2.
    #[test]
    fn the_time_zone_moves_the_boundary() {
        let sunday_late = MONDAY - 3_600;
        assert_eq!(week_start(sunday_late, 0), MONDAY - 7 * DAY);
        assert_eq!(week_start(sunday_late, 2 * 3_600), MONDAY - 2 * 3_600);
        assert_eq!(
            week_start(sunday_late, -5 * 3_600),
            MONDAY - 7 * DAY + 5 * 3_600
        );
    }

    /// Totals, average and last session; the weeks fill from the oldest, the current one last.
    #[test]
    fn summary_counts_totals_and_buckets() {
        let now = MONDAY + 2 * DAY;
        let sessions = [
            session(MONDAY + 3_600, 3_600),
            session(MONDAY - DAY, 1_800),
            session(MONDAY - 9 * DAY, 600),
            session(MONDAY - 40 * DAY, 7_200),
        ];
        let s = summarize(&sessions, now, 0, 3);
        assert_eq!(s.total_seconds, 3_600 + 1_800 + 600 + 7_200);
        assert_eq!(s.session_count, 4);
        assert_eq!(s.average_seconds, 13_200 / 4);
        assert_eq!(s.last_played, MONDAY + 7_200);
        let seconds: Vec<i64> = s.weeks.iter().map(|w| w.seconds).collect();
        assert_eq!(
            seconds,
            vec![600, 1_800, 3_600],
            "the session of 40 days ago is out of range"
        );
        assert_eq!(s.weeks[2].week_start, MONDAY);
        assert_eq!(s.weeks[0].week_start, MONDAY - 14 * DAY);
    }

    /// With no session everything is zero, and the chart still has its weeks.
    #[test]
    fn an_empty_history_is_all_zero() {
        let s = summarize(&[], MONDAY, 3_600, 4);
        assert_eq!(
            (
                s.total_seconds,
                s.session_count,
                s.average_seconds,
                s.last_played
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(s.weeks.len(), 4);
        assert!(s.weeks.iter().all(|w| w.seconds == 0));
        assert!(summarize(&[], MONDAY, 0, 0).weeks.is_empty());
    }
}
