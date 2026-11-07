/-
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama
-/
import Mathlib.Analysis.SpecialFunctions.Complex.Log
import Mathlib.Analysis.SpecialFunctions.Trigonometric.Basic

/-!
# The Karplus–Strong string

`karplus_strong` in the Rust crate (after `lifesound.py`) feeds a burst of noise
round a loop: a delay of `N` samples followed by the two-point average
`yₙ = decay · (xₙ + xₙ₋₁)/2`. At `ω` radians per sample (a frequency `f` sampled
`sr` times a second has `ω = 2πf / sr`):

* `avgResponse_eq`: the average's response `(1 + e^{-iω})/2` equals
  `e^{-iω/2} · cos(ω/2)`, so it delays every frequency by exactly half a sample
  (while `cos(ω/2) > 0`, that is for `|ω| < π`).
* `loopGain_pos_real_iff`: the gain once round the loop,
  `G(ω) = e^{-iωN} · (1 + e^{-iω})/2`, is a positive real number for `|ω| < π`
  exactly when `ω = 2πk/(N + 1/2)` for an integer `k`. Those are the loop's
  resonances: the string sounds at `f_k = k · sr/(N + 1/2)`, not `k · sr/N`.
* `loopGain_decay`: per trip round the loop a component at `ω` is scaled in size
  by `|decay| · |cos(ω/2)|`, which falls as `ω` rises from 0 to `π`: higher
  partials die faster.

These are statements about exact complex numbers. That the floating-point string
really sounds at `sr/(N + 1/2)` is tested in the Rust crate, not proved here.
-/

namespace SynthesisTheory

open Complex

/-- The frequency response of the two-point average `(xₙ + xₙ₋₁)/2` at `ω` radians per sample. -/
noncomputable def avgResponse (ω : ℝ) : ℂ :=
  (1 + exp (-(ω * I))) / 2

/-- The gain once round the Karplus–Strong loop (a delay of `N` samples, then the two-point
average) at `ω` radians per sample: `G(ω) = e^{-iωN} · (1 + e^{-iω})/2`. -/
noncomputable def loopGain (N : ℕ) (ω : ℝ) : ℂ :=
  exp (-(ω * N * I)) * avgResponse ω

/-- **Half a sample of delay.** `(1 + e^{-iω})/2 = e^{-iω/2} · cos(ω/2)` for every real `ω`. -/
theorem avgResponse_eq (ω : ℝ) :
    avgResponse ω = exp (-((ω / 2 : ℝ) * I)) * (Real.cos (ω / 2) : ℂ) := by
  rw [ofReal_cos, Complex.cos, avgResponse, mul_div_assoc', mul_add, ← Complex.exp_add,
    ← Complex.exp_add]
  congr 1
  rw [show -(((ω / 2 : ℝ) : ℂ) * I) + ((ω / 2 : ℝ) : ℂ) * I = 0 by ring, Complex.exp_zero]
  congr 2
  push_cast
  ring

/-- The loop gain is a pure delay of `N + 1/2` samples times the real number `cos(ω/2)`. -/
theorem loopGain_eq (N : ℕ) (ω : ℝ) :
    loopGain N ω = exp (-((ω * (N + 1 / 2) : ℝ) * I)) * (Real.cos (ω / 2) : ℂ) := by
  rw [loopGain, avgResponse_eq, ← mul_assoc, ← Complex.exp_add]
  congr 2
  push_cast
  ring

/-- **Resonances at `2πk/(N + 1/2)`.** For `|ω| < π`, the loop gain `G(ω)` is a positive real
number exactly when `ω = 2πk/(N + 1/2)` for some integer `k`: the string's partials are at
`f_k = k · sr/(N + 1/2)`. -/
theorem loopGain_pos_real_iff (N : ℕ) {ω : ℝ} (hω : |ω| < Real.pi) :
    (∃ r : ℝ, 0 < r ∧ loopGain N ω = r) ↔ ∃ k : ℤ, ω = 2 * Real.pi * k / (N + 1 / 2) := by
  have hD : (0 : ℝ) < N + 1 / 2 := by positivity
  have hc : 0 < Real.cos (ω / 2) := by
    apply Real.cos_pos_of_mem_Ioo
    constructor <;> linarith [abs_lt.mp hω]
  have hcne : (Real.cos (ω / 2) : ℂ) ≠ 0 := ofReal_ne_zero.mpr hc.ne'
  rw [loopGain_eq]
  constructor
  · rintro ⟨r, hr, hG⟩
    set e := exp (-((ω * (N + 1 / 2) : ℝ) * I)) with he
    have hnorm : ‖e‖ = 1 := by
      rw [he, show -(((ω * (N + 1 / 2) : ℝ) : ℂ) * I) = ((-(ω * (N + 1 / 2)) : ℝ) : ℂ) * I by
        push_cast; ring]
      exact Complex.norm_exp_ofReal_mul_I _
    have he' : e = ((r / Real.cos (ω / 2) : ℝ) : ℂ) := by
      rw [ofReal_div, eq_div_iff hcne, hG]
    have hq : r / Real.cos (ω / 2) = 1 := by
      have := hnorm
      rw [he', Complex.norm_real, Real.norm_eq_abs, abs_of_pos (div_pos hr hc)] at this
      exact this
    rw [hq, ofReal_one, he, Complex.exp_eq_one_iff] at he'
    obtain ⟨n, hn⟩ := he'
    refine ⟨-n, ?_⟩
    have h1 : ((ω * (N + 1 / 2) : ℝ) : ℂ) * I = ((-(2 * Real.pi * n) : ℝ) : ℂ) * I := by
      push_cast at hn ⊢
      linear_combination -hn
    have h2 : ω * (N + 1 / 2) = -(2 * Real.pi * n) :=
      ofReal_injective (mul_right_cancel₀ I_ne_zero h1)
    rw [eq_div_iff hD.ne', h2]
    push_cast
    ring
  · rintro ⟨k, hk⟩
    refine ⟨Real.cos (ω / 2), hc, ?_⟩
    have h : -(((ω * (N + 1 / 2) : ℝ) : ℂ) * I) = ((-k : ℤ) : ℂ) * (2 * Real.pi * I) := by
      rw [hk, div_mul_cancel₀ _ hD.ne']
      push_cast
      ring
    rw [h, Complex.exp_int_mul_two_pi_mul_I, one_mul]

/-- **Higher partials die faster.** Per trip round the loop, with the gain `decay`, a component at
`ω` is scaled in size by `|decay| · |cos(ω/2)|`; for `decay ≠ 0` that factor strictly falls as `ω`
rises from 0 to `π`. -/
theorem loopGain_decay (decay : ℝ) (N : ℕ) :
    (∀ ω : ℝ, ‖(decay : ℂ) * loopGain N ω‖ = |decay| * |Real.cos (ω / 2)|) ∧
      (decay ≠ 0 →
        StrictAntiOn (fun ω : ℝ => ‖(decay : ℂ) * loopGain N ω‖) (Set.Icc 0 Real.pi)) := by
  have hnorm : ∀ ω : ℝ, ‖(decay : ℂ) * loopGain N ω‖ = |decay| * |Real.cos (ω / 2)| := by
    intro ω
    rw [loopGain_eq, norm_mul, norm_mul, Complex.norm_real, Complex.norm_real, Real.norm_eq_abs,
      Real.norm_eq_abs,
      show -(((ω * (N + 1 / 2) : ℝ) : ℂ) * I) = ((-(ω * (N + 1 / 2)) : ℝ) : ℂ) * I by
        push_cast; ring,
      Complex.norm_exp_ofReal_mul_I, one_mul]
  refine ⟨hnorm, fun hd ω₁ h₁ ω₂ h₂ hlt => ?_⟩
  simp only [hnorm]
  have hpos : 0 < |decay| := abs_pos.mpr hd
  have hc₁ : 0 ≤ Real.cos (ω₁ / 2) :=
    Real.cos_nonneg_of_mem_Icc ⟨by linarith [h₁.1, Real.pi_pos], by linarith [h₁.2]⟩
  have hc₂ : 0 ≤ Real.cos (ω₂ / 2) :=
    Real.cos_nonneg_of_mem_Icc ⟨by linarith [h₂.1, Real.pi_pos], by linarith [h₂.2]⟩
  rw [abs_of_nonneg hc₁, abs_of_nonneg hc₂]
  apply mul_lt_mul_of_pos_left _ hpos
  apply Real.cos_lt_cos_of_nonneg_of_le_pi_div_two
  · linarith [h₁.1]
  · linarith [h₂.2]
  · linarith

end SynthesisTheory
