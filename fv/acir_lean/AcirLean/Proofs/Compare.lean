/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Division

/-!
`AcirContext::more_than_eq_var(a, b, m)`: divide `2^m + a - b` by the constant
`2^m` at bit size `m + 1` and return the quotient.
-/

namespace AcirLean

theorem more_than_eq_sound {m : ℕ} (hm : 1 ≤ m) (hm128 : m ≤ 128) {a b q r : F}
    (ha : a.val < 2 ^ m) (hb : b.val < 2 ^ m)
    (c : DivConstConstraints (m + 1) (2 ^ m) (a - b + ((2 ^ m : ℕ) : F)) q r) :
    q.val = if b.val ≤ a.val then 1 else 0 := by
  have hbits : bits (2 ^ m) = m + 1 := Nat.size_pow
  have hc2 : 2 ≤ 2 ^ m := by
    calc 2 = 2 ^ 1 := by norm_num
      _ ≤ 2 ^ m := Nat.pow_le_pow_right (by norm_num) hm
  have h := div_const_sound hc2 (by rw [hbits]; omega) c
  have hp := p_gt
  have h129 : 2 ^ (m + 1) ≤ 2 ^ 129 := Nat.pow_le_pow_right (by norm_num) (by omega)
  have hpow : 2 ^ (m + 1) = 2 * 2 ^ m := by ring
  have h129' : (2:ℕ) ^ 129 < 2 ^ 253 := by norm_num
  -- the dividend's value is 2^m + a - b as a natural number
  have hv : (a - b + ((2 ^ m : ℕ) : F)).val = 2 ^ m + a.val - b.val := by
    have hn : 2 ^ m + a.val - b.val < p := by omega
    apply eq_nat_of_eq_cast hn
    rw [Nat.cast_sub (by omega), Nat.cast_add, ZMod.natCast_zmod_val,
      ZMod.natCast_zmod_val]; ring
  rw [h.1, hv]
  split_ifs with hab
  · exact Nat.div_eq_of_lt_le (by omega) (by omega)
  · rw [Nat.div_eq_zero_iff]; omega

end AcirLean
