// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! Property tests: the measured pitch of the Karplus–Strong strings, and input
//! validation for values JSON cannot carry (NaN, infinity).

use std::f64::consts::TAU;
use synthesis_theory as st;

/// The frequency between `lo` and `hi` hertz at which the spectrum of `x` peaks:
/// the magnitude of the Hann-windowed discrete-time Fourier transform, searched on
/// a grid of half-bin steps and then refined by golden-section search to 1e-7 Hz.
///
/// A Hann window times the string's decay is a smooth real envelope, so each
/// partial's peak sits at its frequency; only the faint tails of the other
/// partials can pull it.
fn peak_hz(x: &[f64], sr: f64, lo: f64, hi: f64) -> f64 {
    let n = x.len();
    let last = (n - 1) as f64;
    let w: Vec<f64> = x
        .iter()
        .enumerate()
        .map(|(i, v)| v * (0.5 - 0.5 * (TAU * i as f64 / last).cos()))
        .collect();
    let mag = |f: f64| {
        let (s, c) = (TAU * f / sr).sin_cos();
        let (mut zr, mut zi, mut re, mut im) = (1.0, 0.0, 0.0, 0.0);
        for v in &w {
            re += v * zr;
            im += v * zi;
            (zr, zi) = (zr * c + zi * s, zi * c - zr * s);
        }
        re.hypot(im)
    };
    let step = sr / n as f64 / 2.0;
    let (mut best, mut best_mag, mut f) = (lo, 0.0, lo);
    while f <= hi {
        let m = mag(f);
        if m > best_mag {
            (best, best_mag) = (f, m);
        }
        f += step;
    }
    let r = (5.0_f64.sqrt() - 1.0) / 2.0;
    let (mut a, mut b) = (best - step, best + step);
    while b - a > 1e-7 {
        let (c, d) = (b - r * (b - a), a + r * (b - a));
        if mag(c) > mag(d) {
            b = d;
        } else {
            a = c;
        }
    }
    0.5 * (a + b)
}

/// The measured fundamental of a one-second string near `expected` hertz.
fn fundamental(x: &[f64], sr: u32, expected: f64) -> f64 {
    peak_hz(x, f64::from(sr), 0.97 * expected, 1.03 * expected)
}

fn cents(from: f64, to: f64) -> f64 {
    pitch_math::cents(from, to).unwrap()
}

/// Five pitches an octave apart (110 to 1760 Hz) at 44.1 kHz, and two of them at 48 kHz.
const STRINGS: [(f64, u32); 7] = [
    (110.0, 44_100),
    (220.0, 44_100),
    (440.0, 44_100),
    (880.0, 44_100),
    (1760.0, 44_100),
    (440.0, 48_000),
    (1760.0, 48_000),
];

#[test]
fn karplus_strong_sounds_at_sr_over_n_plus_a_half() {
    for (freq, sr) in STRINGS {
        let x = st::karplus_strong(freq, 1.0, sr, 0.996, 0, Some(2000.0)).unwrap();
        let predicted = st::karplus_strong_pitch(freq, sr).unwrap();
        let measured = fundamental(&x, sr, predicted);
        let off = cents(predicted, measured);
        println!(
            "{freq} Hz at {sr}: predicted {predicted:.4}, measured {measured:.4} Hz ({off:+.5} cents)"
        );
        assert!(
            off.abs() < 0.02,
            "{freq} Hz at {sr}: {off} cents from sr / (N + 1/2)"
        );
        // ... and so it is out of tune by the predicted amount.
        let detune = st::karplus_strong_detune_cents(freq, sr).unwrap();
        assert!((cents(freq, measured) - detune).abs() < 0.02);
    }
    // At A4 and 44.1 kHz: N = 100, 438.806 Hz, 4.70 cents flat.
    let x = st::karplus_strong(440.0, 1.0, 44_100, 0.996, 0, Some(2000.0)).unwrap();
    let measured = fundamental(&x, 44_100, 438.806);
    assert!((measured - 44_100.0 / 100.5).abs() < 0.01);
    assert!((cents(440.0, measured) + 4.704).abs() < 0.02);
}

#[test]
fn tuned_string_is_within_one_cent() {
    let mut worst: f64 = 0.0;
    for (freq, sr) in STRINGS.iter().copied().chain([(261.6255653005986, 44_100)]) {
        let x = st::karplus_strong_tuned(freq, 1.0, sr, 0.996, 0, Some(2000.0)).unwrap();
        let off = cents(freq, fundamental(&x, sr, freq));
        println!("tuned {freq} Hz at {sr}: {off:+.5} cents");
        worst = worst.max(off.abs());
        assert!(off.abs() < 1.0, "tuned {freq} Hz at {sr}: {off} cents");
    }
    println!("worst tuned error: {worst:.5} cents");
    // The claim states the worst case, 0.15 cents (at 1760 Hz and 44.1 kHz).
    assert!(worst < 0.15, "worst tuned error {worst} cents");
}

#[test]
fn short_loops_go_flat_above_the_tested_range() {
    // A loop of a few samples at 44.1 kHz: the loop's losses, mostly the average's
    // cos(ω/2) rather than decay, pull its pole flat of sr / (N + 1/2), and in the
    // tuned string the allpass's delay at the
    // fundamental also drifts from Δ. Each case: the string, the pitch it is meant
    // to sound at, and the range its measured offset from that pitch must lie in.
    let plain = |f| st::karplus_strong(f, 1.0, 44_100, 0.996, 0, Some(2000.0)).unwrap();
    let tuned = |f| st::karplus_strong_tuned(f, 1.0, 44_100, 0.996, 0, Some(2000.0)).unwrap();
    for (name, x, target, lo, hi) in [
        ("tuned 5000 Hz", tuned(5000.0), 5000.0, -3.0, -2.75),
        ("tuned 10000 Hz", tuned(10_000.0), 10_000.0, -17.0, -16.6),
        // sr / (N + 1/2): 44100 / 4.5 and 44100 / 2.5.
        ("plain N = 4", plain(11_025.0), 9800.0, -7.0, -6.6),
        ("plain N = 2", plain(20_000.0), 17_640.0, -112.0, -109.0),
    ] {
        let off = cents(target, peak_hz(&x, 44_100.0, 0.93 * target, target));
        println!("{name} at 44100: {off:+.5} cents");
        assert!((lo..hi).contains(&off), "{name}: {off} cents");
    }
}

#[test]
fn outputs_are_finite_and_normalised() {
    let peak = |x: &[f64]| x.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    for x in [
        st::modal_chime(440.0, 0.2, 1.0, 8000).unwrap(),
        st::karplus_strong(220.0, 0.2, 8000, 1.0, 5, None).unwrap(),
        st::karplus_strong_tuned(220.0, 0.2, 8000, 1.0, 5, None).unwrap(),
    ] {
        assert!(x.iter().all(|v| v.is_finite()));
        assert!(peak(&x) < 1.0 && peak(&x) > 1.0 - 1e-9);
    }
    for x in [
        st::fm_bell(440.0, 0.2, 1.4, 10.0, 8000).unwrap(),
        st::fm_tone(440.0, 616.0, 10.0, 0.2, 8000).unwrap(),
    ] {
        assert!(peak(&x) <= 1.0);
    }
    // Not normalised, as in lifesound.py: at most the sum of the partials (1.36) or of the thumps (1.6).
    assert!(peak(&st::soft_voice(220.0, 0.2, 8000, 5.0, 12.0).unwrap()) <= 1.36);
    assert!(peak(&st::heartbeat(72.0, 2.0, 8000, 48.0).unwrap()) <= 1.6);
}

#[test]
fn nan_and_infinity_are_rejected() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(st::modal_chime(bad, 0.1, 1.0, 8000).is_err());
        assert!(st::modal_chime(440.0, bad, 1.0, 8000).is_err());
        assert!(st::modal_chime(440.0, 0.1, bad, 8000).is_err());
        assert!(st::fm_bell(440.0, 0.1, bad, 10.0, 8000).is_err());
        assert!(st::fm_bell(440.0, 0.1, 1.4, bad, 8000).is_err());
        assert!(st::fm_tone(bad, 1.0, 1.0, 0.1, 8000).is_err());
        assert!(st::fm_tone(1.0, bad, 1.0, 0.1, 8000).is_err());
        assert!(st::fm_tone(1.0, 1.0, bad, 0.1, 8000).is_err());
        assert!(st::fm_spectrum(1.0, 1.0, bad, 2, 0.1, 8000).is_err());
        assert!(st::karplus_strong(bad, 0.1, 8000, 0.9, 0, None).is_err());
        assert!(st::karplus_strong(440.0, 0.1, 8000, bad, 0, None).is_err());
        assert!(st::karplus_strong(440.0, 0.1, 8000, 0.9, 0, Some(bad)).is_err());
        assert!(st::karplus_strong_tuned(bad, 0.1, 8000, 0.9, 0, None).is_err());
        assert!(st::karplus_strong_pitch(bad, 8000).is_err());
        assert!(st::soft_voice(440.0, 0.1, 8000, bad, 12.0).is_err());
        assert!(st::soft_voice(440.0, 0.1, 8000, 5.0, bad).is_err());
        assert!(st::heartbeat(bad, 1.0, 8000, 48.0).is_err());
        assert!(st::heartbeat(60.0, 1.0, 8000, bad).is_err());
    }
}
