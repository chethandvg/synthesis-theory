/-
SPDX-License-Identifier: Apache-2.0
SPDX-FileCopyrightText: 2026 Chethan Dandebala Venkatarama
-/
import Mathlib.Analysis.SpecialFunctions.Trigonometric.Complex

/-!
# Beats

Two sine tones of nearly equal frequency, played together, swell and fade: they
*beat*. The sum-to-product identity explains why:

  `sin a + sin b = 2 · sin((a + b)/2) · cos((a − b)/2)`  (`sin_add_sin`),

so two tones of frequencies `f₁` and `f₂` add up to a tone at the mean frequency
`(f₁ + f₂)/2` whose amplitude follows the envelope `|2 · cos(π(f₁ − f₂)t)|`
(`beats`). That envelope repeats every `1/|f₁ − f₂|` seconds and falls silent
exactly at `t = (k + 1/2)/(f₁ − f₂)`, once per period (`beat_rate`): the beat
rate is `|f₁ − f₂|` per second.

These are statements about exact real numbers. They describe the physical sum of
two waves (the monaural beat) only; they say nothing about how the ear hears it.
-/

namespace SynthesisTheory

open Real

/-- **Sum to product.** `sin a + sin b = 2 · sin((a + b)/2) · cos((a − b)/2)` for all real `a`, `b`. -/
theorem sin_add_sin (a b : ℝ) :
    sin a + sin b = 2 * sin ((a + b) / 2) * cos ((a - b) / 2) := by
  have h₁ := sin_add ((a + b) / 2) ((a - b) / 2)
  have h₂ := sin_sub ((a + b) / 2) ((a - b) / 2)
  rw [show (a + b) / 2 + (a - b) / 2 = a by ring] at h₁
  rw [show (a + b) / 2 - (a - b) / 2 = b by ring] at h₂
  rw [h₁, h₂]
  ring

/-- The beat envelope of two tones of frequencies `f₁` and `f₂` at time `t`: `|2 · cos(π(f₁ − f₂)t)|`. -/
noncomputable def beatEnvelope (f₁ f₂ t : ℝ) : ℝ :=
  |2 * cos (π * (f₁ - f₂) * t)|

/-- **Beats.** Two tones of frequencies `f₁` and `f₂` add up to a tone at their mean frequency,
`sin(π(f₁ + f₂)t)`, times `2 · cos(π(f₁ − f₂)t)`, whose size is the beat envelope. -/
theorem beats (f₁ f₂ t : ℝ) :
    sin (2 * π * f₁ * t) + sin (2 * π * f₂ * t) =
        2 * cos (π * (f₁ - f₂) * t) * sin (π * (f₁ + f₂) * t) ∧
      |sin (2 * π * f₁ * t) + sin (2 * π * f₂ * t)| =
        beatEnvelope f₁ f₂ t * |sin (π * (f₁ + f₂) * t)| := by
  have h : sin (2 * π * f₁ * t) + sin (2 * π * f₂ * t) =
      2 * cos (π * (f₁ - f₂) * t) * sin (π * (f₁ + f₂) * t) := by
    rw [sin_add_sin,
      show (2 * π * f₁ * t + 2 * π * f₂ * t) / 2 = π * (f₁ + f₂) * t by ring,
      show (2 * π * f₁ * t - 2 * π * f₂ * t) / 2 = π * (f₁ - f₂) * t by ring]
    ring
  refine ⟨h, ?_⟩
  rw [h, beatEnvelope, abs_mul]

/-- **The beat rate is `|f₁ − f₂|`.** For `f₁ ≠ f₂` the envelope repeats every `1/|f₁ − f₂|`
seconds, and it is zero exactly at the times `t = (k + 1/2)/(f₁ − f₂)` for integers `k`:
one silence per period. -/
theorem beat_rate {f₁ f₂ : ℝ} (h : f₁ ≠ f₂) :
    (∀ t, beatEnvelope f₁ f₂ (t + 1 / |f₁ - f₂|) = beatEnvelope f₁ f₂ t) ∧
      ∀ t, beatEnvelope f₁ f₂ t = 0 ↔ ∃ k : ℤ, t = (k + 1 / 2) / (f₁ - f₂) := by
  have hd : f₁ - f₂ ≠ 0 := sub_ne_zero.mpr h
  refine ⟨fun t => ?_, fun t => ?_⟩
  · unfold beatEnvelope
    rcases lt_or_gt_of_ne hd with hneg | hpos
    · rw [abs_of_neg hneg,
        show π * (f₁ - f₂) * (t + 1 / -(f₁ - f₂)) = π * (f₁ - f₂) * t - π by
          field_simp
          ring,
        cos_sub_pi, mul_neg, abs_neg]
    · rw [abs_of_pos hpos,
        show π * (f₁ - f₂) * (t + 1 / (f₁ - f₂)) = π * (f₁ - f₂) * t + π by
          field_simp,
        cos_add_pi, mul_neg, abs_neg]
  · unfold beatEnvelope
    rw [abs_eq_zero, mul_eq_zero, or_iff_right (two_ne_zero), cos_eq_zero_iff]
    constructor
    · rintro ⟨k, hk⟩
      refine ⟨k, ?_⟩
      rw [eq_div_iff hd]
      apply mul_left_cancel₀ pi_ne_zero
      linear_combination hk
    · rintro ⟨k, rfl⟩
      refine ⟨k, ?_⟩
      field_simp

end SynthesisTheory
