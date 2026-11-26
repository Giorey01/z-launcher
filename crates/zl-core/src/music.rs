//! Which tracks of the theme's background music play now: the playlist (in order or shuffled)
//! or, in time mode, the track of the schedule whose start is the latest one already passed.

use zl_theme::{MusicMode, PlaybackMode};

use crate::appearance_types::{MusicSetup, ScheduleTrack};

/// Minutes in a day, for the schedule's clock.
const MINUTES_PER_DAY: u32 = 24 * 60;

/// The tracks to play now, in order; empty when music is off or the theme has none.
/// `minute_of_day` is the local time (0-1439), `seed` varies the shuffle between sessions.
pub fn music_queue(setup: &MusicSetup, minute_of_day: u32, seed: u64) -> Vec<String> {
    match setup.mode {
        MusicMode::Disabled => Vec::new(),
        MusicMode::Playlist => match setup.playback_mode {
            PlaybackMode::InOrder => setup.playlist.clone(),
            PlaybackMode::Shuffle => shuffled(&setup.playlist, seed),
        },
        MusicMode::Time => scheduled(&setup.schedule, minute_of_day % MINUTES_PER_DAY)
            .map(|t| vec![t.path.clone()])
            .unwrap_or_default(),
    }
}

/// The track whose start is the latest at or before `minute`; before the first start of the
/// day, the last one of the day before keeps playing.
fn scheduled(schedule: &[ScheduleTrack], minute: u32) -> Option<&ScheduleTrack> {
    let start = |t: &ScheduleTrack| u32::from(t.hour) * 60 + u32::from(t.minute);
    schedule
        .iter()
        .filter(|t| start(t) <= minute)
        .max_by_key(|t| start(t))
        .or_else(|| schedule.iter().max_by_key(|t| start(t)))
}

/// `tracks` in an order decided by `seed` (Fisher-Yates over a xorshift sequence).
fn shuffled(tracks: &[String], seed: u64) -> Vec<String> {
    let mut out = tracks.to_vec();
    let mut state = seed | 1;
    for i in (1..out.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = usize::try_from(state % (i as u64 + 1)).unwrap_or(0);
        out.swap(i, j);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A setup with three playlist tracks and a two-entry schedule.
    fn setup(mode: MusicMode, playback_mode: PlaybackMode) -> MusicSetup {
        let track = |hour, path: &str| ScheduleTrack {
            hour,
            minute: 0,
            path: path.into(),
        };
        MusicSetup {
            mode,
            playback_mode,
            playlist: vec!["a".into(), "b".into(), "c".into()],
            schedule: vec![track(6, "day"), track(20, "night")],
        }
    }

    #[test]
    fn music_off_plays_nothing() {
        assert!(music_queue(&setup(MusicMode::Disabled, PlaybackMode::InOrder), 0, 1).is_empty());
    }

    #[test]
    fn a_playlist_in_order_keeps_the_theme_order() {
        let q = music_queue(&setup(MusicMode::Playlist, PlaybackMode::InOrder), 0, 1);
        assert_eq!(q, vec!["a", "b", "c"]);
    }

    #[test]
    fn a_shuffled_playlist_keeps_every_track_once() {
        let mut q = music_queue(&setup(MusicMode::Playlist, PlaybackMode::Shuffle), 0, 42);
        q.sort();
        assert_eq!(q, vec!["a", "b", "c"]);
    }

    #[test]
    fn the_schedule_plays_the_latest_track_started() {
        let s = setup(MusicMode::Time, PlaybackMode::InOrder);
        assert_eq!(music_queue(&s, 6 * 60, 0), vec!["day"]);
        assert_eq!(music_queue(&s, 19 * 60 + 59, 0), vec!["day"]);
        assert_eq!(music_queue(&s, 21 * 60, 0), vec!["night"]);
    }

    #[test]
    fn before_the_first_start_the_night_track_goes_on() {
        let s = setup(MusicMode::Time, PlaybackMode::InOrder);
        assert_eq!(music_queue(&s, 3 * 60, 0), vec!["night"]);
    }
}
