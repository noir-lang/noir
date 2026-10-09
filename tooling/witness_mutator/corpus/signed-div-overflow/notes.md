# signed-div-overflow (control)

**Origin:** `72c5842a449` — "fix: check for signed division overflow" (#9857). The fix landed in
`acir_context`; signed division has since moved to the `expand_signed_math` SSA pass, so the defect
is ported by hand rather than reverted.

**Label: A** — the circuit accepts an input it should reject; the witness stays unique.

**Defect.** `i8::MIN / -1` is `128`, which does not fit in `i8`. Without the explicit overflow
constraint the circuit accepts it and returns `-128`.

**Patch.** Removes the `insert_constrain(min_overflow, zero, ...)` call from
`ExpandSignedMath::insert_div_or_mod`.

**Trigger.** `program/`: `x / y` on `i8`, with `x = -128`, `y = -1`.

**Verified:** on clean `master` `nargo execute` fails with "Attempt to divide with overflow"; with
the patch applied it succeeds and returns `-128`. Clear `program/target/` between runs.

**Expected oracle:** `acir_vs_brillig`, not the witness mutator.
