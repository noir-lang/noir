# div-quotient-bound (dev target)

**Origin:** `47ba6138f81` — "fix: properly constrain quotient during field truncation" (#7895)

**Label: W** — a second witness exists for the same inputs.

**Defect.** Euclidean division by a constant emits a `Quotient` hint producing `(q, r)` with
`a = q*b + r`. When `q*b` can overflow the field, `q` must additionally be bounded by
`q0 = p / b`; without that bound a prover may pick `q' = (a + k*p) / b`, which satisfies the
recomposition constraint because it wraps around the modulus once.

**Patch.** Removes the `bound_constraint_with_offset(quotient_var, q0_var, ...)` call in
`acir_context::euclidean_division_var`.

**Trigger.** `program/`: `x as u64` (truncation of a `Field`), on 20 inputs drawn from
`random.Random(20260922)` over the BN254 field. The inputs are deliberately *not* the crafted
value used by the fix's own test.

Not every input admits a second witness. A `k = 1` alias needs `a + p < 2^254`, i.e. roughly the
lowest third of the field, so a detection rate near 1/3 over these 20 inputs is the expected
result rather than 20/20.

**Verified:** with the patch applied, `cargo test -p noirc_evaluator
properly_constrains_quotient_when_truncating_fields` FAILS (passes on clean master); the compiled
circuit for `program/` drops from 12 to 10 ACIR opcodes.

**Expected oracle:** the witness mutator. `acir_vs_brillig` cannot see it: honest execution is
unchanged.
