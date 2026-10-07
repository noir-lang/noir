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
import AcirLean.Proofs.EqNotFieldDiv
import AcirLean.Proofs.Shift
import AcirLean.Proofs.Shl
import AcirLean.Proofs.Bitwise

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
    corpus_claims,
    fun n hn => ⟨acirGenEq_sound hn, (eqNotFieldDiv_satisfiable hn).1⟩,
    fun n hn => ⟨acirGenNot_sound hn, (eqNotFieldDiv_satisfiable hn).2.1⟩,
    ⟨acirGenFieldDiv_sound, (eqNotFieldDiv_satisfiable (n := 8) (by decide)).2.2⟩,
    fun n hn => shr_claims hn, fun n hn => shl_claims hn,
    fun n hn => by
      obtain ⟨s1, s2, s3, s4, s5⟩ := bitwise_satisfiable hn
      have a1 := acirGenBitwise_sound (n := n) false
      have a2 := acirGenBitwise_sound (n := n) true
      have a3 := shippedBitwise_sound (n := n) false
      have a4 := shippedBitwise_sound (n := n) true
      simp only [Bool.false_eq_true, if_false, if_true] at a1 a2 a3 a4
      exact ⟨a1, s1, a3, s3, a2, s2, a4, s4, acirGenOr_sound hn, s5⟩⟩
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
