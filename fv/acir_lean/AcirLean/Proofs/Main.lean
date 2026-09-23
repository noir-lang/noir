/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Templates
import AcirLean.Proofs.Satisfiable

/-! The proof of `AllClaims`, assembled from the gadget theorems. -/

namespace AcirLean

theorem allClaims : AllClaims := by
  refine ⟨fun n hn => ⟨?_, ?_⟩, fun k hk => ⟨?_, ?_⟩⟩
  · have : n ≤ 126 := by simp [pinnedWidths] at hn; omega
    exact fun σ h hin => divVarT_sound this σ h (hin (1, n) (by simp))
  · exact divVarT_satisfiable n (by simp [pinnedWidths] at hn; omega)
  · exact fun σ h _ => truncT_sound (by simp [pinnedWidths] at hk; omega)
      (by simp [pinnedWidths] at hk; omega) σ h
  · simp only [pinnedWidths, List.mem_cons, List.not_mem_nil, or_false] at hk
    rcases hk with rfl | rfl | rfl | rfl
    · exact truncT_satisfiable_8
    · exact truncT_satisfiable_16
    · exact truncT_satisfiable_32
    · exact truncT_satisfiable_64

end AcirLean
