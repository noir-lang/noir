/-
REVIEWED: trusted entry point. `scripts/check.sh` runs this file; it fails
unless `AcirLean.allClaims` proves exactly `AcirLean.AllClaims` (stated in
`AcirLean/Spec/Claims.lean`) using only Lean's three standard axioms.
-/

import AcirLean.Spec.Claims
import AcirLean.Proofs.Main

example : AcirLean.AllClaims := AcirLean.allClaims

/-- info: 'AcirLean.allClaims' depends on axioms: [propext, Classical.choice, Quot.sound] -/
#guard_msgs in
#print axioms AcirLean.allClaims
