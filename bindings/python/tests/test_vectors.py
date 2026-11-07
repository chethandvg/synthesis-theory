# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama
"""The shared golden vectors, checked against the Python bindings."""

from pathlib import Path

import numpy as np
import pytest
import synthesis_theory as st
from evidence_kernel.vectors import check_dir

VECTORS = Path(__file__).resolve().parents[3] / "vectors"


def by_name(module):
    """Every public function of `module`, under its own name, returning numpy arrays as lists."""

    def wrap(fn):
        return lambda *args: (lambda r: r.tolist() if isinstance(r, np.ndarray) else r)(fn(*args))

    return {name: wrap(fn) for name, fn in vars(module).items() if callable(fn) and not name.startswith("_")}


def test_golden_vectors():
    assert check_dir(VECTORS, by_name(st)) > 0


def test_defaults_and_names_follow_lifesound():
    assert np.array_equal(st.modal_chime(440.0, sr=8000), st.modal_chime(440.0, 3.0, 1.0, 8000))
    assert np.array_equal(st.fm_bell(440.0, 0.1), st.fm_bell(440.0, 0.1, 1.4, 10.0, st.SR))
    assert np.array_equal(st.karplus_strong(440.0, 0.1), st.karplus_strong(440.0, 0.1, st.SR, 0.996, 0, 2000.0))
    assert np.array_equal(
        st.karplus_strong(440.0, 0.1, pluck_brightness_hz=None), st.karplus_strong(440.0, 0.1, 44100, 0.996, 0, None)
    )
    assert np.array_equal(st.soft_voice(220.0, 0.1), st.soft_voice(220.0, 0.1, st.SR, 5.0, 12))
    assert np.array_equal(st.heartbeat(72.0, 1.0), st.heartbeat(72.0, 1.0, st.SR, 48.0))
    assert np.array_equal(st.fm_spectrum(1000.0, 1400.0, 2.0, 3), st.fm_spectrum(1000.0, 1400.0, 2.0, 3, 1.0, 44100))
    assert st.BAR_MODES == [1.0, 2.756, 5.404, 8.933] and st.SR == 44100


def test_results_and_errors():
    x = st.karplus_strong(440.0, 0.01)
    assert isinstance(x, np.ndarray) and x.dtype == np.float64 and x.shape == (441,)
    assert st.karplus_strong_pitch(440.0) == 44100 / 100.5
    with pytest.raises(ValueError, match="decay must be in"):
        st.karplus_strong(440.0, 0.1, 44100, 1.5)
    with pytest.raises(ValueError, match="sr must be at least 1 sample per second"):
        st.modal_chime(440.0, 0.1, 1.0, 0)
    with pytest.raises(OverflowError):
        st.karplus_strong(440.0, 0.1, 44100, 0.9, -1)
    with pytest.raises(TypeError):
        st.free_bar_roots(1.5)
