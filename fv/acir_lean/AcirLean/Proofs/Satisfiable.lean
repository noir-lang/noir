/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Spec.Claims
import AcirLean.Proofs.Basic

/-! Honest witnesses for every pinned constraint list, checked by evaluation. -/

namespace AcirLean

instance instDecidableCstrSatProofs (σ : ℕ → F) (c : Opcode) : Decidable (c.Holds σ) := by
  cases c <;> unfold Opcode.Holds <;> unfold Range <;> infer_instance

instance instDecidableAllSat (σ : ℕ → F) (cs : List Opcode) : Decidable (AllHold σ cs) := by
  unfold AllHold; infer_instance

instance instDecidableInputsFit (σ : ℕ → F) (inputs : List (ℕ × ℕ)) :
    Decidable (InputsFit σ inputs) := by
  unfold InputsFit; infer_instance

/-- `0 / 1 = 0 rem 0`; at `u128`, `1 = 2^64 · 0 + 1` for the divisor split. -/
def divWitness : ℕ → F
  | 1 | 2 | 9 => 1
  | _ => 0

/-- `0 / 1` with the predicate on: `z = 1/b = 1`, `[b == 0] = 0`. -/
def divPredWitness : ℕ → F
  | 1 | 2 | 3 | 13 => 1
  | 11 => ((2 ^ 64 : ℕ) : F)
  | 14 => ((2 ^ 64 + 1 : ℕ) : F)
  | _ => 0

/-- `x = 2^k · q0` truncates to `0`, with the quotient at its bound `q0`. -/
def truncWitness (k : ℕ) : ℕ → F :=
  if k = 128 then fun
    | 0 => ((2 ^ 128 * q0 128 : ℕ) : F)
    | 1 => ((q0 128 : ℕ) : F)
    | 3 => ((2 ^ 128 - 1 : ℕ) : F)
    | 4 => ((q0 128 + Rq 128 : ℕ) : F)
    | 6 => 1
    | 7 => ((R' 128 : ℕ) : F)
    | _ => 0
  else fun
    | 0 => ((2 ^ k * q0 k : ℕ) : F)
    | 1 => ((q0 k : ℕ) : F)
    | 5 => 1
    | 6 => ((R' k : ℕ) : F)
    | _ => 0

/-- `0 >= 0`: `2^m + 0 - 0 = 2^m · 1 + 0`. -/
def geWitness : ℕ → F
  | 2 => 1
  | 4 => ((2 ^ 128 - 1 : ℕ) : F)
  | _ => 0

theorem divVarGadget_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    Satisfiable (divVarGadget n) [(0, n), (1, n)] := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> exact ⟨divWitness, by decide +kernel⟩

theorem divPredGadget_satisfiable {n : ℕ} (hn : n ∈ pinnedWidths) :
    Satisfiable (divPredGadget n) [(0, n), (1, n)] := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn
  rcases hn with rfl | rfl | rfl | rfl | rfl <;> exact ⟨divPredWitness, by decide +kernel⟩

theorem truncateGadget_satisfiable {k : ℕ} (hk : k ∈ pinnedWidths) : Satisfiable (truncateGadget k) [] := by
  refine ⟨truncWitness k, ?_⟩
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hk
  rcases hk with rfl | rfl | rfl | rfl | rfl <;> decide +kernel

theorem moreThanEqGadget_satisfiable {m : ℕ} (hm : m ∈ pinnedWidths) :
    Satisfiable (moreThanEqGadget m) [(0, m), (1, m)] := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hm
  rcases hm with rfl | rfl | rfl | rfl | rfl <;> exact ⟨geWitness, by decide +kernel⟩

end AcirLean
