# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama
"""Write the golden vectors for synthesis-theory from the reference implementation.

The oracle has two parts, and nothing here calls the Rust code.

1. The instruments of the project's earlier Python toolkit, `lifesound.py`
   (`modal_chime`, `fm_bell`, `karplus_strong`, `soft_voice`, `heartbeat` and the
   helpers they call), copied below unchanged with the two deliberate
   differences that signal-core's oracle makes, both documented in the README:

   * randomness comes from `SplitMix64`, a pure-Python port of signal-core's
     generator (copied from signal-core's vectors/generate.py, because one
     repo's Python cannot import another's), offering the `uniform` method
     lifesound.py calls, so the copied code runs as written;
   * `exp_decay` uses ln(1000) = 6.907755278982137 where lifesound.py had 6.907755.

2. Exact or certified references for the theory, independent of any rendering:
   the Karplus-Strong pitch sr / (N + 1/2) in exact fractions, its detuning in
   cents with mpmath, the FM sideband amplitudes from mpmath's Bessel functions,
   and the free-bar roots of cos x cosh x = 1 certified by interval arithmetic
   (mpmath.iv). The certification is the check of the V claim synth:BAR-MODES:
   if any step fails, this script stops before writing anything.

`karplus_strong_tuned` and `fm_tone` are not in lifesound.py; their reference is
the plain-Python loop and numpy formula below. Run with numpy and mpmath installed:

    python vectors/generate.py
"""

from __future__ import annotations

import json
import math
from fractions import Fraction
from pathlib import Path

import mpmath as mp
import numpy as np

HERE = Path(__file__).resolve().parent
SR = 44100
EXACT = {"rel": 0, "abs": 0}
CLOSE = {"rel": 1e-12, "abs": 1e-12}
mp.mp.dps = 50
mp.iv.dps = 50


def write(name: str, description: str, cases: list[dict], tolerance: dict = CLOSE) -> None:
    doc = {"schema": "golden-vectors@1", "description": description, "tolerance": tolerance, "cases": cases}
    (HERE / f"{name}.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


# signal-core's random number generator, ported (copied from signal-core's vectors/generate.py) --------

MASK = (1 << 64) - 1
GAMMA = 0x9E3779B97F4A7C15


class SplitMix64:
    """SplitMix64 (Steele, Lea and Flood 2014), with the numpy Generator method lifesound.py's strings call."""

    def __init__(self, seed: int) -> None:
        self.state = seed & MASK

    def next_u64(self) -> int:
        self.state = (self.state + GAMMA) & MASK
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)

    def random(self) -> float:
        return (self.next_u64() >> 11) * 2.0**-53  # exact: 53 bits times a power of two

    def uniform(self, low: float, high: float, size) -> np.ndarray:
        shape = (size,) if isinstance(size, int) else tuple(size)
        count = math.prod(shape)
        return np.array([low + (high - low) * self.random() for _ in range(count)], dtype=float).reshape(shape)


class _Random:
    """Stands in for `np.random` inside the copied lifesound.py code."""

    default_rng = SplitMix64


rng_module = _Random()


# The reference implementation: lifesound.py ------------------------------------


def n_samples(dur: float, sr: int = SR) -> int:
    return int(round(dur * sr))


def time_axis(dur: float, sr: int = SR) -> np.ndarray:
    return np.arange(n_samples(dur, sr)) / sr


def osc(freq, dur: float, sr: int = SR, phase: float = 0.0, shape: str = "sine") -> np.ndarray:
    n = n_samples(dur, sr)
    f = np.broadcast_to(np.asarray(freq, dtype=float), (n,))
    ph = 2 * np.pi * np.cumsum(f) / sr + phase
    if shape == "sine":
        return np.sin(ph)
    if shape == "triangle":
        return 2 / np.pi * np.arcsin(np.sin(ph))
    raise ValueError(shape)


def additive(f0, dur: float, partial_amps, sr: int = SR, ratios=None) -> np.ndarray:
    ratios = ratios if ratios is not None else np.arange(1, len(partial_amps) + 1)
    out = np.zeros(n_samples(dur, sr))
    for r, a in zip(ratios, partial_amps, strict=False):
        out += np.asarray(a) * osc(np.asarray(f0) * r, dur, sr)
    return out


def adsr(n: int, sr: int = SR, a=0.01, d=0.1, s=0.7, r=0.2) -> np.ndarray:
    na, nd, nr = int(a * sr), int(d * sr), int(r * sr)
    ns = max(n - na - nd - nr, 0)
    env = np.concatenate(
        [
            np.linspace(0, 1, na, endpoint=False),
            np.linspace(1, s, nd, endpoint=False),
            np.full(ns, s),
            np.linspace(s, 0, nr),
        ]
    )
    return np.pad(env, (0, max(n - len(env), 0)))[:n]


def exp_decay(n: int, t60: float, sr: int = SR) -> np.ndarray:
    return np.exp(-math.log(1000) * np.arange(n) / (sr * t60))  # lifesound.py: 6.907755


BAR_MODES = (1.0, 2.756, 5.404, 8.933)


def modal_chime(freq: float, dur: float = 3.0, brightness: float = 1.0, sr: int = SR) -> np.ndarray:
    n = n_samples(dur, sr)
    t = np.arange(n) / sr
    out = np.zeros(n)
    for k, ratio in enumerate(BAR_MODES):
        amp = (0.55**k) * (brightness**k)
        t60 = dur / (1 + 1.6 * k)  # higher modes die away faster
        out += amp * np.sin(2 * np.pi * freq * ratio * t) * exp_decay(n, t60, sr)
    strike = np.exp(-t / 0.002) * 0.15  # tiny noise-free "strike" transient
    out[: len(strike)] += strike * np.sin(2 * np.pi * freq * 6.0 * t)
    return out / (np.max(np.abs(out)) + 1e-12)


def fm_bell(fc: float, dur: float = 6.0, ratio: float = 1.4, index: float = 10.0, sr: int = SR) -> np.ndarray:
    n = n_samples(dur, sr)
    t = np.arange(n) / sr
    env = exp_decay(n, dur, sr)
    idx = index * env  # brightness decays with loudness, like a real bell
    y = env * np.sin(2 * np.pi * fc * t + idx * np.sin(2 * np.pi * fc * ratio * t))
    return y * adsr(n, sr, a=0.002, d=0.0, s=1.0, r=0.05)


def karplus_strong(
    freq: float,
    dur: float,
    sr: int = SR,
    decay: float = 0.996,
    seed: int = 0,
    pluck_brightness_hz: float | None = 2000.0,
) -> np.ndarray:
    rng = rng_module.default_rng(seed)
    period = max(int(round(sr / freq)), 2)
    n = n_samples(dur, sr)
    out = np.zeros(n + period + 1)
    burst = rng.uniform(-1, 1, period + 1)
    if pluck_brightness_hz:
        burst = box_lowpass(burst, pluck_brightness_hz, sr, passes=2)
    out[: period + 1] = burst - burst.mean()
    pos = period + 1
    while pos < len(out):
        end = min(pos + period, len(out))
        seg = np.arange(pos, end)
        out[seg] = decay * 0.5 * (out[seg - period] + out[seg - period - 1])
        pos = end
    return out[:n] / (np.max(np.abs(out[:n])) + 1e-12)


def soft_voice(freq: float, dur: float, sr: int = SR, vibrato_hz: float = 5.0, vibrato_cents: float = 12) -> np.ndarray:
    t = time_axis(dur, sr)
    f = freq * 2 ** (vibrato_cents / 1200 * np.sin(2 * np.pi * vibrato_hz * t) * np.minimum(t / 0.4, 1))
    y = additive(f, dur, [1.0, 0.25, 0.08, 0.03], sr)
    return y * adsr(len(y), sr, a=0.08, d=0.2, s=0.8, r=min(0.4, dur / 3))


def heartbeat(bpm: float, dur: float, sr: int = SR, f0: float = 48.0) -> np.ndarray:
    out = np.zeros(n_samples(dur, sr))
    thump_n = n_samples(0.18, sr)
    tt = np.arange(thump_n) / sr
    thump = np.sin(2 * np.pi * f0 * tt * (1 - 0.35 * tt / 0.18)) * np.exp(-tt / 0.045)
    beat = 60.0 / bpm
    s2_delay = min(0.30, beat * 0.4)
    t = 0.0
    while t < dur:
        for offset, amp in ((0.0, 1.0), (s2_delay, 0.6)):
            i = n_samples(t + offset, sr)
            if i + thump_n < len(out):
                out[i : i + thump_n] += amp * thump
        t += beat
    return out


def box_lowpass(x: np.ndarray, cutoff_hz: float, sr: int = SR, passes: int = 3) -> np.ndarray:
    h = max(int(0.44 * sr / cutoff_hz) // 2, 1)
    L = 2 * h + 1
    y = x.astype(float)
    for _ in range(passes):
        padded = np.pad(y, [(h + 1, h)] + [(0, 0)] * (y.ndim - 1), mode="edge")
        c = np.cumsum(padded, axis=0)
        y = (c[L:] - c[:-L]) / L
    return y


# References beyond lifesound.py --------------------------------------------------


def fm_tone(fc: float, fm: float, index: float, dur: float, sr: int = SR) -> np.ndarray:
    """Constant-index FM, written as lifesound.fm_bell writes its phase."""
    t = np.arange(n_samples(dur, sr)) / sr
    return np.sin(2 * np.pi * fc * t + index * np.sin(2 * np.pi * fm * t))


def karplus_strong_tuned(
    freq: float,
    dur: float,
    sr: int = SR,
    decay: float = 0.996,
    seed: int = 0,
    pluck_brightness_hz: float | None = 2000.0,
) -> np.ndarray:
    """lifesound.py's string with a first-order allpass of delay delta in the loop, as a plain loop."""
    rng = rng_module.default_rng(seed)
    d = sr / freq
    period = math.floor(d - 0.5 - 0.1)
    delta = d - 0.5 - period
    c = (1 - delta) / (1 + delta)
    n = n_samples(dur, sr)
    burst = rng.uniform(-1, 1, period + 1)
    if pluck_brightness_hz:
        burst = box_lowpass(burst, pluck_brightness_hz, sr, passes=2)
    out = list(burst - burst.mean())
    k = decay * 0.5
    v1 = y1 = 0.0
    for i in range(period + 1, n):
        v = k * (out[i - period] + out[i - period - 1])
        y = c * v + v1 - c * y1
        v1, y1 = v, y
        out.append(y)
    x = np.array(out[:n])
    return x / (np.max(np.abs(x)) + 1e-12)


def ks_pitch(freq: float, sr: int) -> Fraction:
    """The exact pitch of lifesound's string: sr / (N + 1/2), N = max(round(sr / freq), 2)."""
    return Fraction(sr) / (max(int(round(sr / freq)), 2) + Fraction(1, 2))


def ks_detune_cents(freq: float, sr: int) -> float:
    p = ks_pitch(freq, sr)
    return float(1200 * mp.log(mp.mpf(p.numerator) / p.denominator / mp.mpf(freq), 2))


def fm_spectrum(fc: int, fm: int, index: float, n_max: int, dur: float, sr: int) -> list[float]:
    """|Jacobi-Anger amplitude| at fc + n fm, n = -n_max..n_max, as the DFT over N = dur * sr samples sees it.

    The tone is sum_m J_m(index) sin(2 pi F_m t) with F_m = fc + m fm. With every F_m a whole number of
    1/dur-hertz bins, the DFT of sin at bin F has (N / 2i) at bin F and -(N / 2i) at bin -F (mod N), so the
    amplitude (2/N)|X| at bin K is |sum_m J_m ([F_m = K] - [F_m = -K])|, all modulo N. Terms with |m| > 80
    are below 1e-60 for the indices used here.
    """
    n = n_samples(dur, sr)
    to_bin = Fraction(n, sr)
    assert (fc * to_bin).denominator == 1 and (fm * to_bin).denominator == 1, "frequencies must be whole bins"
    bc, bm = int(fc * to_bin), int(fm * to_bin)
    J = {m: mp.besselj(m, mp.mpf(index)) for m in range(-80, 81)}
    out = []
    for k in range(-n_max, n_max + 1):
        K = (bc + k * bm) % n
        s = sum(J[m] for m in J if (bc + m * bm - K) % n == 0) - sum(J[m] for m in J if (bc + m * bm + K) % n == 0)
        out.append(float(abs(s)))
    return out


# The free-free bar: roots of cos x cosh x = 1, certified with interval arithmetic ---------------


def _f(x):
    """cos x cosh x - 1, for an mpmath interval x."""
    return mp.iv.cos(x) * (mp.iv.exp(x) + mp.iv.exp(-x)) / 2 - 1


def _df(x):
    """d/dx (cos x cosh x) = cos x sinh x - sin x cosh x."""
    e, ei = mp.iv.exp(x), mp.iv.exp(-x)
    return mp.iv.cos(x) * (e - ei) / 2 - mp.iv.sin(x) * (e + ei) / 2


def _sign(x) -> int:
    """+1 or -1 if the interval f(x) has a definite sign, 0 if it contains 0."""
    v = _f(x)
    return 1 if v.a > 0 else -1 if v.b < 0 else 0


def certified_bar_roots(x_end: Fraction = Fraction(28)) -> list:
    """Every root of cos x cosh x = 1 in (0, x_end], each as an interval narrower than 1e-45.

    (0, 2]: cos x cosh x = sum_k (-4)^k x^(4k) / (4k)!, so f(x) = -x^4/6 + x^8/2520 - ...; for 0 < x <= 2 the
    terms after the first fall (each is at most 4 * 2^4 / (5*6*7*8) of the one before), so the alternating
    tail is bounded by its first term and f(x) <= -x^4 (1/6 - x^4/2520) < 0: no root.
    [2, x_end]: split until every piece either has f of one sign (no root) or f' of one sign with f of
    opposite signs at its ends (exactly one root), then bisect each root's piece with sign-definite midpoints.
    """
    assert Fraction(4 * 2**4, 5 * 6 * 7 * 8) < 1 and Fraction(2**4, 2520) < Fraction(1, 6)
    brackets, stack = [], [(Fraction(2), x_end)]
    while stack:
        a, b = stack.pop()
        whole = mp.iv.mpf([mp.mpf(a.numerator) / a.denominator, mp.mpf(b.numerator) / b.denominator])
        v = _f(whole)
        if v.a > 0 or v.b < 0:
            continue  # no root in [a, b]
        dv = _df(whole)
        sa, sb = _sign(_point(a)), _sign(_point(b))
        if (dv.a > 0 or dv.b < 0) and sa * sb == -1:
            brackets.append((a, b))  # monotone with a sign change: exactly one root
        else:
            assert b - a > Fraction(1, 2**40), f"cannot classify [{float(a)}, {float(b)}]"
            mid = (a + b) / 2
            stack += [(mid, b), (a, mid)]
    roots = []
    for a, b in sorted(brackets):
        sa = _sign(_point(a))
        while b - a > Fraction(1, 10**45):
            mid = (a + b) / 2
            sm = _sign(_point(mid))
            assert sm != 0, "a midpoint sits on the root"
            a, b = (mid, b) if sm == sa else (a, mid)
        roots.append(mp.iv.mpf([_mpf(a), _mpf(b)]))
    return roots


def _mpf(q: Fraction):
    return mp.mpf(q.numerator) / q.denominator


def _point(q: Fraction):
    return mp.iv.mpf([_mpf(q), _mpf(q)])


def nearest_double(x) -> float:
    """The double nearest the interval x, which must round the same at both ends (so the value is certain)."""
    lo, hi = (float(mp.mpf(end)) for end in x._mpi_)  # the exact endpoints, rounded to nearest
    assert lo == hi, f"{x} straddles a rounding boundary"
    return lo


ROOTS = certified_bar_roots()
assert len(ROOTS) == 8, f"expected 8 roots in (0, 28], found {len(ROOTS)}"
RATIOS = [(r / ROOTS[0]) ** 2 for r in ROOTS]
# synth:BAR-MODES: the first four ratios, and how lifesound.py's BAR_MODES sit beside them.
for ratio, lo, hi in ((RATIOS[1], "2.7565385", "2.7565386"), (RATIOS[2], "5.4039176", "5.4039177")):
    assert mp.mpf(lo) < ratio.a and ratio.b < mp.mpf(hi)
assert mp.mpf("8.9329503") < RATIOS[3].a and RATIOS[3].b < mp.mpf("8.9329504")
for ratio, given in zip(RATIOS[1:4], BAR_MODES[1:], strict=True):
    assert abs(ratio - mp.mpf(given)).b < mp.mpf("5.4e-4")
assert [round(float(r.a), 3) for r in RATIOS[1:4]] == [2.757, 5.404, 8.933]  # 2.756 is cut off, not rounded


# Adapters from the crate's API to the reference ---------------------------------


def listed(fn):
    return lambda *args: np.asarray(fn(*args), dtype=float).tolist()


ORACLE = {
    "modal_chime": listed(modal_chime),
    "fm_bell": listed(fm_bell),
    "fm_tone": listed(fm_tone),
    "fm_spectrum": fm_spectrum,
    "karplus_strong": listed(karplus_strong),
    "karplus_strong_tuned": listed(karplus_strong_tuned),
    "karplus_strong_pitch": lambda freq, sr: float(ks_pitch(freq, sr)),
    "karplus_strong_detune_cents": ks_detune_cents,
    "soft_voice": listed(soft_voice),
    "heartbeat": listed(heartbeat),
    "free_bar_roots": lambda count: [nearest_double(r) for r in ROOTS[:count]],
    "free_bar_ratios": lambda count: [nearest_double(r) for r in RATIOS[:count]],
}


def case(claim: str, fn: str, *args, note: str | None = None, tolerance: dict | None = None) -> dict:
    """A case whose expected value comes from the oracle."""
    c = {"claim": claim, "fn": fn, "args": list(args), "expected": ORACLE[fn](*args)}
    if tolerance is not None:
        c["tolerance"] = tolerance
    if note:
        c["note"] = note
    return c


def error(fn: str, *args, note: str) -> dict:
    return {"claim": "synth:INPUT-VALIDATION", "fn": fn, "args": list(args), "error": True, "note": note}


# The cases ------------------------------------------------------------------------


def instruments() -> list[dict]:
    m, b, v, h = (f"synth:{x}-MATCHES-REFERENCE" for x in ("MODAL-CHIME", "FM", "SOFT-VOICE", "HEARTBEAT"))
    return [
        case(m, "modal_chime", 880.0, 0.05, 1.0, 8000, note="400 samples of a struck bar"),
        case(m, "modal_chime", 440.0, 0.02, 0.5, 8000, note="a darker strike: mode k has amplitude 0.275^k"),
        case(m, "modal_chime", 523.25, 0.01, 0.0, 22050, note="brightness 0: the fundamental mode and the strike"),
        case(b, "fm_bell", 440.0, 0.05, 1.4, 10.0, 8000, note="lifesound.py's bell: ratio 1.4, index 10"),
        case(b, "fm_bell", 200.0, 0.03, 2.0, 3.0, 8000),
        case(b, "fm_bell", 300.0, 0.01, 1.4, 0.0, 8000, note="index 0: a decaying sine"),
        case(v, "soft_voice", 220.0, 0.05, 8000, 5.0, 12, note="lifesound.py's vibrato: 12 cents at 5 Hz"),
        case(v, "soft_voice", 110.0, 0.6, 1000, 5.0, 50.0, note="the vibrato fades in over 0.4 s, then holds"),
        case(v, "soft_voice", 330.0, 0.03, 8000, 6.0, 0.0, note="no vibrato: plain additive synthesis"),
        case(h, "heartbeat", 72.0, 1.5, 400, 48.0, note="72 bpm: S2 0.3 s after S1"),
        case(h, "heartbeat", 150.0, 1.0, 1000, 60.0, note="150 bpm: S2 at 0.4 beat = 0.16 s; last thumps cut off"),
        case(h, "heartbeat", 60.0, 0.1, 1000, 48.0, note="too short for a whole thump: silence"),
    ]


def strings() -> list[dict]:
    ks, tuned = "synth:KARPLUS-STRONG-MATCHES-REFERENCE", "synth:KS-TUNED"
    detune = "synth:KS-DETUNE"
    cases = [
        case(ks, "karplus_strong", 1000.0, 0.01, 8000, 0.996, 0, 2000.0, note="period 8, burst low-passed"),
        case(ks, "karplus_strong", 440.0, 0.02, 8000, 0.99, 7, None, note="period round(18.18) = 18, bright burst"),
        case(ks, "karplus_strong", 4000.0, 0.002, 8000, 1.0, 3, 1000.0, note="period 2, the shortest loop"),
        case(ks, "karplus_strong", 300.0, 0.01, 8000, 0.5, 2**53 - 1, None, note="the largest seed JS passes exactly"),
        case(ks, "karplus_strong", 100.0, 0.001, 8000, 0.996, 0, None, note="shorter than the burst"),
        case(tuned, "karplus_strong_tuned", 1000.0, 0.01, 8000, 0.996, 0, 2000.0, note="N = 7, delta = 0.5"),
        case(tuned, "karplus_strong_tuned", 440.0, 0.02, 8000, 0.99, 7, None, note="N = 17, delta = 0.68"),
        case(tuned, "karplus_strong_tuned", 3000.0, 0.005, 8000, 1.0, 1, None, note="N = 2, delta = 0.167"),
    ]
    table = [(110.0, 44100), (220.0, 44100), (261.6255653005986, 44100), (440.0, 44100), (880.0, 44100)]
    table += [(1760.0, 44100), (440.0, 48000), (30000.0, 44100)]
    for f, sr in table:
        n = max(int(round(sr / f)), 2)
        cases.append(case(detune, "karplus_strong_pitch", f, sr, tolerance=EXACT, note=f"{sr} / (N + 1/2), N = {n}"))
    for f, sr in table:
        cases.append(case(detune, "karplus_strong_detune_cents", f, sr))
    return cases


def fm() -> list[dict]:
    s = "synth:FM-SIDEBANDS"
    spectrum = {"rel": 0, "abs": 1e-12}
    return [
        case("synth:FM-MATCHES-REFERENCE", "fm_tone", 1000.0, 1400.0, 2.0, 0.005, 8000),
        case("synth:FM-MATCHES-REFERENCE", "fm_tone", 440.0, 110.0, 5.0, 0.004, 22050),
        case(s, "fm_spectrum", 1000, 1400, 2.0, 6, 1.0, 44100, tolerance=spectrum, note="the bell's ratio, index 2"),
        case(s, "fm_spectrum", 1000, 1400, 10.0, 15, 1.0, 44100, tolerance=spectrum, note="the bell's first index"),
        case(
            s,
            "fm_spectrum",
            440,
            440,
            1.0,
            4,
            1.0,
            44100,
            tolerance=spectrum,
            note="fm = fc: sideband -2 reflects onto the carrier, so it reads |J0 - J-2|",
        ),
        case(s, "fm_spectrum", 1000, 700, 3.0, 8, 1.0, 8000, tolerance=spectrum, note="sidebands above 4 kHz alias"),
    ]


def bar_modes() -> list[dict]:
    r = "synth:BAR-ROOTS-IN-CODE"
    close = {"rel": 1e-15, "abs": 0}
    return [
        case(r, "free_bar_roots", 8, tolerance=close, note="certified by interval arithmetic in generate.py"),
        case(r, "free_bar_roots", 1, tolerance=close),
        case(r, "free_bar_roots", 0, tolerance=close),
        case(r, "free_bar_ratios", 4, tolerance=close, note="1 : 2.7565 : 5.4039 : 8.9330"),
        case(r, "free_bar_ratios", 8, tolerance=close),
    ]


def errors() -> list[dict]:
    return [
        error("modal_chime", 440.0, 0.0, 1.0, 8000, note="dur must be > 0 (lifesound.py fails on np.max of nothing)"),
        error("modal_chime", -440.0, 0.1, 1.0, 8000, note="freq must be > 0"),
        error("modal_chime", 440.0, 0.1, 1.0, 0, note="sr must be >= 1"),
        error("fm_bell", 440.0, -1.0, 1.4, 10.0, 8000, note="a negative duration"),
        error("fm_bell", 0.0, 1.0, 1.4, 10.0, 8000, note="fc must be > 0"),
        error("fm_tone", 1000.0, 1400.0, 2.0, -0.1, 8000, note="a negative duration"),
        error("fm_tone", 0.0, 1400.0, 2.0, 0.1, 8000, note="fc must be > 0"),
        error("fm_tone", 1000.0, 0.0, 2.0, 0.1, 8000, note="fm must be > 0"),
        error("fm_spectrum", -1000, 1400, 2.0, 3, 1.0, 8000, note="fc must be > 0"),
        error("fm_spectrum", 1000, 1400, 2.0, 3, 0.0, 8000, note="no samples to measure"),
        error("fm_spectrum", 1000, 1400, 2.0, 2**27 + 1, 1.0, 8000, note="n_max above MAX_SAMPLES / 2"),
        error("karplus_strong", 440.0, 0.1, 8000, 1.5, 0, None, note="decay above 1 would make the string grow"),
        error("karplus_strong", 440.0, 0.1, 8000, -0.1, 0, None, note="decay must be in [0, 1]"),
        error("karplus_strong", 0.0, 0.1, 8000, 0.9, 0, None, note="freq must be > 0"),
        error("karplus_strong", 1e-9, 0.1, 44100, 0.9, 0, None, note="a period longer than MAX_SAMPLES"),
        error("karplus_strong", 440.0, 0.1, 8000, 0.9, -1, None, note="a negative seed"),
        error("karplus_strong", 440.0, 0.1, 8000, 0.9, 0, 0.0, note="pluck_brightness_hz must be > 0 (or null)"),
        error("karplus_strong", 440.0, -0.1, 8000, 0.9, 0, None, note="a negative duration"),
        error("karplus_strong_tuned", 4001.0, 0.1, 8000, 0.9, 0, None, note="freq must be at most sr / 2"),
        error("karplus_strong_tuned", 440.0, 0.1, 8000, 1.01, 0, None, note="decay must be in [0, 1]"),
        error("karplus_strong_pitch", 440.0, 0, note="sr must be >= 1"),
        error("karplus_strong_pitch", -1.0, 44100, note="freq must be > 0"),
        error("karplus_strong_detune_cents", 0.0, 44100, note="freq must be > 0"),
        error("soft_voice", 0.0, 0.1, 8000, 5.0, 12, note="freq must be > 0"),
        error("soft_voice", 220.0, -0.1, 8000, 5.0, 12, note="a negative duration"),
        error("heartbeat", 0.0, 1.0, 1000, 48.0, note="bpm must be > 0"),
        error("heartbeat", 60001.0, 1.0, 1000, 48.0, note="more than one beat per sample"),
        error("heartbeat", 60.0, 1.0, 1000, 0.0, note="f0 must be > 0"),
        error("heartbeat", 60.0, 1.0, -1000, 48.0, note="a negative sample rate"),
        error("free_bar_roots", 10001, note="count must be at most 10000"),
        error("free_bar_ratios", -1, note="a negative count"),
    ]


if __name__ == "__main__":
    write("instruments", "modal_chime, fm_bell, soft_voice and heartbeat against lifesound.py.", instruments())
    write(
        "strings",
        "karplus_strong (lifesound.py with SplitMix64 noise), karplus_strong_tuned, and the pitch sr / (N + 1/2).",
        strings(),
    )
    write("fm", "fm_tone against its formula; fm_spectrum against the Bessel amplitudes J_n(I) (mpmath).", fm())
    write("bar_modes", "free_bar_roots and free_bar_ratios against roots certified with mpmath.iv.", bar_modes())
    write("errors", "Invalid input is rejected with an error in every binding.", errors())
    print("certified roots:", [mp.nstr(r.mid, 20) for r in ROOTS[:4]])
    print("certified ratios:", [mp.nstr(r.mid, 12) for r in RATIOS[:4]])
