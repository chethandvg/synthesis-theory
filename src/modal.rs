// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! Modal synthesis of a struck bar, and the mode ratios of a free–free bar.

use crate::{Result, at_most, finite, normalized, positive, time};
use signal_core::{exp_decay, n_samples};
use std::f64::consts::PI;

/// Mode-frequency ratios of a free–free uniform bar (Euler–Bernoulli beam), the
/// values `lifesound.py` uses: 1 : 2.756 : 5.404 : 8.933. They are the squares of
/// `xₖ / x₁` for the first four positive roots of `cos x · cosh x = 1`
/// ([`free_bar_ratios`]), 1 : 2.75654 : 5.40392 : 8.93295, written to three
/// decimals (5.404 and 8.933 rounded, 2.756 cut off: 2.75654 rounds to 2.757).
pub const BAR_MODES: [f64; 4] = [1.0, 2.756, 5.404, 8.933];

/// The most roots [`free_bar_roots`] computes in one call.
const MAX_BAR_ROOTS: usize = 10_000;

/// A struck bar by modal synthesis, as `lifesound.modal_chime`: one decaying
/// sine per mode in [`BAR_MODES`], mode `k` (from 0) at `freq · BAR_MODES[k]`
/// hertz with amplitude `(0.55 · brightness)ᵏ` and a 60 dB decay time of
/// `dur / (1 + 1.6k)` seconds, plus a short strike transient at `6 · freq`, all
/// scaled to a peak just below 1. `lifesound.py`'s defaults are `dur = 3.0`,
/// `brightness = 1.0` and `sr = 44100`.
///
/// The decays use `signal_core::exp_decay`, which takes ln 1000 where
/// `lifesound.py` took 6.907755.
pub fn modal_chime(freq: f64, dur: f64, brightness: f64, sr: u32) -> Result<Vec<f64>> {
    let n = n_samples(positive("dur", dur)?, sr)?;
    let (freq, brightness) = (positive("freq", freq)?, finite("brightness", brightness)?);
    let rate = f64::from(sr);
    let mut out = vec![0.0; n];
    for (k, &ratio) in BAR_MODES.iter().enumerate() {
        let k = k as f64;
        let amp = 0.55_f64.powf(k) * brightness.powf(k);
        let env = exp_decay(n, dur / (1.0 + 1.6 * k), sr)?;
        let w = 2.0 * PI * freq * ratio;
        for (i, (o, e)) in out.iter_mut().zip(env).enumerate() {
            *o += amp * (w * time(i, rate)).sin() * e;
        }
    }
    let w = 2.0 * PI * freq * 6.0;
    for (i, o) in out.iter_mut().enumerate() {
        let t = time(i, rate);
        *o += (-t / 0.002).exp() * 0.15 * (w * t).sin();
    }
    Ok(normalized(out))
}

/// The root of `cos x · cosh x = 1` between `kπ` and `(k + 1)π`, by bisection on
/// `cos x − 1/cosh x` (the same roots, without the overflow of `cosh`) down to
/// adjacent floating-point numbers; the one with the smaller residual wins.
fn bar_root(k: f64) -> f64 {
    let g = |x: f64| x.cos() - 1.0 / x.cosh();
    let (mut lo, mut hi) = (k * PI, (k + 1.0) * PI);
    let lo_positive = g(lo) > 0.0;
    loop {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        if (g(mid) > 0.0) == lo_positive {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if g(lo).abs() <= g(hi).abs() { lo } else { hi }
}

/// The first `count` positive roots `x₁ < x₂ < …` of `cos x · cosh x = 1`
/// (4.730041, 7.853205, 10.995608, 14.137165, …): the free–free bar's modes
/// have wavenumbers `xₖ / L` for a bar of length `L`. Root `k` is found between
/// `kπ` and `(k + 1)π`; `count` is at most 10 000.
pub fn free_bar_roots(count: usize) -> Result<Vec<f64>> {
    let count = at_most("count", count, MAX_BAR_ROOTS)?;
    Ok((1..=count).map(|k| bar_root(k as f64)).collect())
}

/// The first `count` mode-frequency ratios of a free–free bar, `(xₖ / x₁)²` for
/// the roots of [`free_bar_roots`]: 1, 2.756539, 5.403918, 8.932950, … (a bar's
/// mode frequencies grow with the square of the wavenumber).
pub fn free_bar_ratios(count: usize) -> Result<Vec<f64>> {
    let roots = free_bar_roots(count)?;
    let x1 = roots.first().copied().unwrap_or(1.0);
    Ok(roots.iter().map(|x| (x / x1).powi(2)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_modes_round_the_computed_ratios() {
        let r = free_bar_ratios(4).unwrap();
        for (k, (b, r)) in BAR_MODES.iter().zip(&r).enumerate() {
            assert!((b - r).abs() < 6e-4, "mode {k}: {b} vs {r}");
        }
        assert!(free_bar_roots(0).unwrap().is_empty() && free_bar_ratios(0).unwrap().is_empty());
        assert!(free_bar_roots(MAX_BAR_ROOTS + 1).is_err());
    }

    #[test]
    fn chime_is_normalised() {
        let y = modal_chime(880.0, 0.05, 1.0, 8000).unwrap();
        assert_eq!(y.len(), 400);
        let peak = y.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        assert!(peak < 1.0 && peak > 1.0 - 1e-9);
        assert!(modal_chime(880.0, 0.0, 1.0, 8000).is_err());
    }
}
