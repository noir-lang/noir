# bound-constraint-bits

**Origin:** `38bb8a455b7` — "fix: correct bit size assumption for bound constraint" (#11654)

**Label: W in principle, no trigger found — excluded from scoring.**

**Defect.** `bound_constraint_with_offset` proves `lhs <= rhs` by range-constraining `rhs - lhs` to
`bits` bits. That argument only holds while `2^bits - 1 < p - 2^bits`, i.e. `bits + 1 < log2(p)`.
The assertion guarding it used `bits < F::max_num_bits()`, one bit too permissive, so a comparison
compiled at `bits = 253` would emit constraints that accept both `lhs <= rhs` and `lhs > rhs`.

**Patch.** Restores the weaker assertion.

**Status.** No Noir program found that reaches `bound_constraint_with_offset` with `bits = 253`; the
whole `acir` test suite still passes with the patch applied. Without a trigger there is nothing to
run the tool on, so this entry is kept for documentation and left out of the score. If a trigger is
found later it becomes a W entry.
