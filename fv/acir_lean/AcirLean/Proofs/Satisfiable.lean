/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Spec.Claims
import AcirLean.Proofs.Basic

/-! Honest witnesses for every pinned constraint list. -/

namespace AcirLean

instance instDecidableCstrSatProofs (σ : ℕ → F) (c : Cstr) : Decidable (c.sat σ) := by
  cases c <;> unfold Cstr.sat <;> unfold Range <;> infer_instance

/-- `0 / 1 = 0 rem 0`. -/
theorem divVarT_satisfiable (n : ℕ) (hn : 1 ≤ n) :
    Satisfiable (divVarT n) [(0, n), (1, n)] := by
  refine ⟨fun i => if i = 1 ∨ i = 2 then 1 else 0, ?_, ?_⟩
  · intro c hc
    simp only [divVarT, List.mem_cons, List.not_mem_nil, or_false] at hc
    rcases hc with h | h | h | h | h | h | h | h <;> subst h <;>
      simp [Cstr.sat, Term.eval, Range]
  · intro iw hiw
    simp only [List.mem_cons, List.not_mem_nil, or_false] at hiw
    rcases hiw with h | h <;> subst h <;> simp [ZMod.val_one, Nat.one_lt_two_pow_iff]; omega

/-- `x = 2^k · q0` truncates to `0`, with the quotient at its bound `q0`. -/
def truncWitness (k : ℕ) : ℕ → F
  | 0 => ((2 ^ k * q0 k : ℕ) : F)
  | 1 => ((q0 k : ℕ) : F)
  | 5 => 1
  | 6 => ((R' k : ℕ) : F)
  | _ => 0

theorem truncT_satisfiable_8 : Satisfiable (truncT 8) [] := by
  refine ⟨truncWitness 8, ?_, by simp [InputsFit]⟩
  intro c hc
  simp only [truncT, List.mem_cons, List.not_mem_nil, or_false] at hc
  rcases hc with h | h | h | h | h | h | h | h | h | h | h | h <;> subst h <;> decide +kernel

theorem truncT_satisfiable_16 : Satisfiable (truncT 16) [] := by
  refine ⟨truncWitness 16, ?_, by simp [InputsFit]⟩
  intro c hc
  simp only [truncT, List.mem_cons, List.not_mem_nil, or_false] at hc
  rcases hc with h | h | h | h | h | h | h | h | h | h | h | h <;> subst h <;> decide +kernel

theorem truncT_satisfiable_32 : Satisfiable (truncT 32) [] := by
  refine ⟨truncWitness 32, ?_, by simp [InputsFit]⟩
  intro c hc
  simp only [truncT, List.mem_cons, List.not_mem_nil, or_false] at hc
  rcases hc with h | h | h | h | h | h | h | h | h | h | h | h <;> subst h <;> decide +kernel

theorem truncT_satisfiable_64 : Satisfiable (truncT 64) [] := by
  refine ⟨truncWitness 64, ?_, by simp [InputsFit]⟩
  intro c hc
  simp only [truncT, List.mem_cons, List.not_mem_nil, or_false] at hc
  rcases hc with h | h | h | h | h | h | h | h | h | h | h | h <;> subst h <;> decide +kernel

end AcirLean
