//! Play time from the system's app usage events (RF-40), with no service running while a game
//! is played: the launcher notes when it started a game, and when it is in front again it reads
//! the usage events of the game's package and counts the time the game was in the foreground.
//!
//! The events are the ones Android's usage statistics report for activities: one becomes
//! visible and active (resumed) or stops being so (paused). A game can have several activities,
//! so the game counts as in front while at least one of them is resumed; a pause (Home pressed,
//! a call, the screen off) stops the count and a resume starts it again.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]

use serde::{Deserialize, Serialize};

/// What happened to an activity of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageKind {
    /// An activity came to the foreground.
    Resumed,
    /// An activity left the foreground.
    Paused,
}

/// One usage event of the game's package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageEvent {
    /// When, in Unix milliseconds.
    pub at_ms: i64,
    /// What.
    pub kind: UsageKind,
}

/// The foreground time of a game between `from_ms` and `to_ms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Foreground {
    /// Milliseconds the game was in front, clipped to the window.
    pub ms: i64,
    /// Whether the game is still in front at `to_ms` (it was not paused after its last resume).
    pub still_running: bool,
    /// Start of the first foreground stretch, if there was one.
    pub first_ms: Option<i64>,
    /// End of the last foreground stretch, if there was one.
    pub last_ms: Option<i64>,
}

/// Counts the foreground time in `[from_ms, to_ms]` from `events` (in any order). Events before
/// the window only set whether the game was already in front when it opens.
pub fn foreground(events: &[UsageEvent], from_ms: i64, to_ms: i64) -> Foreground {
    let mut sorted: Vec<UsageEvent> = events
        .iter()
        .copied()
        .filter(|e| e.at_ms <= to_ms)
        .collect();
    sorted.sort_by_key(|e| (e.at_ms, e.kind == UsageKind::Resumed));
    let mut out = Foreground {
        ms: 0,
        still_running: false,
        first_ms: None,
        last_ms: None,
    };
    let mut resumed = 0_u32;
    let mut since: Option<i64> = None;
    for event in sorted {
        let at = event.at_ms.max(from_ms);
        match event.kind {
            UsageKind::Resumed => {
                resumed += 1;
                since.get_or_insert(at);
            }
            UsageKind::Paused => {
                resumed = resumed.saturating_sub(1);
                if resumed == 0 {
                    if let Some(start) = since.take() {
                        close(&mut out, start, at);
                    }
                }
            }
        }
    }
    if let Some(start) = since {
        close(&mut out, start, to_ms);
        out.still_running = true;
    }
    out
}

/// Adds the stretch `[start, end]` to `out` when it lies in the window.
fn close(out: &mut Foreground, start: i64, end: i64) {
    if end <= start {
        return;
    }
    out.ms += end - start;
    out.first_ms.get_or_insert(start);
    out.last_ms = Some(end);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand for an event.
    fn ev(at_ms: i64, kind: UsageKind) -> UsageEvent {
        UsageEvent { at_ms, kind }
    }

    use UsageKind::{Paused, Resumed};

    /// One stretch, then a pause and a resume: the pause does not count.
    #[test]
    fn pauses_do_not_count() {
        let events = [
            ev(1_000, Resumed),
            ev(61_000, Paused),
            ev(121_000, Resumed),
            ev(181_000, Paused),
        ];
        let f = foreground(&events, 0, 300_000);
        assert_eq!(f.ms, 120_000);
        assert!(!f.still_running);
        assert_eq!((f.first_ms, f.last_ms), (Some(1_000), Some(181_000)));
    }

    /// Two activities that overlap count once, and a game still in front is reported so.
    #[test]
    fn overlapping_activities_and_running_games() {
        let events = [
            ev(0, Resumed),
            ev(10_000, Resumed),
            ev(10_001, Paused),
            ev(20_000, Paused),
        ];
        assert_eq!(foreground(&events, 0, 60_000).ms, 20_000);
        let running = foreground(&[ev(5_000, Resumed)], 0, 65_000);
        assert_eq!((running.ms, running.still_running), (60_000, true));
    }

    /// Time before the window and events after it are left out; a pause sorted at the same
    /// instant as a resume closes first.
    #[test]
    fn the_window_clips_the_time() {
        let events = [
            ev(-50_000, Resumed),
            ev(30_000, Paused),
            ev(30_000, Resumed),
            ev(90_000, Paused),
        ];
        let f = foreground(&events, 0, 60_000);
        assert_eq!(f.ms, 60_000);
        assert!(f.still_running, "the pause at 90 s is after the window");
        assert_eq!(foreground(&[], 0, 1_000).ms, 0);
        assert_eq!(
            foreground(&[ev(10, Paused)], 0, 1_000).ms,
            0,
            "a lone pause counts nothing"
        );
    }
}
