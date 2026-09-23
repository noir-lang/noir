/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Spec.Claims
import Mathlib.Tactic

/-! `expand_signed_math`'s signed `lt` is correct for every width. -/

namespace AcirLean

theorem signedLtT_run (n a b : ℕ) :
    (signedLtT n).run [a, b] =
      (a / 2 ^ (n - 1) ^^^ b / 2 ^ (n - 1)) ^^^ (if a < b then 1 else 0) := by
  simp [SsaFn.run, signedLtT, SsaIns.step, BinOp.eval, Operand.eval, List.lookup]

theorem signedLtT_correct {n : ℕ} (hn : 1 ≤ n) : ComputesSignedLt (signedLtT n) n := by
  intro a b ha hb
  rw [signedLtT_run]
  unfold sint
  have h2 : (2 : ℕ) ^ n = 2 * 2 ^ (n - 1) := by
    rw [← pow_succ']; congr 1; omega
  have hz : (2 : ℤ) ^ n = 2 * 2 ^ (n - 1) := by exact_mod_cast h2
  rw [hz, show (2 : ℤ) ^ (n - 1) = ((2 ^ (n - 1) : ℕ) : ℤ) by push_cast; rfl]
  generalize hN : 2 ^ (n - 1) = N at *
  have hN0 : 0 < N := by rw [← hN]; positivity
  have sign : ∀ x, x < 2 * N → (x < N ∧ x / N = 0) ∨ (N ≤ x ∧ x / N = 1) := by
    intro x hx
    rcases Nat.lt_or_ge x N with h | h
    · exact Or.inl ⟨h, Nat.div_eq_of_lt h⟩
    · exact Or.inr ⟨h, Nat.div_eq_of_lt_le (by omega) (by omega)⟩
  rcases sign a (by omega) with ⟨ha', hda⟩ | ⟨ha', hda⟩ <;>
  rcases sign b (by omega) with ⟨hb', hdb⟩ | ⟨hb', hdb⟩ <;> rw [hda, hdb] <;>
    split_ifs <;> first | rfl | omega
