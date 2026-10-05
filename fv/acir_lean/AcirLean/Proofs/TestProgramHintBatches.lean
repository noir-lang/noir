/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.TestProgramHintCerts
import AcirLean.Templates.TestPrograms
import AcirLean.Spec.Claims

/-!
# The hint checker on the test programs

`checkProgH` accepts every test program with its certificate from
`testProgramHintCerts`, except those in `hintOpenPrograms`. Nothing here is a
claim yet: `checkProgH` has no soundness proof. The kernel checks one batch of
20 programs per theorem, as `testPrograms_claims` does.

Nothing imports this module, so CI does not build it: all eleven batches take
longer than the `FV Lean` job's 30 minutes. Build it by hand with
`lake build AcirLean.Proofs.TestProgramHintBatches`.
-/

namespace AcirLean

/-- The test programs the hint checker does not prove yet. -/
def hintOpenPrograms : List String :=
  ["regression_10008", "regression_8519", "regression_9971", "signed_inactive_division_by_zero"]

def hintProgramOK (ec : TestProgram × (List Entry × List (Option (ℕ × Comb)))) : Bool :=
  decide (ec.1.name ∈ hintOpenPrograms) || checkProgH ec.1.prog ec.1.fn ec.2.1 ec.2.2

abbrev hintProgramOK.batch (k : ℕ) : Prop :=
  (((testPrograms.zip testProgramHintCerts).drop (20 * k)).take 20).all hintProgramOK = true

set_option maxHeartbeats 0 in
theorem hintPrograms_batch0 : hintProgramOK.batch 0 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch1 : hintProgramOK.batch 1 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch2 : hintProgramOK.batch 2 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch3 : hintProgramOK.batch 3 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch4 : hintProgramOK.batch 4 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch5 : hintProgramOK.batch 5 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch6 : hintProgramOK.batch 6 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch7 : hintProgramOK.batch 7 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch8 : hintProgramOK.batch 8 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch9 : hintProgramOK.batch 9 := by decide +kernel

set_option maxHeartbeats 0 in
theorem hintPrograms_batch10 : hintProgramOK.batch 10 := by decide +kernel

end AcirLean
