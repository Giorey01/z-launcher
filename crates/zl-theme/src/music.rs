//! Music and volume fields of `theme.json` (spec 3.6), which live at the top level.

use serde_json::{Map, Value};

use crate::enums::StringEnum;
use crate::issue::IssueCode;
use crate::reader::{join, Reader};

string_enum! {
    /// How background music is chosen.
    MusicMode {
        /// By time of day, from `music_time_schedule`.
        Time => "TIME",
        /// From `music_playlist`.
        Playlist => "PLAYLIST",
        /// No music.
        Disabled => "DISABLED",
    }
}

string_enum! {
    /// Order in which a playlist plays.
    PlaybackMode {
        /// In the listed order.
        InOrder => "IN ORDER",
        /// Shuffled.
        Shuffle => "SHUFFLE",
    }
}

/// A track that starts at a time of day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleEntry {
    /// Hour, 0-23 (written without a leading zero in real packs).
    pub hour: u8,
    /// Minute, 0-59.
    pub minute: u8,
    /// Track path relative to `music/`.
    pub file: String,
}

/// Music-related fields; each `None` means the field is absent.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MusicConfig {
    /// `music_mode`.
    pub mode: Option<MusicMode>,
    /// `music_playlist`, split on `||`.
    pub playlist: Option<Vec<String>>,
    /// `music_time_schedule`, split on `||` into `H:M|file` entries.
    pub schedule: Option<Vec<ScheduleEntry>>,
    /// `music_playback_mode`.
    pub playback_mode: Option<PlaybackMode>,
    /// `sfx_volume`, 0.0-1.0.
    pub sfx_volume: Option<f64>,
    /// `music_volume`, 0.0-1.0.
    pub music_volume: Option<f64>,
    /// `jingle_volume`, 0.0-1.0.
    pub jingle_volume: Option<f64>,
}

/// Range of the volume fields.
const VOLUME: (f64, f64) = (0.0, 1.0);
/// Separator between list items inside a music string.
const LIST_SEPARATOR: &str = "||";

/// Reads the music and volume fields of the top-level object (path `$`).
pub(crate) fn parse_music(obj: &Map<String, Value>, reader: &mut Reader) -> MusicConfig {
    MusicConfig {
        mode: reader.enum_value(obj, "$", "music_mode"),
        playlist: reader
            .string(obj, "$", "music_playlist")
            .filter(|s| !s.trim().is_empty())
            .map(|s| split_list(&s)),
        schedule: parse_schedule(obj, reader),
        playback_mode: parse_playback_mode(obj, reader),
        sfx_volume: reader.number(obj, "$", "sfx_volume", Some(VOLUME)),
        music_volume: reader.number(obj, "$", "music_volume", Some(VOLUME)),
        jingle_volume: reader.number(obj, "$", "jingle_volume", Some(VOLUME)),
    }
}

/// Splits a `||`-separated string, trimming items and dropping empty ones.
fn split_list(text: &str) -> Vec<String> {
    text.split(LIST_SEPARATOR)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Reads `music_playback_mode`, tolerating `IN_ORDER` for `IN ORDER`.
fn parse_playback_mode(obj: &Map<String, Value>, reader: &mut Reader) -> Option<PlaybackMode> {
    let text = reader.string(obj, "$", "music_playback_mode")?;
    let parsed = PlaybackMode::parse_text(&text.replace('_', " "));
    if parsed.is_none() {
        reader.warn(
            IssueCode::UnknownEnumValue,
            join("$", "music_playback_mode"),
        );
    }
    parsed
}

/// Reads `music_time_schedule`, skipping entries that are not `H:M|file`.
fn parse_schedule(obj: &Map<String, Value>, reader: &mut Reader) -> Option<Vec<ScheduleEntry>> {
    let text = reader
        .string(obj, "$", "music_time_schedule")
        .filter(|s| !s.trim().is_empty())?;
    let mut entries = Vec::new();
    for item in split_list(&text) {
        match parse_entry(&item) {
            Some(entry) => entries.push(entry),
            None => reader.warn(IssueCode::BadSchedule, join("$", "music_time_schedule")),
        }
    }
    Some(entries)
}

/// Parses one `H:M|file` entry.
fn parse_entry(item: &str) -> Option<ScheduleEntry> {
    let (time, file) = item.split_once('|')?;
    let (h, m) = time.trim().split_once(':')?;
    let (hour, minute) = (h.trim().parse::<u8>().ok()?, m.trim().parse::<u8>().ok()?);
    let file = file.trim();
    if hour > 23 || minute > 59 || file.is_empty() {
        return None;
    }
    Some(ScheduleEntry {
        hour,
        minute,
        file: file.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Entries are split on `||` and the time is validated.
    #[test]
    fn schedule_entries() {
        assert_eq!(
            parse_entry("06:30|a/b.mp3"),
            Some(ScheduleEntry {
                hour: 6,
                minute: 30,
                file: "a/b.mp3".into()
            })
        );
        assert_eq!(
            parse_entry("0:0|music/Yotsuba dekakeru.mp3"),
            Some(ScheduleEntry {
                hour: 0,
                minute: 0,
                file: "music/Yotsuba dekakeru.mp3".into()
            })
        );
        assert_eq!(
            parse_entry("4:0|m.mp3").map(|e| (e.hour, e.minute)),
            Some((4, 0))
        );
        assert_eq!(parse_entry("24:00|x.mp3"), None);
        assert_eq!(parse_entry("12:60|x.mp3"), None);
        assert_eq!(parse_entry("noon|x.mp3"), None);
        assert_eq!(
            split_list("a.mp3||b.mp3|| ||c.mp3"),
            vec!["a.mp3", "b.mp3", "c.mp3"]
        );
    }
}
