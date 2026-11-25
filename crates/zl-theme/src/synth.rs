//! Renders the launcher's own interface sounds (recipes in `zl_config::sounds`) to 16-bit mono
//! WAV files.
//!
//! Each tone glides exponentially from its start pitch to its end pitch, rises in its attack time
//! and then fades with a squared curve to silence at its last sample, so no sound starts or ends
//! with a click. The mix is normalised so the loudest sample sits at the configured peak.

use std::f32::consts::TAU;

use zl_config::sounds::{Tone, Wave, PEAK, RECIPES, SAMPLE_RATE};

/// Bytes of the RIFF/WAVE header written before the samples. A format fact.
const WAV_HEADER_LEN: usize = 44;

/// The recipe of an action, by its name in theme files.
pub fn recipe(action: &str) -> Option<&'static [Tone]> {
    RECIPES
        .iter()
        .find(|(name, _)| *name == action)
        .map(|(_, tones)| *tones)
}

/// Samples in `ms` milliseconds.
fn samples_in(ms: u32) -> usize {
    (u64::from(ms) * u64::from(SAMPLE_RATE) / 1000) as usize
}

/// One period of `wave` at `phase` (0 to 1).
fn oscillator(wave: Wave, phase: f32) -> f32 {
    match wave {
        Wave::Sine => (TAU * phase).sin(),
        Wave::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
        Wave::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

/// Envelope at sample `i` of `len`: linear rise over `attack`, then a squared fade to zero.
fn envelope(i: usize, len: usize, attack: usize) -> f32 {
    if i < attack {
        return i as f32 / attack as f32;
    }
    let rest = (len - i) as f32 / (len - attack).max(1) as f32;
    rest * rest
}

/// Adds one tone into `mix`.
fn add_tone(mix: &mut [f32], tone: &Tone) {
    let start = samples_in(tone.delay_ms);
    let len = samples_in(tone.ms).max(1);
    let attack = samples_in(tone.attack_ms).clamp(1, len);
    let ratio = tone.to_hz / tone.from_hz;
    let mut phase = 0.0_f32;
    for i in 0..len {
        let Some(slot) = mix.get_mut(start + i) else {
            break;
        };
        let progress = i as f32 / len as f32;
        let hz = tone.from_hz * ratio.powf(progress);
        *slot += tone.gain * envelope(i, len, attack) * oscillator(tone.wave, phase);
        phase = (phase + hz / SAMPLE_RATE as f32).fract();
    }
}

/// The samples of a sound, normalised to the peak.
pub fn render(tones: &[Tone]) -> Vec<i16> {
    let total = tones
        .iter()
        .map(|t| samples_in(t.delay_ms) + samples_in(t.ms).max(1))
        .max()
        .unwrap_or(0);
    let mut mix = vec![0.0_f32; total];
    for tone in tones {
        add_tone(&mut mix, tone);
    }
    let loudest = mix.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    let scale = if loudest > 0.0 { PEAK / loudest } else { 0.0 };
    mix.iter()
        .map(|s| (s * scale * f32::from(i16::MAX)).round() as i16)
        .collect()
}

/// A 16-bit mono PCM WAV file of `samples`.
pub fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let data_len = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(WAV_HEADER_LEN + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::StringEnum;
    use crate::sounds::SoundAction;

    /// Every action a theme can replace has a recipe of our own, and no recipe is unused.
    #[test]
    fn every_action_has_a_recipe() {
        for action in SoundAction::ALL {
            assert!(recipe(action.as_str()).is_some(), "{}", action.as_str());
        }
        assert_eq!(RECIPES.len(), SoundAction::ALL.len());
    }

    /// Sounds are short, peak at the configured level and start and end silent (no clicks).
    #[test]
    fn sounds_are_short_normalised_and_click_free() {
        let peak = (PEAK * f32::from(i16::MAX)).round() as i16;
        for (name, tones) in RECIPES {
            let samples = render(tones);
            assert!(
                !samples.is_empty() && samples.len() < samples_in(600),
                "{name}"
            );
            let loudest = samples.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
            assert!(
                loudest.abs_diff(peak.unsigned_abs()) <= 1,
                "{name}: {loudest}"
            );
            let edge = i16::MAX / 50;
            assert!(samples[0].abs() < edge, "{name} starts with a click");
            assert!(
                samples[samples.len() - 1].abs() < edge,
                "{name} ends with a click"
            );
        }
        let navigate = render(recipe("navigate").unwrap_or_default());
        let launch = render(recipe("launch").unwrap_or_default());
        assert!(
            navigate.len() * 5 < launch.len(),
            "navigation is the shortest kind of sound"
        );
    }

    /// The header is the canonical 44-byte PCM one and sizes match the samples.
    #[test]
    fn wav_header_describes_the_samples() {
        let bytes = wav_bytes(&[0, 1000, -1000]);
        assert_eq!(bytes.len(), WAV_HEADER_LEN + 6);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(
            u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            SAMPLE_RATE
        );
        assert_eq!(
            u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]),
            6
        );
        assert_eq!(i16::from_le_bytes([bytes[46], bytes[47]]), 1000);
    }
}
