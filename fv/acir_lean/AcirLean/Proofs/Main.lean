/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Extended
import AcirLean.Proofs.Satisfiable
import AcirLean.Proofs.Signed
import AcirLean.Proofs.Programs
import AcirLean.Proofs.Shipped
import AcirLean.Proofs.SignedDivMod
import AcirLean.Proofs.Checker
import AcirLean.Proofs.Checker2Sound

/-! The proof of `AllClaims`, assembled from the gadget theorems. -/

namespace AcirLean

theorem allClaims : AllClaims := by
  refine ⟨fun n hn => ⟨?_, divVarGadget_satisfiable hn⟩, fun n hn => ⟨?_, divPredGadget_satisfiable hn⟩,
    fun k hk => ⟨?_, truncateGadget_satisfiable hk⟩, fun m hm => ⟨?_, moreThanEqGadget_satisfiable hm⟩,
    fun n hn => signedLtSsa_correct (by rcases pinned_cases hn with h | h <;> omega),
    fun n hn => ⟨acirGenDiv_sound hn, (acir_satisfiable hn).1⟩,
    fun n hn => ⟨acirGenLt_sound hn, (acir_satisfiable hn).2.1⟩,
    fun n hn => ⟨acirGenTruncate_sound hn, (acir_satisfiable hn).2.2.1⟩,
    fun n hn => ⟨acirGenSignedLt_sound hn, (acir_satisfiable hn).2.2.2⟩,
    fun n hn => ⟨(shipped_sound hn).1, (shipped_satisfiable hn).1⟩,
    fun n hn => ⟨(shipped_sound hn).2.1, (shipped_satisfiable hn).2.1⟩,
    fun n hn => ⟨(shipped_sound hn).2.2.1, (shipped_satisfiable hn).2.2.1⟩,
    fun n hn => ⟨(shipped_sound hn).2.2.2, (shipped_satisfiable hn).2.2.2⟩,
    fun n hn => ⟨shippedSignedDiv_sound hn, (signed_satisfiable hn).1⟩,
    fun n hn => ⟨shippedSignedMod_sound hn, (signed_satisfiable hn).2⟩,
    corpus_claims, testPrograms_claims⟩
  · rcases pinned_cases hn with h | rfl
    · exact fun σ h' hin => divVarGadget_sound (by omega) σ h' (hin (1, n) (by simp))
    · exact fun σ h' hin => divVarGadget128_sound σ h' (hin (1, 128) (by simp))
  · rcases pinned_cases hn with h | rfl
    · exact fun σ h' hin => divPredGadget_sound (by omega) σ h' (hin (1, n) (by simp))
    · exact fun σ h' hin => divPredGadget128_sound σ h' (hin (1, 128) (by simp))
  · rcases pinned_cases hk with h | rfl
    · exact fun σ h' _ => truncateGadget_sound (by omega) (by omega) σ h'
    · exact fun σ h' _ => truncateGadget128_sound σ h'
  · rcases pinned_cases hm with h | h
    · exact fun σ h' hin => moreThanEqGadget_sound (by omega) (by omega) σ h' (hin (0, m) (by simp))
        (hin (1, m) (by simp))
    · exact fun σ h' hin => moreThanEqGadget_sound (by omega) (by omega) σ h' (hin (0, m) (by simp))
        (hin (1, m) (by simp))

end AcirLean
