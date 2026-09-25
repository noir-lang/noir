/-
EXAMPLE: not part of the claims, no review needed. Lean checks it.
-/

import AcirLean.Templates.Gadgets
import AcirLean.Proofs.Basic

/-!
`truncateGadget` with the `q ≤ q0` bound removed (the bound fixed in noir-lang/noir#7895).
Witnesses: `0 = x, 1 = q, 2 = r, 3 = z, 4 = y, 5 = u`. These constraints accept
a wrong remainder (`buggy_truncation_forgery`), so no soundness proof for them
exists (`buggy_truncation_not_sound`): the `q ≤ q0` bound is load-bearing.
-/

namespace AcirLean

def truncBugT (k : ℕ) : List Constraint :=
  [ .range 1 (254 - k), .range 2 k, .range 1 (254 - k), .range 2 k, .range 2 k,
    .zero [⟨1, [0]⟩, ⟨-(2 ^ k : ℕ), [1]⟩, ⟨-1, [2]⟩],
    .zero [⟨1, []⟩, ⟨1, [1, 3]⟩, ⟨-(q0 k : ℤ), [3]⟩, ⟨-1, [4]⟩],
    .zero [⟨1, [1, 4]⟩, ⟨-(q0 k : ℤ), [4]⟩],
    .zero [⟨1, [2, 4]⟩, ⟨(R' k : ℤ), [4]⟩, ⟨-1, [5]⟩],
    if N' k = 0 then .zero [⟨1, [5]⟩] else .range 5 (N' k) ]

instance instDecidableCstrSatExample (σ : ℕ → F) (c : Constraint) : Decidable (c.Holds σ) := by
  cases c <;> unfold Constraint.Holds <;> unfold Range <;> infer_instance

/-- The forged assignment for `x as u64`: pick `q = q0 + 1`, so `2^64 * q`
wraps past `p` and lands on `x = 2^64 - p % 2^64` with remainder `r = 0`. -/
def forged : ℕ → F
  | 0 => ((2 ^ 64 - p % 2 ^ 64 : ℕ) : F)   -- x
  | 1 => ((p / 2 ^ 64 + 1 : ℕ) : F)        -- q  (one more than q0)
  | 2 => 0                                  -- r  (claimed x mod 2^64)
  | 3 => -1                                 -- z
  | _ => 0                                  -- y = 0, u = 0

theorem buggy_truncation_forgery :
    AllHold forged (truncBugT 64) ∧
      (forged 2).val ≠ (forged 0).val % 2 ^ 64 := by
  constructor
  · intro c hc
    simp only [truncBugT, List.mem_cons, List.not_mem_nil, or_false] at hc
    rcases hc with h | h | h | h | h | h | h | h | h | h <;> subst h <;> decide +kernel
  · decide +kernel

end AcirLean

namespace AcirLean

/-- The soundness statement `truncateGadget_sound` makes for the fixed gadget is false
for the buggy one, so no proof of it can exist. -/
theorem buggy_truncation_not_sound :
    ¬ ∀ σ : ℕ → F, AllHold σ (truncBugT 64) → (σ 2).val = (σ 0).val % 2 ^ 64 :=
  fun h => buggy_truncation_forgery.2 (h forged buggy_truncation_forgery.1)

end AcirLean
