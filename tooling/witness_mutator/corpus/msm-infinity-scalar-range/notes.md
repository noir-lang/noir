# msm-infinity-scalar-range (control)

**Origin:** `235e1a7c800` — "fix(ssa): retain MSM scalar limb range checks for constant infinity
points" (#12885)

**Label: A** — the circuit accepts an input it should reject; the witness stays unique.

**Defect.** Simplifying a multi-scalar multiplication against a constant point at infinity dropped
the range checks on the scalar limbs, so an out-of-range scalar limb was accepted.

**Patch.** The source hunks of the fix, reverse-applied (they still apply cleanly).

**Trigger.** `test_programs/execution_failure/msm_infinity_scalar_out_of_range` (added by the fix),
with `hi = 2^128`.

**Verified:** on clean `master` `nargo execute` fails with "Limb ... is not less than 2^128"; with
the patch applied it succeeds. Clear the program's `target/` directory between runs — `nargo`
caches the compiled artifact and will otherwise reuse a circuit built with the other source tree.

**Expected oracle:** `acir_vs_brillig`, not the witness mutator. A miss by the mutator is the
correct outcome and must not be scored as a failure.
