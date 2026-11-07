# Synthesis theory

**Beats, FM sidebands, free-bar mode ratios and Karplus-Strong strings in Rust, Python and WebAssembly, proved in Lean 4 or verified numerically.**

[![CI](https://github.com/chethandvg/synthesis-theory/actions/workflows/ci.yml/badge.svg)](https://github.com/chethandvg/synthesis-theory/actions/workflows/ci.yml)
[![Proofs](https://github.com/chethandvg/synthesis-theory/actions/workflows/proofs.yml/badge.svg)](https://github.com/chethandvg/synthesis-theory/actions/workflows/proofs.yml)
[![Licence](https://img.shields.io/badge/licence-Apache--2.0-blue)](LICENSE)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.24044021.svg)](https://doi.org/10.5281/zenodo.24044021)

Part of **[Frequencies of Life](https://chethandvg.github.io/frequencies-of-life/)**, open code for the rhythms of life in which every claim says how it is known: proved, verified, tested, referenced or assumed.

## Claims and evidence

<!-- claims-table:start -->
| Claim | Level | Statement | Evidence |
|---|---|---|---|
| `synth:BEATS` | P | sin a + sin b = 2·sin((a + b)/2)·cos((a − b)/2) for all real a, b (sin_add_sin), so two tones of frequencies f₁ and f₂ add up to 2·cos(π(f₁ − f₂)t)·sin(π(f₁ + f₂)t): a tone at the mean frequency whose size is the envelope \|2·cos(π(f₁ − f₂)t)\|. | [`SynthesisTheory.beats`](proofs/SynthesisTheory/Beats.lean) |
| `synth:BEAT-RATE` | P | For f₁ ≠ f₂ the beat envelope \|2·cos(π(f₁ − f₂)t)\| repeats every 1/\|f₁ − f₂\| seconds and is zero exactly at t = (k + 1/2)/(f₁ − f₂) for integers k, once per period: two tones beat \|f₁ − f₂\| times a second. | [`SynthesisTheory.beat_rate`](proofs/SynthesisTheory/Beats.lean) |
| `synth:BINAURAL-BEATS` | R | When each ear hears a slightly different tone, the brain produces a beat at the difference frequency although the two waves never add physically (a binaural beat). It is generated in the auditory pathway, needs carriers up to about 1 kHz (best near 400 Hz), is heard only for differences up to about 30 Hz, and evokes weaker brain responses than the acoustic beat of the same two tones. | Ingendoh, Posny and Heine 2023, PLOS ONE 18(5):e0286023 (systematic review) ([doi:10.1371/journal.pone.0286023](https://doi.org/10.1371/journal.pone.0286023)); Schwarz and Taylor 2005, Clinical Neurophysiology 116(3):658–668 ([doi:10.1016/j.clinph.2004.09.014](https://doi.org/10.1016/j.clinph.2004.09.014)); Pratt et al. 2010, Hearing Research 262(1–2):34–44 ([doi:10.1016/j.heares.2010.01.013](https://doi.org/10.1016/j.heares.2010.01.013)) |
| `synth:FM-SIDEBANDS` | V | For a constant index I, sin(ω_c·t + I·sin ω_m·t) = Σₙ Jₙ(I)·sin((ω_c + n·ω_m)t), the Jacobi–Anger expansion (DLMF §10.12, cited, not proved here), so an FM tone has partials at f_c + n·f_m whose amplitudes are \|Jₙ(I)\|. Checked numerically: over one second (1 Hz bins) the DFT amplitudes of fm_tone at f_c + n·f_m match the amplitudes computed from mpmath's Jₙ(I) within 1e-12 for (f_c, f_m, I) = (1000 Hz, 1400 Hz, 2) and (1000 Hz, 1400 Hz, 10) at 44.1 kHz, and where reflected sidebands meet (f_c = f_m = 440 Hz, I = 1, where the carrier reads \|J₀ − J₋₂\|) or alias (1000 Hz, 700 Hz, 3 at 8 kHz). | check: [`vectors/fm.json`](vectors/fm.json) |
| `synth:BAR-MODES` | V | The first four positive roots of cos x·cosh x = 1, the frequency equation of a uniform free–free (Euler–Bernoulli) bar, are 4.7300407…, 7.8532046…, 10.9956078… and 14.1371654…, and its mode-frequency ratios (xₖ/x₁)² are 1 : 2.7565385… : 5.4039176… : 8.9329503… (digits cut off, not rounded), each certified to the digits shown by interval arithmetic (mpmath.iv): the equation has exactly eight roots in (0, 28] and each is bracketed to within 1e-45. lifesound.py's BAR_MODES (1, 2.756, 5.404, 8.933) agree within 5.4e-4: 5.404 and 8.933 are the ratios rounded to three decimals, while 2.756 is 2.75654 cut off (rounded, it would be 2.757). | check: [`vectors/generate.py`](vectors/generate.py) |
| `synth:TWO-POINT-AVERAGE-DELAY` | P | (1 + e^{−iω})/2 = e^{−iω/2}·cos(ω/2) for every real ω: the two-point average of the Karplus–Strong loop delays every frequency by exactly half a sample (while cos(ω/2) > 0, that is for \|ω\| < π). | [`SynthesisTheory.avgResponse_eq`](proofs/SynthesisTheory/KarplusStrong.lean) |
| `synth:KS-PITCH` | P | With a loop delay of N samples and the two-point average, the loop gain G(ω) = e^{−iωN}·(1 + e^{−iω})/2 equals e^{−iω(N + 1/2)}·cos(ω/2), and for \|ω\| < π it is a positive real number exactly when ω = 2πk/(N + 1/2) for an integer k: the loop resonates at f_k = k·sr/(N + 1/2), not at k·sr/N. | [`SynthesisTheory.loopGain_pos_real_iff`](proofs/SynthesisTheory/KarplusStrong.lean) |
| `synth:KS-DECAY` | P | Per trip round the loop with gain decay, a component at ω is scaled in magnitude by \|decay\|·\|cos(ω/2)\|, and for decay ≠ 0 that factor strictly falls as ω rises from 0 to π: higher partials die faster. | [`SynthesisTheory.loopGain_decay`](proofs/SynthesisTheory/KarplusStrong.lean) |
| `synth:KS-DETUNE` | T | karplus_strong rounds its loop to N = max(round(sr/freq), 2) samples and ignores the half sample of the average, so it sounds at karplus_strong_pitch = sr/(N + 1/2): at 44.1 kHz, 438.806 Hz for 440 Hz (N = 100, 4.70 cents flat), 13.30 cents flat at 880 Hz and 30.35 cents flat at 1760 Hz. The fundamental measured from the rendered string (the peak of its Hann-windowed spectrum) is within 0.02 cents of sr/(N + 1/2) from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz, and karplus_strong_detune_cents (pitch_math::cents) gives the detuning. | [`tests/properties.rs`](tests/properties.rs), [`vectors/strings.json`](vectors/strings.json), [`src/string.rs`](src/string.rs) |
| `synth:KS-TUNED` | T | karplus_strong_tuned adds a first-order allpass (C + z⁻¹)/(1 + C·z⁻¹), C = (1 − Δ)/(1 + Δ), that makes the loop N + 1/2 + Δ = sr/freq samples long (Δ in [0.1, 1.1)). Its measured fundamental is within 1 cent of freq (at most 0.15 cents, at 1760 Hz and 44.1 kHz) from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz, where karplus_strong is up to 30 cents flat, and it matches the oracle (a plain-Python loop) within 1e-12. Higher strings, a few samples long, go flat again as the allpass's delay at the fundamental drifts from Δ and the loop's losses, mostly the average's cos(ω/2) rather than decay, pull its pole flat: 2.9 cents at 5 kHz and 16.8 cents at 10 kHz at 44.1 kHz. | [`tests/properties.rs`](tests/properties.rs), [`vectors/strings.json`](vectors/strings.json), [`src/string.rs`](src/string.rs) |
| `synth:KARPLUS-STRONG-MATCHES-REFERENCE` | T | karplus_strong matches lifesound.py's string within 1e-12: a burst of N + 1 uniform samples in [−1, 1), low-passed by two passes of box_lowpass at pluck_brightness_hz and shifted to zero mean, then yᵢ = decay·0.5·(yᵢ₋ₙ + yᵢ₋ₙ₋₁), scaled to a peak just below 1, with the burst drawn from signal-core's SplitMix64 instead of numpy's generator. | [`vectors/strings.json`](vectors/strings.json), [`src/string.rs`](src/string.rs) |
| `synth:MODAL-CHIME-MATCHES-REFERENCE` | T | modal_chime matches lifesound.py within 1e-12: one sine per mode of BAR_MODES at freq·ratio, mode k with amplitude (0.55·brightness)ᵏ and a 60 dB decay over dur/(1 + 1.6k) seconds, plus a 2 ms strike at 6·freq, scaled to a peak just below 1. | [`vectors/instruments.json`](vectors/instruments.json), [`src/modal.rs`](src/modal.rs) |
| `synth:FM-MATCHES-REFERENCE` | T | fm_bell matches lifesound.py's FM bell within 1e-12 (A(t)·sin(2π·fc·t + index·A(t)·sin(2π·fc·ratio·t)), A falling 60 dB over dur, then a 2 ms attack and 50 ms release), and fm_tone matches its formula sin(2π·fc·t + index·sin(2π·fm·t)) evaluated with numpy. | [`vectors/instruments.json`](vectors/instruments.json), [`vectors/fm.json`](vectors/fm.json) |
| `synth:SOFT-VOICE-MATCHES-REFERENCE` | T | soft_voice matches lifesound.py within 1e-12 on the golden vectors (signals of at most 600 samples): harmonics 1–4 (amplitudes 1, 0.25, 0.08, 0.03) of freq·2^(c/1200), with c = vibrato_cents·sin(2π·vibrato_hz·t)·min(t/0.4, 1), phase-integrated by signal-core's osc_sweep and shaped by its adsr. With vibrato the difference grows with the length of the signal, so longer voices can differ by more than 1e-12. | [`vectors/instruments.json`](vectors/instruments.json), [`src/voice.rs`](src/voice.rs) |
| `synth:HEARTBEAT-MATCHES-REFERENCE` | T | heartbeat matches lifesound.py exactly in placement and within 1e-12 in value: per beat of 60/bpm seconds, a 0.18 s thump sin(2π·f0·t·(1 − 0.35·t/0.18))·e^{−t/0.045} and a second at amplitude 0.6, min(0.3, 0.4·60/bpm) s later, each left out if it would reach the end. | [`vectors/instruments.json`](vectors/instruments.json), [`src/voice.rs`](src/voice.rs) |
| `synth:BAR-ROOTS-IN-CODE` | T | free_bar_roots (bisection on cos x − 1/cosh x between kπ and (k + 1)π, down to adjacent doubles) returns the certified roots of synth:BAR-MODES rounded to the nearest double, for the first eight within 1e-15 relative (exactly, on the tested platforms), and free_bar_ratios their ratios (xₖ/x₁)² within 1e-15 relative. | [`vectors/bar_modes.json`](vectors/bar_modes.json), [`src/modal.rs`](src/modal.rs) |
| `synth:INPUT-VALIDATION` | T | Each function rejects, among its arguments, a zero or negative sample rate; a tone frequency (freq, fc, fm, f0), pluck brightness or bpm of 0 or below; a negative duration (and for modal_chime and fm_bell a duration of 0, for fm_spectrum one too short for a sample); NaN or an infinity anywhere; a decay outside [0, 1]; a string whose burst (one period plus one sample) would exceed MAX_SAMPLES; a tuned string above sr/2; more than one heartbeat per sample; and counts above their limits. The shape parameters brightness, ratio, index, vibrato_hz and vibrato_cents may be any finite number, as in lifesound.py, except that soft_voice also rejects, where lifesound.py gave NaN samples (or a 0 Hz pitch), a freq above f64::MAX/4 (its 4th harmonic would overflow), a vibrato_hz so large that 2π·vibrato_hz·t overflows by the last sample, and a vibrato_cents so large in size (about 1.2e6 at 220 Hz) that a harmonic of the pitch overflows or the pitch underflows to 0 Hz, each error naming that argument, though none of the three when the duration is too short for a sample (the signal is then empty, as in lifesound.py). Rust returns SynthError, Python raises ValueError (OverflowError for a negative or too-large integer argument such as a count, sample rate or seed, TypeError for a non-integer one) and JavaScript throws an Error (also for a count, sample rate or seed that is not a whole number up to 2^53 − 1). | [`vectors/errors.json`](vectors/errors.json), [`tests/properties.rs`](tests/properties.rs), [`src/lib.rs`](src/lib.rs), [`src/fm.rs`](src/fm.rs), [`src/string.rs`](src/string.rs), [`src/voice.rs`](src/voice.rs), [`bindings/python/tests/test_vectors.py`](bindings/python/tests/test_vectors.py), [`bindings/wasm/tests/vectors.test.mjs`](bindings/wasm/tests/vectors.test.mjs) |
| `synth:IMPLEMENTATIONS-AGREE` | T | The Rust crate, the Python bindings and the WebAssembly bindings all reproduce every golden vector. Every expected output comes from the oracle (lifesound.py's instruments run with numpy, with signal-core's SplitMix64 in place of numpy's generator and ln 1000 in exp_decay, plus exact fractions, mpmath Bessel functions and mpmath.iv intervals for the theory), never from the Rust output; the error cases in errors.json are the crate's own input rules. | [`vectors/generate.py`](vectors/generate.py), [`tests/vectors.rs`](tests/vectors.rs), [`bindings/python/tests/test_vectors.py`](bindings/python/tests/test_vectors.py), [`bindings/wasm/tests/vectors.test.mjs`](bindings/wasm/tests/vectors.test.mjs) |
<!-- claims-table:end -->

Levels: **P** machine-checked proof (Lean 4) · **V** verified numerics · **T** tested · **R** referenced (DOI or PMID) · **M** model assumption.

## Use

A port of the instruments of the project's earlier Python toolkit (`lifesound.py`): a struck bar (`modal_chime`), Chowning's FM bell (`fm_bell`), the Karplus–Strong plucked string (`karplus_strong`), a soft voice (`soft_voice`) and a heartbeat (`heartbeat`). Beside them is the theory they rest on: the pitch a Karplus–Strong string really sounds at (`karplus_strong_pitch`, `karplus_strong_detune_cents`) and a string tuned with an allpass filter (`karplus_strong_tuned`); constant-index FM (`fm_tone`) and its measured sidebands (`fm_spectrum`); and the mode ratios of a free bar (`free_bar_roots`, `free_bar_ratios`, and the constant `BAR_MODES` that `lifesound.py` used). Samples are `f64`, mono, and every function takes the sample rate `sr` (44 100 is the usual value, exported as `SR`). Envelopes, sample counts, the moving-average filter and the seeded SplitMix64 noise come from [signal-core](https://github.com/chethandvg/signal-core), and cents from [pitch-math](https://github.com/chethandvg/pitch-math).

**Rust**

```toml
[dependencies]
synthesis-theory = { git = "https://github.com/chethandvg/synthesis-theory", tag = "v0.1.0" }
```

```rust
use synthesis_theory::{fm_spectrum, free_bar_ratios, karplus_strong, karplus_strong_pitch, karplus_strong_tuned, SR};

// lifesound.py's plucked A4: a 100-sample loop plus half a sample from the average.
let flat = karplus_strong(440.0, 2.0, SR, 0.996, 0, Some(2000.0))?;
assert_eq!(karplus_strong_pitch(440.0, SR)?, 44_100.0 / 100.5); // 438.806 Hz, 4.70 cents flat
// The same string with an allpass that adds the missing fraction of a sample: 440 Hz.
let tuned = karplus_strong_tuned(440.0, 2.0, SR, 0.996, 0, Some(2000.0))?;
// FM with carrier 1000 Hz, modulator 1400 Hz, index 2: |J_n(2)| at 1000 + 1400n Hz, n = -3..=3.
let sidebands = fm_spectrum(1000.0, 1400.0, 2.0, 3, 1.0, SR)?; // 0.1289, 0.3528, 0.5767, 0.2239, …
let ratios = free_bar_ratios(4)?; // 1, 2.756539, 5.403918, 8.932950
```

**Python** (builds the Rust core with maturin; signals come back as numpy arrays)

```bash
pip install "git+https://github.com/chethandvg/synthesis-theory@v0.1.0#subdirectory=bindings/python"
```

```python
import synthesis_theory as st

st.karplus_strong_pitch(440.0)  # 438.8059701492537 (sr=SR)
st.karplus_strong_detune_cents(880.0)  # -13.296193133050858
pluck = st.karplus_strong(440.0, 2.0)  # numpy.ndarray of 88200 samples, lifesound.py's defaults
bell = st.fm_bell(440.0)  # 6 s, ratio 1.4, index 10
st.fm_spectrum(1000.0, 1400.0, 2.0, 3)  # array([0.1289, 0.3528, 0.5767, 0.2239, 0.5767, 0.3528, 0.1289]) (rounded)
st.free_bar_ratios(4)  # array([1.        , 2.75653851, 5.40391763, 8.93295035])
st.karplus_strong(440.0, 1.0, decay=1.5)  # ValueError: decay must be in [0, 1], got 1.5
```

**JavaScript / WebAssembly** (build steps in [bindings/wasm](bindings/wasm))

```js
import init, { karplus_strong, karplus_strong_tuned, fm_bell, heartbeat } from "./pkg/web/synthesis_theory_wasm.js";

await init();
const flat = karplus_strong(440, 2, 44100, 0.996, 0, 2000);        // Float64Array, sounds at 438.81 Hz
const tuned = karplus_strong_tuned(440, 2, 44100, 0.996, 0, 2000); // the same pluck at 440 Hz
const bell = fm_bell(440, 6, 1.4, 10, 44100);
const heart = heartbeat(72, 10, 44100, 48);
```

Ten seconds each of the six instruments at 44.1 kHz take about 0.2 s in Node; the WebAssembly test checks that they finish in under 10 s.

The functions have the same names and positional arguments in all three languages. Python adds `lifesound.py`'s argument names and defaults (`dur=3.0` for `modal_chime`, `dur=6.0, ratio=1.4, index=10.0` for `fm_bell`, `sr=SR, decay=0.996, seed=0, pluck_brightness_hz=2000.0` for the strings, and so on) and the constants `SR`, `MAX_SAMPLES` and `BAR_MODES`; `fm_spectrum` defaults to `dur=1.0`, which gives 1 Hz bins. In Python `pluck_brightness_hz=None`, and in JavaScript `null` or leaving it out, gives an unfiltered pluck. In JavaScript, sample rates, seeds and counts are numbers that must be whole and at most 2⁵³ − 1. Errors are `SynthError` in Rust, `ValueError` in Python (`OverflowError` for a negative or too-large integer argument, `TypeError` for a non-integer one) and `Error` in JavaScript.

### Differences from `lifesound.py`

- **Noise:** the string's noise burst comes from signal-core's SplitMix64, not numpy's PCG64, so its samples differ from the old toolkit's; the algorithm and the order of the draws are the same.
- **`exp_decay`** (used by `modal_chime` and `fm_bell`) is signal-core's, with ln 1000 = 6.907755278982137 where `lifesound.py` used 6.907755.
- **Strict input:** invalid arguments are errors. `karplus_strong` takes a decay in [0, 1] (above 1 the string grows) and `None` rather than 0 for an unfiltered pluck. `modal_chime` and `fm_bell` reject a duration of 0 or below (`lifesound.py`'s `modal_chime` failed, on the empty array at 0 and on a negative array size below 0, and its `fm_bell` returned an empty array). A positive duration that rounds to zero samples gives an empty signal from both (`lifesound.py`'s `fm_bell` did the same; its `modal_chime` failed). `fm_tone`, `fm_spectrum` and `heartbeat` need `fc`, `fm` and `f0` above 0. The shape parameters `brightness`, `ratio`, `index`, `vibrato_hz` and `vibrato_cents` may be any finite number, as in `lifesound.py` (0 turns the effect off; a negative value flips a sign). A frequency so large that `2π·f·t` overflows (near `f64::MAX`) gives NaN samples, not an error, in both. The one exception is `soft_voice`, which also rejects, where `lifesound.py` gave NaN samples (or a 0 Hz pitch), a `freq` above `f64::MAX / 4` (its 4th harmonic would overflow), a `vibrato_hz` so large that `2π·vibrato_hz·t` overflows by the last sample, and a `vibrato_cents` so large in size (about 1.2e6 at 220 Hz) that a harmonic of the pitch overflows or the pitch underflows to 0 Hz, though none of the three when the duration is too short for a sample (the signal is then empty, as in `lifesound.py`).
- **New here:** `karplus_strong_pitch`, `karplus_strong_detune_cents`, `karplus_strong_tuned`, `fm_tone`, `fm_spectrum`, `free_bar_roots` and `free_bar_ratios`.

## How it is verified

- **Proofs:** the Lean 4 files in [`proofs/`](proofs) prove the five P-level claims. In [`Beats.lean`](proofs/SynthesisTheory/Beats.lean): the sum-to-product identity `sin a + sin b = 2 sin((a + b)/2) cos((a − b)/2)`, the two-tone form `2 cos(π(f₁ − f₂)t) sin(π(f₁ + f₂)t)`, and that the envelope `|2 cos(π(f₁ − f₂)t)|` repeats every `1/|f₁ − f₂|` seconds with exactly one zero per period. In [`KarplusStrong.lean`](proofs/SynthesisTheory/KarplusStrong.lean): `(1 + e^{−iω})/2 = e^{−iω/2} cos(ω/2)` (the average delays every frequency by half a sample); the loop gain `e^{−iωN}(1 + e^{−iω})/2` is a positive real for `|ω| < π` exactly at `ω = 2πk/(N + 1/2)`; and each trip scales a partial by `|decay|·|cos(ω/2)|`, which falls as `ω` rises. CI rebuilds the proofs with Mathlib, rejects `sorry`, and allows only the standard axioms (`propext`, `Classical.choice`, `Quot.sound`).
- **Verified numerics:** the free-bar roots of `cos x · cosh x = 1` are certified by interval arithmetic (mpmath.iv) in [`vectors/generate.py`](vectors/generate.py), which stops before writing any vector if a step fails: no root in `(0, 2]` (an alternating-series bound), exactly eight in `(0, 28]`, each bracketed to within 1e-45. The FM sidebands are checked against the Jacobi–Anger amplitudes `|Jₙ(I)|` from mpmath: the expected values in [`vectors/fm.json`](vectors/fm.json) come from the Bessel functions alone, and the crate's `fm_spectrum` measures them from a rendered tone in Rust, Python and WebAssembly, within 1e-12. The certification runs when the maintainer regenerates the vectors, not in CI; CI checks the code against its results.
- **Referenced, not proved:** the Jacobi–Anger expansion itself is cited (DLMF §10.12), and binaural beats, a perception of the brain rather than a sum of waves, are an R claim citing a systematic review and two human EEG studies.
- **Implementation:** one Rust implementation, compiled to Python (PyO3, numpy arrays) and WebAssembly. The same 78 golden vectors in [`vectors/`](vectors) are checked in all three; CI runs the Rust and Python checks on Linux and Windows and the WebAssembly checks on Linux. Their oracle runs `lifesound.py`'s instruments with numpy, copied with signal-core's two deliberate changes (SplitMix64 noise, ln 1000 in `exp_decay`), plus exact fractions, mpmath Bessel functions and certified intervals for the theory; nothing in it calls the Rust code. Every expected output comes from the oracle; the error cases in `errors.json` are the crate's own input rules. Property tests in [`tests/properties.rs`](tests/properties.rs) measure the pitch of rendered strings: `karplus_strong` within 0.02 cents of `sr/(N + 1/2)` and `karplus_strong_tuned` within 1 cent of the requested frequency (at most 0.15 cents), from 110 to 1760 Hz at 44.1 kHz and at 440 and 1760 Hz at 48 kHz. Above that range, in loops of a few samples, both go flat (also measured at 44.1 kHz): the tuned string by 2.9 cents at 5 kHz and 16.8 cents at 10 kHz, and `karplus_strong` by 6.8 cents from `sr/(N + 1/2)` at `N = 4` and about 110 cents at `N = 2`. `soft_voice` matches `lifesound.py` within 1e-12 on the short vectors; with vibrato the difference grows with length (1.4e-12 after 2 s at 44.1 kHz), because the crate takes cents through pitch-math.
- **What the proofs don't cover:** they are about exact real and complex numbers. They do not cover floating-point rounding, the code's finite signals, or the damping (`decay` and the two-point average's `cos(ω/2)`) that moves the string's poles off `2πk/(N + 1/2)` (the fundamental's by under 0.01 cents from 110 to 1760 Hz, but by 6.8 cents at `N = 4`; each higher partial's by more); that the rendered strings sound where the proofs say is tested, not proved. The proofs say nothing about how the ear hears a beat, and nothing about binaural beats. The allpass of `karplus_strong_tuned` has exactly the delay `Δ` only at 0 Hz; its accuracy at the partials is tested, not proved. The free-bar ratios are certified for the first eight roots only, and the instruments (`modal_chime`, `fm_bell`, `soft_voice`, `heartbeat`) are tested against the reference, not proved.

## Cite

If you use this work, please cite it. GitHub's **Cite this repository** button gives APA and BibTeX from [CITATION.cff](CITATION.cff).

## Licence

Code: Apache-2.0 ([LICENSE](LICENSE)). Copyright notices are in [NOTICE](NOTICE).
