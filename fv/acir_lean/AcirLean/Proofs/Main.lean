/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Extended
import AcirLean.Proofs.Satisfiable
import AcirLean.Proofs.Signed
import AcirLean.Proofs.Programs

/-! The proof of `AllClaims`, assembled from the gadget theorems. -/

namespace AcirLean

theorem allClaims : AllClaims := by
  refine ⟨fun n hn => ⟨?_, divVarT_satisfiable hn⟩, fun n hn => ⟨?_, divPredT_satisfiable hn⟩,
    fun k hk => ⟨?_, truncT_satisfiable hk⟩, fun m hm => ⟨?_, moreThanEqT_satisfiable hm⟩,
    fun n hn => signedLtT_correct (by rcases pinned_cases hn with h | h <;> omega),
    fun n hn => ⟨acirDivT_sound hn, (acir_satisfiable hn).1⟩,
    fun n hn => ⟨acirLtT_sound hn, (acir_satisfiable hn).2.1⟩,
    fun n hn => ⟨acirTruncT_sound hn, (acir_satisfiable hn).2.2.1⟩,
    fun n hn => ⟨acirSignedLtT_sound hn, (acir_satisfiable hn).2.2.2⟩⟩
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
