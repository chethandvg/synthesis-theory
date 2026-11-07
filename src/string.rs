// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! The Karplus–Strong plucked string, the pitch it really sounds at, and a tuned version.

use crate::{MAX_SAMPLES, Result, normalized, positive, rate, require};
use signal_core::{box_lowpass, n_samples, white_noise};

/// The smallest fractional delay [`karplus_strong_tuned`] gives its allpass:
/// the fraction is kept in [0.1, 1.1), away from 0, where the allpass
/// coefficient tends to 1 and its pole to z = −1.
const MIN_FRACTION: f64 = 0.1;

const TOO_LOW: &str = "high enough that one period plus one sample is at most MAX_SAMPLES samples";

/// `decay`, which must be in [0, 1] so that the string never grows.
fn gain(decay: f64) -> Result<f64> {
    require((0.0..=1.0).contains(&decay), "decay", decay, "in [0, 1]")
}

/// The loop delay of `lifesound.karplus_strong`: `max(round(sr / freq), 2)`
/// samples, with halves rounded to even as Python's `round` does. The burst is
/// one sample longer than the loop, so the loop must be below `MAX_SAMPLES`.
fn period(freq: f64, sr: f64) -> Result<usize> {
    let p = (sr / positive("freq", freq)?).round_ties_even().max(2.0);
    require(p < MAX_SAMPLES as f64, "freq", freq, TOO_LOW)?;
    Ok(p as usize)
}

/// A string with loop delay `p`: the first `p + 1` samples are the burst
/// (uniform noise from `SplitMix64::new(seed)`, low-passed by two passes of
/// `signal_core::box_lowpass` when `brightness` is given, minus its mean), and
/// each later sample is `step(out, i)`. The first `n` samples, scaled to a peak
/// just below 1, are returned.
fn pluck(
    n: usize,
    p: usize,
    sr: u32,
    seed: u64,
    brightness: Option<f64>,
    mut step: impl FnMut(&[f64], usize) -> f64,
) -> Result<Vec<f64>> {
    let mut out = white_noise(p + 1, seed)?;
    if let Some(cutoff) = brightness {
        out = box_lowpass(&out, 1, positive("pluck_brightness_hz", cutoff)?, sr, 2)?;
    }
    let mean = out.iter().sum::<f64>() / out.len() as f64;
    out.iter_mut().for_each(|v| *v -= mean);
    out.reserve(n.saturating_sub(out.len()));
    for i in p + 1..n {
        let v = step(&out, i);
        out.push(v);
    }
    out.truncate(n);
    Ok(normalized(out))
}

/// The Karplus–Strong (1983) plucked string of `lifesound.karplus_strong`: a
/// burst of `N + 1` noise samples, then `yᵢ = decay · 0.5 · (yᵢ₋ₙ + yᵢ₋ₙ₋₁)`
/// with `N = max(round(sr / freq), 2)`, scaled to a peak just below 1.
/// `pluck_brightness_hz` low-passes the burst for a softer pluck (`None` leaves
/// it bright). `lifesound.py`'s defaults are `sr = 44100`, `decay = 0.996`,
/// `seed = 0` and `pluck_brightness_hz = 2000`.
///
/// The noise is `signal_core`'s SplitMix64, not numpy's generator, so the samples
/// differ from the old toolkit's; the algorithm is the same. The two-point
/// average delays the loop by half a sample more than `N`, so the string sounds
/// at [`karplus_strong_pitch`] = `sr / (N + 1/2)`, not at `freq`: checked from
/// 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz (within
/// 0.02 cents). Loops of a few samples sound flatter still (see
/// [`karplus_strong_pitch`]).
pub fn karplus_strong(
    freq: f64,
    dur: f64,
    sr: u32,
    decay: f64,
    seed: u64,
    pluck_brightness_hz: Option<f64>,
) -> Result<Vec<f64>> {
    let n = n_samples(dur, sr)?;
    let p = period(freq, f64::from(sr))?;
    let k = gain(decay)? * 0.5;
    pluck(n, p, sr, seed, pluck_brightness_hz, |y, i| {
        k * (y[i - p] + y[i - p - 1])
    })
}

/// The frequency [`karplus_strong`] really sounds at: `sr / (N + 1/2)` hertz,
/// with `N = max(round(sr / freq), 2)`. The loop is `N` samples of delay plus the
/// two-point average, which delays every frequency by exactly half a sample
/// (`synth:TWO-POINT-AVERAGE-DELAY`), so the loop resonates at multiples of
/// `sr / (N + 1/2)` (`synth:KS-PITCH`). At 440 Hz and 44.1 kHz, `N = 100` and
/// the string sounds at 438.806 Hz.
///
/// The rendered string's fundamental is checked against this from 110 to
/// 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz (within 0.02 cents).
/// Loops of a few samples sound flatter still, because the loop's losses, mostly
/// the average's `cos(ω/2)` rather than `decay`, pull its fundamental flat: with
/// `decay = 0.996`, by 6.8 cents at `N = 4` and about 110 cents at `N = 2`.
pub fn karplus_strong_pitch(freq: f64, sr: u32) -> Result<f64> {
    let sr = rate(sr)?;
    Ok(sr / (period(freq, sr)? as f64 + 0.5))
}

/// How far [`karplus_strong`] is out of tune, in cents: `pitch_math::cents` from
/// `freq` to [`karplus_strong_pitch`]. At 44.1 kHz: −4.70 cents at 440 Hz,
/// −13.30 at 880 Hz, −30.35 at 1760 Hz (rounding `N` can also make it sharp
/// before the half sample pulls it flat). It holds where
/// [`karplus_strong_pitch`] does: checked from 110 to 1760 Hz at 44.1 kHz and at
/// 440 and 1760 Hz at 48 kHz (within 0.02 cents), while loops of a few samples
/// sound flatter still.
pub fn karplus_strong_detune_cents(freq: f64, sr: u32) -> Result<f64> {
    Ok(pitch_math::cents(freq, karplus_strong_pitch(freq, sr)?)?)
}

/// A Karplus–Strong string tuned to `freq` (Jaffe and Smith 1983): the loop of
/// [`karplus_strong`] with a first-order allpass filter that adds the missing
/// fraction of a sample. The loop must be `D = sr / freq` samples long; with
/// `N = ⌊D − 0.6⌋` samples of delay and half a sample from the average, the
/// allpass `(C + z⁻¹)/(1 + C·z⁻¹)` with `C = (1 − Δ)/(1 + Δ)` supplies the rest,
/// `Δ = D − N − 1/2` (in [0.1, 1.1); exactly `Δ` at 0 Hz, nearly so at the
/// partials). Each sample after the burst is
/// `vᵢ = decay · 0.5 · (yᵢ₋ₙ + yᵢ₋ₙ₋₁)`, `yᵢ = C·vᵢ + vᵢ₋₁ − C·yᵢ₋₁`, with the
/// allpass starting at rest. `freq` must be at most `sr / 2`.
///
/// The tuning is checked from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz
/// at 48 kHz (within 0.15 cents). Higher strings, only a few samples long, go
/// flat again as the allpass's delay at the fundamental drifts from `Δ` and the
/// loop's losses, mostly the average's `cos(ω/2)` rather than `decay`, pull its
/// pole flat: at 44.1 kHz the string measures 2.9 cents flat at 5 kHz and
/// 16.8 cents flat at 10 kHz.
pub fn karplus_strong_tuned(
    freq: f64,
    dur: f64,
    sr: u32,
    decay: f64,
    seed: u64,
    pluck_brightness_hz: Option<f64>,
) -> Result<Vec<f64>> {
    let n = n_samples(dur, sr)?;
    let rate = f64::from(sr);
    let freq = positive("freq", freq)?;
    require(freq <= rate / 2.0, "freq", freq, "at most sr / 2")?;
    let d = rate / freq;
    let p = (d - 0.5 - MIN_FRACTION).floor();
    require(p < MAX_SAMPLES as f64, "freq", freq, TOO_LOW)?;
    let delta = d - 0.5 - p;
    let c = (1.0 - delta) / (1.0 + delta);
    let (p, k) = (p as usize, gain(decay)? * 0.5);
    let (mut v1, mut y1) = (0.0, 0.0);
    pluck(n, p, sr, seed, pluck_brightness_hz, |y, i| {
        let v = k * (y[i - p] + y[i - p - 1]);
        let out = c * v + v1 - c * y1;
        (v1, y1) = (v, out);
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_follows_lifesound() {
        assert_eq!(period(440.0, 44_100.0).unwrap(), 100); // round(100.227)
        assert_eq!(period(1.0, 0.3).unwrap(), 2); // round(0.3) = 0, clamped at 2
        assert_eq!(period(1.0, 2.5).unwrap(), 2); // round(2.5) = 2, halves to even
        assert_eq!(period(1.0, 3.5).unwrap(), 4);
        assert!(period(1e-12, 44_100.0).is_err());
        // The burst is p + 1 samples, so a loop of MAX_SAMPLES is too long, and the error names freq.
        let max = MAX_SAMPLES as f64;
        assert_eq!(period(1.0, max - 1.0).unwrap(), MAX_SAMPLES - 1);
        assert_eq!(period(1.0, max).unwrap_err().name(), "freq");
        let sr = u32::try_from(MAX_SAMPLES).unwrap();
        let e = karplus_strong(1.0, 0.0, sr, 0.9, 0, None).unwrap_err();
        assert_eq!(e.name(), "freq");
        // sr / freq = MAX_SAMPLES + 1.1 gives the tuned loop N = MAX_SAMPLES.
        let f = max / (max + 1.1);
        let e = karplus_strong_tuned(f, 0.0, sr, 0.9, 0, None).unwrap_err();
        assert_eq!(e.name(), "freq");
    }

    #[test]
    fn pitch_is_half_a_sample_flat() {
        assert_eq!(
            karplus_strong_pitch(440.0, 44_100).unwrap(),
            44_100.0 / 100.5
        );
        assert!((karplus_strong_detune_cents(440.0, 44_100).unwrap() + 4.7044).abs() < 1e-4);
        assert!(karplus_strong_pitch(440.0, 0).is_err());
    }

    #[test]
    fn short_strings_are_the_burst() {
        let y = karplus_strong(4000.0, 0.0005, 8000, 0.996, 1, None).unwrap();
        assert_eq!(y.len(), 4); // period 2: a 3-sample burst, then one loop sample
        assert!(karplus_strong(440.0, 0.01, 8000, 1.5, 0, None).is_err());
        assert!(karplus_strong(440.0, 0.01, 8000, 0.9, 0, Some(0.0)).is_err());
        assert!(karplus_strong_tuned(4001.0, 0.01, 8000, 0.9, 0, None).is_err());
        assert_eq!(
            karplus_strong_tuned(4000.0, 0.01, 8000, 0.9, 0, None)
                .unwrap()
                .len(),
            80
        );
    }
}
