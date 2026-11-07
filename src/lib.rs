// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! # Synthesis theory
//!
//! Beats, FM sidebands, free-bar mode ratios and Karplus-Strong strings in Rust, Python and WebAssembly, proved in Lean 4 or verified numerically.
//!
//! Part of [Frequencies of Life](https://chethandvg.github.io/frequencies-of-life/). Each public function corresponds to
//! claims in `claims.yaml`; the README lists what is proved and how.
//!
//! This is a port of the instruments of the project's earlier Python toolkit,
//! `lifesound.py` (kept as the golden-vector oracle): a struck bar
//! ([`modal_chime`]), Chowning's FM bell ([`fm_bell`]), the Karplus–Strong
//! plucked string ([`karplus_strong`]), a soft voice ([`soft_voice`]) and a
//! heartbeat ([`heartbeat`]), with the arguments and defaults of `lifesound.py`.
//! Around them are the pieces of theory they rest on: the pitch a Karplus–Strong
//! string really sounds at ([`karplus_strong_pitch`]) and a tuned string that
//! fixes it ([`karplus_strong_tuned`]), constant-index FM and its measured
//! sidebands ([`fm_tone`], [`fm_spectrum`]), and the free-bar mode ratios
//! ([`free_bar_roots`], [`free_bar_ratios`]).
//!
//! Oscillator-free building blocks come from `signal-core` (sample counts,
//! envelopes, the moving-average filter and the SplitMix64 noise generator) and
//! cents from `pitch-math`, so the same arguments give the same samples in Rust,
//! Python and JavaScript.
//!
//! ```
//! use synthesis_theory::{karplus_strong, karplus_strong_detune_cents, karplus_strong_pitch, SR};
//!
//! // lifesound.py's string at A4 rounds its loop to 100 samples, then the two-point
//! // average adds half a sample: it sounds at 44100 / 100.5 Hz, 4.7 cents flat.
//! let string = karplus_strong(440.0, 1.0, SR, 0.996, 0, Some(2000.0))?;
//! assert_eq!(string.len(), 44_100);
//! assert_eq!(karplus_strong_pitch(440.0, SR)?, 44_100.0 / 100.5);
//! assert!((karplus_strong_detune_cents(440.0, SR)? + 4.704).abs() < 1e-3);
//! # Ok::<(), synthesis_theory::SynthError>(())
//! ```
//!
//! Invalid arguments are a [`SynthError`], never a panic: a sample rate of 0, a
//! tone frequency (`freq`, `fc`, `fm`, `f0`), pluck brightness or `bpm` of 0 or
//! below, a negative duration, NaN or an infinity, a decay outside [0, 1]. The
//! shape parameters `brightness`, `ratio`, `index`, `vibrato_hz` and
//! `vibrato_cents` may be any finite number, as in `lifesound.py` (0 turns the
//! effect off; a negative value flips a sign). A frequency so large that
//! `2π·f·t` overflows (near `f64::MAX`) gives NaN samples, not an error, as it
//! did in `lifesound.py`. The one exception is [`soft_voice`], which also
//! rejects, where `lifesound.py` gave NaN samples (or a 0 Hz pitch), a `freq`
//! above `f64::MAX / 4` (its 4th harmonic would overflow), a `vibrato_hz` so
//! large that `2π·vibrato_hz·t` overflows by the last sample, and a
//! `vibrato_cents` so large in size (about 1.2e6 at 220 Hz) that a harmonic of
//! the pitch overflows or the pitch underflows to 0 Hz, though none of the three
//! when the duration is too short for a sample (the signal is then empty, as in
//! `lifesound.py`).

use std::fmt;

mod fm;
mod modal;
mod string;
mod voice;

pub use fm::{fm_bell, fm_spectrum, fm_tone};
pub use modal::{BAR_MODES, free_bar_ratios, free_bar_roots, modal_chime};
pub use signal_core::{MAX_SAMPLES, SR};
pub use string::{
    karplus_strong, karplus_strong_detune_cents, karplus_strong_pitch, karplus_strong_tuned,
};
pub use voice::{heartbeat, soft_voice};

/// An argument outside the range the function accepts.
#[derive(Debug, Clone, PartialEq)]
pub struct SynthError {
    name: &'static str,
    value: f64,
    message: String,
}

impl SynthError {
    fn new(name: &'static str, value: f64, requirement: &str) -> Self {
        Self {
            name,
            value,
            message: format!("{name} must be {requirement}, got {value}"),
        }
    }

    /// The parameter that was out of range.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The offending value (for a count, the number as an `f64`).
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for SynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SynthError {}

impl From<signal_core::SignalError> for SynthError {
    fn from(e: signal_core::SignalError) -> Self {
        Self {
            name: e.name(),
            value: e.value(),
            message: e.to_string(),
        }
    }
}

impl From<pitch_math::PitchError> for SynthError {
    fn from(e: pitch_math::PitchError) -> Self {
        Self {
            name: e.name(),
            value: e.value(),
            message: e.to_string(),
        }
    }
}

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, SynthError>;

fn require(ok: bool, name: &'static str, value: f64, requirement: &str) -> Result<f64> {
    if ok {
        Ok(value)
    } else {
        Err(SynthError::new(name, value, requirement))
    }
}

fn finite(name: &'static str, x: f64) -> Result<f64> {
    require(x.is_finite(), name, x, "a finite number")
}

fn positive(name: &'static str, x: f64) -> Result<f64> {
    require(
        x.is_finite() && x > 0.0,
        name,
        x,
        "a finite number greater than 0",
    )
}

/// The sample rate as an `f64`; it must be at least 1, as in `signal-core`.
fn rate(sr: u32) -> Result<f64> {
    require(sr > 0, "sr", 0.0, "at least 1 sample per second").map(|_| f64::from(sr))
}

/// A count of at most `limit`.
fn at_most(name: &'static str, n: usize, limit: usize) -> Result<usize> {
    require(n <= limit, name, n as f64, &format!("at most {limit}")).map(|_| n)
}

/// `x / (max |x| + 1e-12)`: `lifesound.py`'s normalisation to a peak just below 1.
fn normalized(mut x: Vec<f64>) -> Vec<f64> {
    let scale = x.iter().fold(0.0_f64, |m, v| m.max(v.abs())) + 1e-12;
    x.iter_mut().for_each(|v| *v /= scale);
    x
}

/// The time of sample `i` in seconds, `i / sr`, as `numpy.arange(n) / sr` computes it.
fn time(i: usize, sr: f64) -> f64 {
    i as f64 / sr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_name_the_argument_and_keep_upstream_messages() {
        let e = positive("freq", -1.0).unwrap_err();
        assert_eq!((e.name(), e.value()), ("freq", -1.0));
        assert_eq!(
            e.to_string(),
            "freq must be a finite number greater than 0, got -1"
        );
        let e: SynthError = signal_core::n_samples(1.0, 0).unwrap_err().into();
        assert_eq!(e.name(), "sr");
        assert_eq!(
            e.to_string(),
            "sr must be at least 1 sample per second, got 0"
        );
        let e: SynthError = pitch_math::cents(0.0, 1.0).unwrap_err().into();
        assert_eq!(e.name(), "from_hz");
        assert_eq!(
            at_most("count", 5, 4).unwrap_err().to_string(),
            "count must be at most 4, got 5"
        );
    }

    #[test]
    fn normalized_peaks_just_below_one() {
        let y = normalized(vec![0.5, -2.0, 1.0]);
        assert!((y[1] + 1.0).abs() < 1e-12 && y[1] > -1.0);
        assert_eq!(normalized(vec![0.0; 3]), [0.0; 3]);
    }
}
