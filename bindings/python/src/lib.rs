// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama

//! Python bindings for `synthesis-theory`. Every function keeps the Rust name and
//! argument order, so the golden vectors can call it by name, and the
//! instruments take `lifesound.py`'s argument names and defaults (`sr` defaults
//! to `SR`, 44100). Signals come back as float64 numpy arrays; invalid input
//! raises `ValueError` (`OverflowError` for a negative or too-large integer
//! argument such as a count, sample rate or seed, `TypeError` for a non-integer one).

use pyo3::prelude::*;

#[pymodule]
mod synthesis_theory {
    use numpy::{IntoPyArray, PyArray1};
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use synthesis_theory_rs as imp;

    /// A signal result.
    type Array<'py> = Bound<'py, PyArray1<f64>>;

    fn ok<T>(r: imp::Result<T>) -> PyResult<T> {
        r.map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn array<'py>(py: Python<'py>, r: imp::Result<Vec<f64>>) -> PyResult<Array<'py>> {
        Ok(ok(r)?.into_pyarray(py))
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        m.add("SR", imp::SR)?;
        m.add("MAX_SAMPLES", imp::MAX_SAMPLES)?;
        m.add("BAR_MODES", imp::BAR_MODES.to_vec())
    }

    /// A struck bar by modal synthesis: four decaying free-bar modes and a strike, peak just below 1.
    #[pyfunction]
    #[pyo3(signature = (freq, dur = 3.0, brightness = 1.0, sr = imp::SR))]
    fn modal_chime(
        py: Python<'_>,
        freq: f64,
        dur: f64,
        brightness: f64,
        sr: u32,
    ) -> PyResult<Array<'_>> {
        array(py, imp::modal_chime(freq, dur, brightness, sr))
    }

    /// Chowning's FM bell: A(t) sin(2 pi fc t + index A(t) sin(2 pi fc ratio t)), A falling 60 dB over dur.
    #[pyfunction]
    #[pyo3(signature = (fc, dur = 6.0, ratio = 1.4, index = 10.0, sr = imp::SR))]
    fn fm_bell(
        py: Python<'_>,
        fc: f64,
        dur: f64,
        ratio: f64,
        index: f64,
        sr: u32,
    ) -> PyResult<Array<'_>> {
        array(py, imp::fm_bell(fc, dur, ratio, index, sr))
    }

    /// Constant-index FM: sin(2 pi fc t + index sin(2 pi fm t)).
    #[pyfunction]
    #[pyo3(signature = (fc, fm, index, dur, sr = imp::SR))]
    fn fm_tone(
        py: Python<'_>,
        fc: f64,
        fm: f64,
        index: f64,
        dur: f64,
        sr: u32,
    ) -> PyResult<Array<'_>> {
        array(py, imp::fm_tone(fc, fm, index, dur, sr))
    }

    /// Measured amplitudes of fm_tone at fc + n fm, n = -n_max..n_max (|J_n(index)| on whole-hertz bins).
    #[pyfunction]
    #[pyo3(signature = (fc, fm, index, n_max, dur = 1.0, sr = imp::SR))]
    fn fm_spectrum(
        py: Python<'_>,
        fc: f64,
        fm: f64,
        index: f64,
        n_max: usize,
        dur: f64,
        sr: u32,
    ) -> PyResult<Array<'_>> {
        array(py, imp::fm_spectrum(fc, fm, index, n_max, dur, sr))
    }

    /// The Karplus-Strong plucked string of lifesound.py (SplitMix64 noise), peak just below 1.
    #[pyfunction]
    #[pyo3(signature = (freq, dur, sr = imp::SR, decay = 0.996, seed = 0, pluck_brightness_hz = Some(2000.0)))]
    fn karplus_strong(
        py: Python<'_>,
        freq: f64,
        dur: f64,
        sr: u32,
        decay: f64,
        seed: u64,
        pluck_brightness_hz: Option<f64>,
    ) -> PyResult<Array<'_>> {
        let x = imp::karplus_strong(freq, dur, sr, decay, seed, pluck_brightness_hz);
        array(py, x)
    }

    /// A Karplus-Strong string tuned to freq with a first-order allpass (Jaffe and Smith 1983).
    ///
    /// The tuning is checked from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz
    /// (within 0.15 cents); higher strings go flat again, 2.9 cents at 5 kHz and 16.8 cents at
    /// 10 kHz at 44.1 kHz.
    #[pyfunction]
    #[pyo3(signature = (freq, dur, sr = imp::SR, decay = 0.996, seed = 0, pluck_brightness_hz = Some(2000.0)))]
    fn karplus_strong_tuned(
        py: Python<'_>,
        freq: f64,
        dur: f64,
        sr: u32,
        decay: f64,
        seed: u64,
        pluck_brightness_hz: Option<f64>,
    ) -> PyResult<Array<'_>> {
        let x = imp::karplus_strong_tuned(freq, dur, sr, decay, seed, pluck_brightness_hz);
        array(py, x)
    }

    /// The pitch karplus_strong really sounds at: sr / (N + 1/2) Hz, N = max(round(sr / freq), 2).
    ///
    /// Checked from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz (within
    /// 0.02 cents). Loops of a few samples sound flatter still, because the loop's losses,
    /// mostly the average's cos(ω/2) rather than decay, pull its fundamental flat: with decay
    /// 0.996, by 6.8 cents at N = 4 and about 110 cents at N = 2.
    #[pyfunction]
    #[pyo3(signature = (freq, sr = imp::SR))]
    fn karplus_strong_pitch(freq: f64, sr: u32) -> PyResult<f64> {
        ok(imp::karplus_strong_pitch(freq, sr))
    }

    /// How far karplus_strong is out of tune, in cents (negative: flat).
    ///
    /// It holds where karplus_strong_pitch does (checked from 110 to 1760 Hz); loops of a few
    /// samples sound flatter still.
    #[pyfunction]
    #[pyo3(signature = (freq, sr = imp::SR))]
    fn karplus_strong_detune_cents(freq: f64, sr: u32) -> PyResult<f64> {
        ok(imp::karplus_strong_detune_cents(freq, sr))
    }

    /// A gentle voice: four harmonics with vibrato_cents of vibrato at vibrato_hz, and an envelope.
    #[pyfunction]
    #[pyo3(signature = (freq, dur, sr = imp::SR, vibrato_hz = 5.0, vibrato_cents = 12.0))]
    fn soft_voice(
        py: Python<'_>,
        freq: f64,
        dur: f64,
        sr: u32,
        vibrato_hz: f64,
        vibrato_cents: f64,
    ) -> PyResult<Array<'_>> {
        array(
            py,
            imp::soft_voice(freq, dur, sr, vibrato_hz, vibrato_cents),
        )
    }

    /// A lub-dub heartbeat at bpm beats per minute: two damped thumps per beat.
    #[pyfunction]
    #[pyo3(signature = (bpm, dur, sr = imp::SR, f0 = 48.0))]
    fn heartbeat(py: Python<'_>, bpm: f64, dur: f64, sr: u32, f0: f64) -> PyResult<Array<'_>> {
        array(py, imp::heartbeat(bpm, dur, sr, f0))
    }

    /// The first `count` positive roots of cos x cosh x = 1.
    #[pyfunction]
    fn free_bar_roots(py: Python<'_>, count: usize) -> PyResult<Array<'_>> {
        array(py, imp::free_bar_roots(count))
    }

    /// The first `count` free-bar mode-frequency ratios (x_k / x_1)^2.
    #[pyfunction]
    fn free_bar_ratios(py: Python<'_>, count: usize) -> PyResult<Array<'_>> {
        array(py, imp::free_bar_ratios(count))
    }
}
