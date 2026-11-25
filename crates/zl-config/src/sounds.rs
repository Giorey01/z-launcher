//! The launcher's own interface sounds, used for every action the active theme has no file for.
//!
//! Each sound is a short mix of tones synthesised by the core (no recorded audio, so nothing to
//! license). A tone glides from `from_hz` to `to_hz` over `ms`, starts `delay_ms` after the start
//! of the sound, rises in `attack_ms` and fades out to its end. They are kept short and soft:
//! navigation is heard many times a minute, so it is the quietest and shortest of all.

/// Waveform of a tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wave {
    /// Pure and soft: chimes, confirmations.
    Sine,
    /// A little brighter than a sine: clicks and ticks.
    Triangle,
    /// Hollow and buzzy, kept for the error.
    Square,
}

/// One tone of a sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    /// Waveform.
    pub wave: Wave,
    /// Pitch at the start, in hertz.
    pub from_hz: f32,
    /// Pitch at the end, in hertz (the same as `from_hz` for a steady note).
    pub to_hz: f32,
    /// Start, in milliseconds after the start of the sound.
    pub delay_ms: u32,
    /// Length, in milliseconds.
    pub ms: u32,
    /// Rise time, in milliseconds.
    pub attack_ms: u32,
    /// Loudness, 0 to 1, before the sound is normalised to [`PEAK`].
    pub gain: f32,
}

/// Sample rate of the generated files. 44.1 kHz is what Android's mixer runs at on most devices,
/// so SoundPool does not resample.
pub const SAMPLE_RATE: u32 = 44_100;

/// Peak level of the loudest sound, 0 to 1. Leaves headroom when several sounds overlap.
pub const PEAK: f32 = 0.5;

/// Bump this when a recipe changes, so devices generate the files again.
pub const RECIPES_VERSION: u32 = 1;

/// Folder (under the app's data folder) of the generated files.
pub const DEFAULT_SOUNDS_FOLDER: &str = "sounds/default";

/// A tone (shorter to write in the table below): pitch `(from, to)` in hertz, timing
/// `(delay, length, attack)` in milliseconds, and gain.
const fn t(wave: Wave, hz: (f32, f32), timing: (u32, u32, u32), gain: f32) -> Tone {
    Tone {
        wave,
        from_hz: hz.0,
        to_hz: hz.1,
        delay_ms: timing.0,
        ms: timing.1,
        attack_ms: timing.2,
        gain,
    }
}

/// Notes used below (equal temperament, A4 = 440 Hz).
mod note {
    /// C5.
    pub const C5: f32 = 523.25;
    /// E5.
    pub const E5: f32 = 659.25;
    /// G5.
    pub const G5: f32 = 783.99;
    /// A5.
    pub const A5: f32 = 880.0;
    /// B5.
    pub const B5: f32 = 987.77;
    /// C6.
    pub const C6: f32 = 1046.5;
    /// D6.
    pub const D6: f32 = 1174.66;
    /// E6.
    pub const E6: f32 = 1318.51;
    /// G6.
    pub const G6: f32 = 1567.98;
}

use note::*;
use Wave::{Sine, Square, Triangle};

/// The recipe of each action, by the action's name in theme files (`navigate`, `select`, ...).
/// Gains are relative within one sound; navigation and sliders get lower ones because the files
/// are normalised to the same peak and those repeat the most.
pub const RECIPES: &[(&str, &[Tone])] = &[
    (
        "navigate",
        &[t(Triangle, (1800.0, 1500.0), (0, 22, 1), 0.35)],
    ),
    (
        "select",
        &[
            t(Sine, (A5, A5), (0, 45, 2), 0.8),
            t(Sine, (E6, E6), (38, 70, 2), 0.8),
        ],
    ),
    (
        "back",
        &[
            t(Sine, (E6, E6), (0, 45, 2), 0.8),
            t(Sine, (A5, A5), (38, 70, 2), 0.8),
        ],
    ),
    (
        "folder_open",
        &[
            t(Sine, (520.0, 980.0), (0, 95, 4), 0.8),
            t(Triangle, (1040.0, 1960.0), (0, 95, 4), 0.15),
        ],
    ),
    (
        "folder_close",
        &[
            t(Sine, (980.0, 520.0), (0, 95, 4), 0.8),
            t(Triangle, (1960.0, 1040.0), (0, 95, 4), 0.15),
        ],
    ),
    (
        "launch",
        &[
            t(Sine, (C5, C5), (0, 90, 3), 0.7),
            t(Sine, (E5, E5), (60, 90, 3), 0.7),
            t(Sine, (G5, G5), (120, 90, 3), 0.7),
            t(Sine, (C6, C6), (180, 220, 3), 0.9),
        ],
    ),
    (
        "error",
        &[
            t(Square, (220.0, 190.0), (0, 90, 3), 0.45),
            t(Square, (196.0, 170.0), (120, 130, 3), 0.45),
        ],
    ),
    (
        "notification",
        &[
            t(Sine, (C6, C6), (0, 160, 3), 0.8),
            t(Sine, (G6, G6), (90, 240, 3), 0.6),
        ],
    ),
    ("discord_open", &[t(Sine, (700.0, 1100.0), (0, 70, 3), 0.8)]),
    (
        "discord_close",
        &[t(Sine, (1100.0, 700.0), (0, 70, 3), 0.8)],
    ),
    (
        "screen_swap",
        &[
            t(Triangle, (400.0, 1200.0), (0, 80, 5), 0.5),
            t(Triangle, (1200.0, 600.0), (80, 90, 5), 0.5),
        ],
    ),
    ("grid_zoom_in", &[t(Sine, (600.0, 1000.0), (0, 55, 2), 0.7)]),
    (
        "grid_zoom_out",
        &[t(Sine, (1000.0, 600.0), (0, 55, 2), 0.7)],
    ),
    (
        "reorder_pickup",
        &[t(Triangle, (700.0, 950.0), (0, 45, 2), 0.6)],
    ),
    (
        "reorder_place",
        &[t(Triangle, (950.0, 620.0), (0, 55, 2), 0.6)],
    ),
    (
        "open_edit",
        &[
            t(Sine, (C6, C6), (0, 35, 2), 0.7),
            t(Sine, (G6, G6), (30, 55, 2), 0.7),
        ],
    ),
    ("pop", &[t(Sine, (1300.0, 420.0), (0, 40, 1), 0.8)]),
    (
        "slider_increase",
        &[t(Triangle, (1500.0, 1600.0), (0, 18, 1), 0.3)],
    ),
    (
        "slider_decrease",
        &[t(Triangle, (1200.0, 1100.0), (0, 18, 1), 0.3)],
    ),
    ("saving", &[t(Sine, (E5, E5), (0, 80, 6), 0.6)]),
    (
        "saving_end",
        &[
            t(Sine, (E5, E5), (0, 70, 4), 0.6),
            t(Sine, (B5, B5), (60, 120, 4), 0.6),
        ],
    ),
    (
        "retroachievements",
        &[
            t(Sine, (G5, G5), (0, 100, 3), 0.7),
            t(Sine, (B5, B5), (80, 100, 3), 0.7),
            t(Sine, (D6, D6), (160, 100, 3), 0.7),
            t(Sine, (G6, G6), (240, 320, 3), 0.9),
        ],
    ),
];
