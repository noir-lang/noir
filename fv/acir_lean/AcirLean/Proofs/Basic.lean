/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Templates.Gadgets
import Mathlib.Tactic

/-! Arithmetic facts about `p` and the range-check lemmas the gadget proofs use. -/

namespace AcirLean

theorem p_gt : 2 ^ 253 < p := by norm_num [p]
theorem p_lt : p < 2 ^ 254 := by norm_num [p]

theorem val_natCast_of_lt {n : ℕ} (h : n < p) : ((n : ℕ) : F).val = n := by
  rw [ZMod.val_natCast, Nat.mod_eq_of_lt h]

theorem pow_lt_p {k : ℕ} (hk : k ≤ 253) : 2 ^ k < p :=
  lt_of_le_of_lt (Nat.pow_le_pow_right (by norm_num) hk) p_gt

/-- `a = b` over `F` with both sides' natural representatives below `p`. -/
theorem eq_nat_of_eq_cast {a : F} {n : ℕ} (hn : n < p) (h : a = (n : F)) : a.val = n := by
  rw [h, val_natCast_of_lt hn]

/-- The general path of `bound_constraint_with_offset` with offset 1:
`Range (y - (x + 1)) k` together with `Range x k` forces `x < y`
whenever `k + 1 < 254`. -/
theorem lt_of_bound {x y : F} {k : ℕ} (hk : k ≤ 252)
    (hx : x.val < 2 ^ k) (h : (y - (x + 1)).val < 2 ^ k) : x.val < y.val := by
  by_contra hxy
  push Not at hxy
  have hk1 : 2 ^ (k + 1) < p := pow_lt_p (by omega)
  have hpow : 2 ^ (k + 1) = 2 * 2 ^ k := by ring
  have hx1 : (x + 1).val = x.val + 1 := by
    rw [ZMod.val_add_of_lt] <;> simp [ZMod.val_one]; omega
  have hsub : (x + 1 - y).val = x.val + 1 - y.val := by
    rw [ZMod.val_sub (by omega), hx1]
  have hne : x + 1 - y ≠ 0 := by
    intro h0; rw [h0, ZMod.val_zero] at hsub; omega
  have : y - (x + 1) = -(x + 1 - y) := by ring
  rw [this, ZMod.neg_val, if_neg hne, hsub] at h
  have := ZMod.val_lt y
  omega

/-- Same inequality, general path with offset 0 (`x ≤ y`). -/
theorem le_of_bound0 {x y : F} {k : ℕ} (hk : k ≤ 252)
    (hx : x.val < 2 ^ k) (h : (y - x).val < 2 ^ k) : x.val ≤ y.val := by
  by_contra hxy
  push Not at hxy
  have hk1 : 2 ^ (k + 1) < p := pow_lt_p (by omega)
  have hpow : 2 ^ (k + 1) = 2 * 2 ^ k := by ring
  have hsub : (x - y).val = x.val - y.val := ZMod.val_sub (by omega)
  have hne : x - y ≠ 0 := by
    intro h0; rw [h0, ZMod.val_zero] at hsub; omega
  have : y - x = -(x - y) := by ring
  rw [this, ZMod.neg_val, if_neg hne, hsub] at h
  have := ZMod.val_lt y
  omega

/-- The constant path of `bound_constraint_with_offset` (offset 1, constant
`c ≥ 1`): `Range (x + (2^N - c)) N` with `N = bits (c - 1)` forces `x < c`,
given `x < 2^N` and `N ≤ 252`. -/
theorem lt_of_bound_const {x : F} {c N : ℕ} (hc : 1 ≤ c) (hN : N = bits (c - 1))
    (hN252 : N ≤ 252) (hx : x.val < 2 ^ N)
    (h : (x + ((2 ^ N - c : ℕ) : F)).val < 2 ^ N) : x.val < c := by
  have hcN : c - 1 < 2 ^ N := by rw [hN]; exact Nat.lt_size_self _
  have hk1 : 2 ^ (N + 1) < p := pow_lt_p (by omega)
  have hpow : 2 ^ (N + 1) = 2 * 2 ^ N := by ring
  have hR : ((2 ^ N - c : ℕ) : F).val = 2 ^ N - c := val_natCast_of_lt (by omega)
  rw [ZMod.val_add_of_lt (by omega), hR] at h
  omega

/-- `bits (2^k - 1) = k`: the all-ones `k`-bit number. -/
theorem bits_pow_sub_one {k : ℕ} (hk : 1 ≤ k) : bits (2 ^ k - 1) = k := by
  have h2 : 2 ^ (k - 1) * 2 = 2 ^ k := by rw [← pow_succ]; congr 1; omega
  have h1 := Nat.one_le_two_pow (n := k - 1)
  have hle : bits (2 ^ k - 1) ≤ k := Nat.size_le.2 (by omega)
  have hlt : k - 1 < bits (2 ^ k - 1) := Nat.lt_size.2 (by omega)
  omega

end AcirLean
