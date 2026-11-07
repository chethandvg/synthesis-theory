// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! A soft voice with vibrato, and a heartbeat.

use crate::{Result, SynthError, finite, positive, require, time};
use pitch_math::{A4_MIDI, midi_to_hz_with_reference};
use signal_core::{adsr, n_samples, osc_sweep};
use std::f64::consts::PI;

/// The harmonic amplitudes of [`soft_voice`], as in `lifesound.py`.
const VOICE_PARTIALS: [f64; 4] = [1.0, 0.25, 0.08, 0.03];

const TOO_HIGH: &str = "at most f64::MAX / 4 (about 4.49e307), so that its 4th harmonic is finite";
const TOO_FAST: &str = "small enough in size that the vibrato's phase 2π·vibrato_hz·t stays finite";
const TOO_DEEP: &str = "small enough in size that every harmonic stays finite and above 0 Hz";

/// A gentle, flute- or voice-like tone for lullabies, as `lifesound.soft_voice`:
/// four harmonics (amplitudes 1, 0.25, 0.08, 0.03) of a pitch that wavers by
/// `vibrato_cents · sin(2π·vibrato_hz·t)` cents around `freq`, the vibrato fading
/// in over the first 0.4 s, shaped by `signal_core::adsr` (attack 0.08 s, decay
/// 0.2 s to 0.8, release `min(0.4, dur/3)` s). `lifesound.py`'s defaults are
/// `sr = 44100`, `vibrato_hz = 5.0` and `vibrato_cents = 12`.
///
/// The pitch `c` cents above `freq` is `pitch_math`'s equal-tempered note
/// `69 + c/100` with A4 tuned to `freq`, that is `freq · 2^(c/1200)`; each
/// harmonic is a `signal_core::osc_sweep`, whose phase integrates the changing
/// frequency.
///
/// `freq` must be greater than 0 and at most `f64::MAX / 4`, so that its 4th
/// harmonic is finite. `vibrato_hz` and `vibrato_cents` may be any finite number
/// for which the vibrato's phase `2π·vibrato_hz·t` stays finite up to the last
/// sample and every harmonic of the pitch stays finite and above 0 Hz (at
/// 220 Hz, `vibrato_cents` up to about 1.2e6 in size). Outside that,
/// `lifesound.py` gave NaN samples (or a 0 Hz pitch); this is an error naming
/// the argument. A `dur` too short for a sample gives an empty signal, as in
/// `lifesound.py`, for any finite `vibrato_hz` and `vibrato_cents` and any finite
/// `freq` above 0.
pub fn soft_voice(
    freq: f64,
    dur: f64,
    sr: u32,
    vibrato_hz: f64,
    vibrato_cents: f64,
) -> Result<Vec<f64>> {
    let n = n_samples(dur, sr)?;
    let top = VOICE_PARTIALS.len() as f64;
    let freq = positive("freq", freq)?;
    require(n == 0 || (top * freq).is_finite(), "freq", freq, TOO_HIGH)?;
    let rate = f64::from(sr);
    let w = 2.0 * PI * finite("vibrato_hz", vibrato_hz)?;
    let last = time(n.saturating_sub(1), rate);
    require(
        n == 0 || (w * last).is_finite(),
        "vibrato_hz",
        vibrato_hz,
        TOO_FAST,
    )?;
    let depth = finite("vibrato_cents", vibrato_cents)?;
    // With samples, freq · 4 and the vibrato's phase are finite, so only the depth can push a harmonic out of range.
    let pitch = (0..n)
        .map(|i| {
            let t = time(i, rate);
            let cents = depth * (w * t).sin() * (t / 0.4).min(1.0);
            match midi_to_hz_with_reference(A4_MIDI + cents / 100.0, freq) {
                Ok(f) if (top * f).is_finite() => Ok(f),
                _ => Err(SynthError::new("vibrato_cents", depth, TOO_DEEP)),
            }
        })
        .collect::<Result<Vec<f64>>>()?;
    let mut y = vec![0.0; n];
    let mut harmonic = vec![0.0; n];
    for (k, a) in VOICE_PARTIALS.iter().enumerate() {
        let r = (k + 1) as f64;
        harmonic
            .iter_mut()
            .zip(&pitch)
            .for_each(|(h, f)| *h = f * r);
        for (o, s) in y.iter_mut().zip(osc_sweep(&harmonic, sr, 0.0)?) {
            *o += a * s;
        }
    }
    let env = adsr(n, sr, 0.08, 0.2, 0.8, (dur / 3.0).min(0.4))?;
    y.iter_mut().zip(env).for_each(|(v, e)| *v *= e);
    Ok(y)
}

/// A "lub-dub" heartbeat at `bpm` beats per minute, as `lifesound.heartbeat`:
/// each beat is two low, damped thumps, S1 and then a softer S2 (amplitude 0.6)
/// `min(0.3, 0.4 · 60/bpm)` s later. A thump lasts 0.18 s:
/// `sin(2π·f0·t·(1 − 0.35·t/0.18)) · e^{−t/0.045}`, a pitch gliding down from
/// `f0`. A thump that would reach the end of the buffer is left out, as in
/// `lifesound.py`. Its defaults are `sr = 44100` and `f0 = 48.0`. `bpm` and `f0`
/// must be greater than 0, and `bpm` at most `60 · sr` (one beat per sample).
pub fn heartbeat(bpm: f64, dur: f64, sr: u32, f0: f64) -> Result<Vec<f64>> {
    let n = n_samples(dur, sr)?;
    let rate = f64::from(sr);
    let bpm = positive("bpm", bpm)?;
    require(bpm <= 60.0 * rate, "bpm", bpm, "at most 60 · sr")?;
    let w = 2.0 * PI * positive("f0", f0)?;
    let thump: Vec<f64> = (0..n_samples(0.18, sr)?)
        .map(|i| {
            let tt = time(i, rate);
            (w * tt * (1.0 - 0.35 * tt / 0.18)).sin() * (-tt / 0.045).exp()
        })
        .collect();
    let beat = 60.0 / bpm;
    let s2_delay = (beat * 0.4).min(0.30);
    let mut out = vec![0.0; n];
    let mut t = 0.0;
    while t < dur {
        for (offset, amp) in [(0.0, 1.0), (s2_delay, 0.6)] {
            // A start past MAX_SAMPLES is past the end of the buffer too.
            let Ok(i) = n_samples(t + offset, sr) else {
                continue;
            };
            if i + thump.len() < n {
                out[i..i + thump.len()]
                    .iter_mut()
                    .zip(&thump)
                    .for_each(|(o, th)| *o += amp * th);
            }
        }
        t += beat;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_places_lub_and_dub() {
        let y = heartbeat(60.0, 2.0, 1000, 48.0).unwrap();
        assert_eq!(y.len(), 2000);
        // a thump starts with sin(0) = 0 and rises: S1 at 0 s and 1 s, S2 at 0.3 s and 1.3 s
        for start in [0, 300, 1000, 1300] {
            assert_eq!(y[start], 0.0);
            assert!(y[start + 3] > 0.0);
        }
        assert!(heartbeat(0.0, 1.0, 1000, 48.0).is_err());
        assert!(heartbeat(60_001.0, 1.0, 1000, 48.0).is_err());
        assert_eq!(heartbeat(60.0, 1.0, 1000, 0.0).unwrap_err().name(), "f0");
        assert!(heartbeat(60.0, 0.0, 1000, 48.0).unwrap().is_empty());
    }

    #[test]
    fn voice_without_vibrato_is_additive_synthesis() {
        let plain = soft_voice(220.0, 0.05, 8000, 5.0, 0.0).unwrap();
        let tone = signal_core::additive(220.0, 0.05, &VOICE_PARTIALS, 8000, None).unwrap();
        let env = adsr(400, 8000, 0.08, 0.2, 0.8, 0.05 / 3.0).unwrap();
        for ((p, t), e) in plain.iter().zip(&tone).zip(&env) {
            assert!((p - t * e).abs() < 1e-12);
        }
        assert!(soft_voice(-1.0, 0.05, 8000, 5.0, 12.0).is_err());
    }

    #[test]
    fn voice_errors_name_the_argument_that_overflows() {
        let name = |f, hz, c| soft_voice(f, 1.0, 8000, hz, c).unwrap_err().name();
        // freq · 4 overflows just above f64::MAX / 4; below it the phase overflows to NaN, as in lifesound.py.
        let quarter = f64::MAX / 4.0;
        assert_eq!(name(quarter * (1.0 + f64::EPSILON), 5.0, 0.0), "freq");
        assert!(soft_voice(quarter, 0.01, 8000, 5.0, 0.0).unwrap()[1].is_nan());
        // 2π · vibrato_hz overflows, or 2π · vibrato_hz · t does by the last sample.
        assert_eq!(name(220.0, 1e308, 0.0), "vibrato_hz");
        let e = soft_voice(220.0, 2.0, 8000, 2e307, 12.0).unwrap_err();
        assert_eq!(e.name(), "vibrato_hz");
        assert!(soft_voice(220.0, 1.0, 8000, 2e307, 12.0).is_ok());
        // At 220 Hz the 4th harmonic overflows from about 1.217e6 cents, the pitch from 1.219e6.
        for cents in [1.218e6, 2e6, -2e6] {
            assert_eq!(name(220.0, 5.0, cents), "vibrato_cents");
        }
        assert!(soft_voice(220.0, 1.0, 8000, 5.0, 1.2e6).is_ok());
        // With no samples nothing overflows: the signal is empty, as in lifesound.py.
        for (f, hz, c) in [
            (2.0 * quarter, 5.0, 0.0),
            (220.0, 1e308, 0.0),
            (220.0, 5.0, 1e300),
        ] {
            assert!(soft_voice(f, 0.0, 8000, hz, c).unwrap().is_empty());
        }
    }
}
