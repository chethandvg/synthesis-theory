# Changelog

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-07

### Added

- A Rust port of the instruments of the earlier `lifesound.py` toolkit, built on signal-core (sample counts, envelopes, the moving-average filter, SplitMix64 noise) and pitch-math (cents): `modal_chime` with the free-bar ratios `BAR_MODES`, Chowning's `fm_bell`, the Karplus–Strong `karplus_strong`, `soft_voice` and `heartbeat`, with `lifesound.py`'s defaults.
- The pitch a Karplus–Strong string really sounds at, `karplus_strong_pitch` (`sr / (N + 1/2)`) and `karplus_strong_detune_cents`, and `karplus_strong_tuned`, which adds a first-order allpass fractional delay (Jaffe and Smith 1983) and measures within 1 cent of the requested pitch from 110 to 1760 Hz (higher strings go flat again: 2.9 cents at 5 kHz, 16.8 cents at 10 kHz, at 44.1 kHz).
- Constant-index FM, `fm_tone`, and `fm_spectrum`, which measures its sidebands by the discrete Fourier transform.
- The free–free bar's roots of `cos x · cosh x = 1`, `free_bar_roots`, and its mode ratios, `free_bar_ratios`.
- `SynthError` for invalid input.
- Python (PyO3, results as numpy arrays, `lifesound.py`'s argument names and defaults) and WebAssembly (`Float64Array`) bindings with the same function names.
- Lean 4 proofs of five P-level claims: the sum-to-product identity and the two-tone beat, the beat rate `|f₁ − f₂|`, the half-sample delay of the two-point average, the Karplus–Strong resonances at `2πk/(N + 1/2)`, and the faster decay of higher partials.
- Two V-level claims: the free-bar mode ratios certified by interval arithmetic in the oracle, and the FM sidebands checked against Bessel functions from mpmath. One R claim on binaural beats, with three cited sources.
- Golden vectors (78 cases) shared by Rust, Python and WebAssembly: 47 whose expected outputs come from the oracle (`lifesound.py`'s code with SplitMix64 noise and ln 1000 in `exp_decay`, exact fractions, mpmath Bessel functions and certified intervals) and 31 invalid inputs, from the crate's own input rules, that must be rejected; and property tests that measure the pitch of rendered strings.
