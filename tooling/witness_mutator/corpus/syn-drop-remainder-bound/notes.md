# syn-drop-remainder-bound (synthetic)

**Origin:** none — a deliberate mutation, not a historical bug.

**Label: W** — a second witness exists for the same inputs.

**Defect.** Drops the `r < rhs` bound in `euclidean_division_var`, leaving `r` constrained only by
its range check. For `u8` division by a witness, `r` is then merely `r < 256`, so
`(q - 1, r + rhs)` is a second valid witness whenever `r + rhs < 256`.

**Patch.** Removes the `bound_constraint_with_offset(remainder_var, rhs, ...)` call.

**Trigger.** `program/`: `x / y` on `u8`, 20 random pairs with `y != 0` from seed 20260922.

**Verified:** 8 ACIR snapshot tests change with the patch applied. For
`div_u8_no_predicate_by_witness` the removed opcodes are exactly
`ASSERT w6 = w1 - w5 - 1` and `BLACKBOX::RANGE input: w6, bits: 8`.

**Why it is in the corpus.** It exercises a different mechanism from the dev target: the alias is a
small integer shift `(q-1, r+b)` rather than a modulus wraparound, and it needs no knowledge of
what the hint computes. A tool that finds only the dev target and misses this one is overfitted.

**Expected oracle:** the witness mutator. `acir_vs_brillig` cannot see it.
