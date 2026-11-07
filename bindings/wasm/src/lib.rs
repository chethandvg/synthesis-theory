// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! WebAssembly bindings for `synthesis-theory`. Every function keeps the Rust name, so
//! the golden vectors can call it by name; invalid input throws a JS `Error`.
//! Signals come back as `Float64Array`. Sample rates, seeds and counts are
//! ordinary JS numbers that must be whole, so JavaScript never sees a `BigInt`;
//! `pluck_brightness_hz` may be `null` or `undefined` for an unfiltered pluck.

use synthesis_theory_rs as imp;
use wasm_bindgen::prelude::*;

/// The largest whole number a JS number holds exactly, 2⁵³ − 1.
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// `x` as a whole number of type `T`, or an error naming the parameter.
fn whole<T: TryFrom<u64>>(name: &str, x: f64) -> Result<T, JsError> {
    if (0.0..=MAX_SAFE_INTEGER).contains(&x)
        && x.fract() == 0.0
        && let Ok(v) = T::try_from(x as u64)
    {
        return Ok(v);
    }
    Err(JsError::new(&format!(
        "{name} must be a whole number in range, got {x}"
    )))
}

/// The crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// A struck bar by modal synthesis: four decaying free-bar modes and a strike, peak just below 1.
#[wasm_bindgen]
pub fn modal_chime(freq: f64, dur: f64, brightness: f64, sr: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::modal_chime(freq, dur, brightness, whole("sr", sr)?)?)
}

/// Chowning's FM bell: A(t)·sin(2π·fc·t + index·A(t)·sin(2π·fc·ratio·t)), A falling 60 dB over `dur`.
#[wasm_bindgen]
pub fn fm_bell(fc: f64, dur: f64, ratio: f64, index: f64, sr: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::fm_bell(fc, dur, ratio, index, whole("sr", sr)?)?)
}

/// Constant-index FM: sin(2π·fc·t + index·sin(2π·fm·t)).
#[wasm_bindgen]
pub fn fm_tone(fc: f64, fm: f64, index: f64, dur: f64, sr: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::fm_tone(fc, fm, index, dur, whole("sr", sr)?)?)
}

/// Measured amplitudes of `fm_tone` at fc + n·fm for n = −n_max … n_max.
#[wasm_bindgen]
pub fn fm_spectrum(
    fc: f64,
    fm: f64,
    index: f64,
    n_max: f64,
    dur: f64,
    sr: f64,
) -> Result<Vec<f64>, JsError> {
    let (n_max, sr) = (whole("n_max", n_max)?, whole("sr", sr)?);
    Ok(imp::fm_spectrum(fc, fm, index, n_max, dur, sr)?)
}

/// The Karplus–Strong plucked string of `lifesound.py` (SplitMix64 noise), peak just below 1.
#[wasm_bindgen]
pub fn karplus_strong(
    freq: f64,
    dur: f64,
    sr: f64,
    decay: f64,
    seed: f64,
    pluck_brightness_hz: Option<f64>,
) -> Result<Vec<f64>, JsError> {
    let (sr, seed) = (whole("sr", sr)?, whole("seed", seed)?);
    Ok(imp::karplus_strong(
        freq,
        dur,
        sr,
        decay,
        seed,
        pluck_brightness_hz,
    )?)
}

/// A Karplus–Strong string tuned to `freq` with a first-order allpass (Jaffe and Smith 1983).
///
/// The tuning is checked from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz
/// (within 0.15 cents); higher strings go flat again, 2.9 cents at 5 kHz and 16.8 cents at
/// 10 kHz at 44.1 kHz.
#[wasm_bindgen]
pub fn karplus_strong_tuned(
    freq: f64,
    dur: f64,
    sr: f64,
    decay: f64,
    seed: f64,
    pluck_brightness_hz: Option<f64>,
) -> Result<Vec<f64>, JsError> {
    let (sr, seed) = (whole("sr", sr)?, whole("seed", seed)?);
    Ok(imp::karplus_strong_tuned(
        freq,
        dur,
        sr,
        decay,
        seed,
        pluck_brightness_hz,
    )?)
}

/// The pitch `karplus_strong` really sounds at: sr / (N + 1/2) Hz.
///
/// Checked from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz (within
/// 0.02 cents). Loops of a few samples sound flatter still, because the loop's losses,
/// mostly the average's cos(ω/2) rather than decay, pull its fundamental flat: with decay
/// 0.996, by 6.8 cents at N = 4 and about 110 cents at N = 2.
#[wasm_bindgen]
pub fn karplus_strong_pitch(freq: f64, sr: f64) -> Result<f64, JsError> {
    Ok(imp::karplus_strong_pitch(freq, whole("sr", sr)?)?)
}

/// How far `karplus_strong` is out of tune, in cents (negative: flat).
///
/// It holds where `karplus_strong_pitch` does (checked from 110 to 1760 Hz); loops of a few
/// samples sound flatter still.
#[wasm_bindgen]
pub fn karplus_strong_detune_cents(freq: f64, sr: f64) -> Result<f64, JsError> {
    Ok(imp::karplus_strong_detune_cents(freq, whole("sr", sr)?)?)
}

/// A gentle voice: four harmonics with vibrato, and an envelope.
#[wasm_bindgen]
pub fn soft_voice(
    freq: f64,
    dur: f64,
    sr: f64,
    vibrato_hz: f64,
    vibrato_cents: f64,
) -> Result<Vec<f64>, JsError> {
    let sr = whole("sr", sr)?;
    Ok(imp::soft_voice(freq, dur, sr, vibrato_hz, vibrato_cents)?)
}

/// A lub-dub heartbeat at `bpm` beats per minute: two damped thumps per beat.
#[wasm_bindgen]
pub fn heartbeat(bpm: f64, dur: f64, sr: f64, f0: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::heartbeat(bpm, dur, whole("sr", sr)?, f0)?)
}

/// The first `count` positive roots of cos x · cosh x = 1.
#[wasm_bindgen]
pub fn free_bar_roots(count: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::free_bar_roots(whole("count", count)?)?)
}

/// The first `count` free-bar mode-frequency ratios (x_k / x_1)².
#[wasm_bindgen]
pub fn free_bar_ratios(count: f64) -> Result<Vec<f64>, JsError> {
    Ok(imp::free_bar_ratios(whole("count", count)?)?)
}
