/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Extended
import AcirLean.Proofs.Satisfiable
import AcirLean.Proofs.Signed

/-! The proof of `AllClaims`, assembled from the gadget theorems. -/

namespace AcirLean

theorem pinned_cases {n : ℕ} (hn : n ∈ pinnedWidths) : (8 ≤ n ∧ n ≤ 64) ∨ n = 128 := by
  simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hn
  omega

theorem allClaims : AllClaims := by
  refine ⟨fun n hn => ⟨?_, divVarT_satisfiable hn⟩, fun n hn => ⟨?_, divPredT_satisfiable hn⟩,
    fun k hk => ⟨?_, truncT_satisfiable hk⟩, fun m hm => ⟨?_, moreThanEqT_satisfiable hm⟩,
    fun n hn => signedLtT_correct (by rcases pinned_cases hn with h | h <;> omega)⟩
  · rcases pinned_cases hn with h | rfl
    · exact fun σ h' hin => divVarT_sound (by omega) σ h' (hin (1, n) (by simp))
    · exact fun σ h' hin => divVarT128_sound σ h' (hin (1, 128) (by simp))
  · rcases pinned_cases hn with h | rfl
    · exact fun σ h' hin => divPredT_sound (by omega) σ h' (hin (1, n) (by simp))
    · exact fun σ h' hin => divPredT128_sound σ h' (hin (1, 128) (by simp))
  · rcases pinned_cases hk with h | rfl
    · exact fun σ h' _ => truncT_sound (by omega) (by omega) σ h'
    · exact fun σ h' _ => truncT128_sound σ h'
  · rcases pinned_cases hm with h | h
    · exact fun σ h' hin => moreThanEqT_sound (by omega) (by omega) σ h' (hin (0, m) (by simp))
        (hin (1, m) (by simp))
    · exact fun σ h' hin => moreThanEqT_sound (by omega) (by omega) σ h' (hin (0, m) (by simp))
        (hin (1, m) (by simp))

end AcirLean
