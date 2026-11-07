// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! Frequency modulation: Chowning's bell, a constant-index FM tone and its measured spectrum.

use crate::{MAX_SAMPLES, Result, at_most, finite, positive, require, time};
use signal_core::{adsr, exp_decay, n_samples};
use std::f64::consts::{PI, TAU};

/// `sin(wc·t + index · sin(wm·t))`, with `wc` and `wm` in radians per second.
fn fm_sample(wc: f64, wm: f64, index: f64, t: f64) -> f64 {
    (wc * t + index * (wm * t).sin()).sin()
}

/// A constant-index FM tone lasting `dur` seconds: sample `i` is
/// `sin(2π·fc·t + index · sin(2π·fm·t))` at `t = i / sr`.
///
/// By the Jacobi–Anger expansion this is `Σₙ Jₙ(index) · sin(2π(fc + n·fm)t)`: a
/// carrier at `fc` with sidebands at `fc + n·fm` whose amplitudes are Bessel
/// functions of the index ([`fm_spectrum`] measures them). `fc` and `fm` must
/// be greater than 0; `index` may be any finite number.
pub fn fm_tone(fc: f64, fm: f64, index: f64, dur: f64, sr: u32) -> Result<Vec<f64>> {
    let n = n_samples(dur, sr)?;
    let (wc, wm) = (
        2.0 * PI * positive("fc", fc)?,
        2.0 * PI * positive("fm", fm)?,
    );
    let index = finite("index", index)?;
    let rate = f64::from(sr);
    Ok((0..n)
        .map(|i| fm_sample(wc, wm, index, time(i, rate)))
        .collect())
}

/// John Chowning's (1973) FM bell, as `lifesound.fm_bell`:
/// `A(t) · sin(2π·fc·t + I(t) · sin(2π·fc·ratio·t))`, where the loudness `A(t)`
/// falls by 60 dB over `dur` seconds and the index `I(t) = index · A(t)` falls
/// with it (a real bell gets darker as it fades), then a 2 ms attack and 50 ms
/// release. `lifesound.py`'s defaults are `dur = 6.0`, `ratio = 1.4`,
/// `index = 10.0` and `sr = 44100`.
///
/// The decay uses `signal_core::exp_decay` (ln 1000 where `lifesound.py` took
/// 6.907755) and the gate `signal_core::adsr`.
pub fn fm_bell(fc: f64, dur: f64, ratio: f64, index: f64, sr: u32) -> Result<Vec<f64>> {
    let n = n_samples(positive("dur", dur)?, sr)?;
    let fc = positive("fc", fc)?;
    let (ratio, index) = (finite("ratio", ratio)?, finite("index", index)?);
    let env = exp_decay(n, dur, sr)?;
    let gate = adsr(n, sr, 0.002, 0.0, 1.0, 0.05)?;
    let (wc, rate) = (2.0 * PI * fc, f64::from(sr));
    let wm = wc * ratio;
    Ok(env
        .iter()
        .zip(&gate)
        .enumerate()
        .map(|(i, (&a, &g))| a * fm_sample(wc, wm, index * a, time(i, rate)) * g)
        .collect())
}

/// The amplitude of the component of `x` at `f` hertz: `(2/N) · |Σᵢ xᵢ e^{−2πi·f·i/sr}|`.
/// The phase `f·i` is reduced modulo `sr` first, so a whole-hertz `f` has an exact angle.
fn amplitude(x: &[f64], sr: f64, f: f64) -> f64 {
    let (mut re, mut im) = (0.0, 0.0);
    for (i, &v) in x.iter().enumerate() {
        let a = TAU * ((f * i as f64) % sr) / sr;
        re += v * a.cos();
        im -= v * a.sin();
    }
    2.0 * re.hypot(im) / x.len() as f64
}

/// The measured spectrum of [`fm_tone`]`(fc, fm, index, dur, sr)`: for
/// `n = −n_max, …, n_max`, the amplitude of its component at `fc + n·fm` hertz,
/// `(2/N) · |Σᵢ xᵢ e^{−2πi(fc + n·fm)i/sr}|` over its `N` samples (the discrete
/// Fourier transform at that one frequency).
///
/// When every `fc + n·fm` is a whole number of cycles over the tone (whole-hertz
/// frequencies and `dur = 1` second, say) and no two sidebands land on the same
/// frequency, entry `n` is `|Jₙ(index)|`. A sideband below 0 Hz sounds at
/// `|fc + n·fm|` with its sign flipped, and one above `sr/2` aliases, so where
/// two meet their amplitudes add or cancel; at 0 Hz and `sr/2` a sine is zero.
pub fn fm_spectrum(
    fc: f64,
    fm: f64,
    index: f64,
    n_max: usize,
    dur: f64,
    sr: u32,
) -> Result<Vec<f64>> {
    let n_max = at_most("n_max", n_max, MAX_SAMPLES / 2)?;
    let x = fm_tone(fc, fm, index, dur, sr)?;
    require(
        !x.is_empty(),
        "dur",
        dur,
        "long enough for at least one sample",
    )?;
    let rate = f64::from(sr);
    Ok((0..=2 * n_max)
        .map(|j| amplitude(&x, rate, fc + (j as f64 - n_max as f64) * fm))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplitude_of_a_sine_on_a_bin() {
        let x: Vec<f64> = (0..1000)
            .map(|i| 0.5 * (TAU * 50.0 * i as f64 / 1000.0).sin())
            .collect();
        assert!((amplitude(&x, 1000.0, 50.0) - 0.5).abs() < 1e-13);
        assert!(amplitude(&x, 1000.0, 51.0) < 1e-13);
    }

    #[test]
    fn index_zero_is_a_pure_carrier() {
        let s = fm_spectrum(100.0, 30.0, 0.0, 2, 1.0, 1000).unwrap();
        let want = [0.0, 0.0, 1.0, 0.0, 0.0];
        assert!(s.iter().zip(want).all(|(g, w)| (g - w).abs() < 1e-12));
        assert!(fm_spectrum(100.0, 30.0, 1.0, 2, 0.0, 1000).is_err());
        assert_eq!(fm_tone(0.0, 30.0, 1.0, 0.1, 1000).unwrap_err().name(), "fc");
        assert_eq!(
            fm_spectrum(100.0, -30.0, 1.0, 2, 1.0, 1000)
                .unwrap_err()
                .name(),
            "fm"
        );
    }
}
