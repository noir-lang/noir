# syn-drop-radix-limb-range (synthetic)

**Origin:** none — a deliberate mutation, not a historical bug.

**Label: W** — a second witness exists for the same inputs.

**Defect.** Drops the per-limb range constraints in `GeneratedAcir::radix_le_decompose`, leaving
only the recomposition constraint `input == sum(limb_i * radix^i)`. Any limb vector that recomposes
to the same field element is then accepted, including ones with limbs outside `[0, radix)`.

**Patch.** Removes `self.range_constraint(*limb_witness, bit_size)?` from the limb loop.

**Trigger.** `program/`: `x.to_be_bytes()` into `[u8; 31]` (31 limbs, so the honest decomposition is
unique on clean `master`), on 20 random inputs below `2^248` from seed 20260922.

**Verified:** the compiled circuit drops from 64 to 33 ACIR opcodes with the patch applied — the 31
range checks.

**Why it is in the corpus.** A third hint (`ToLeBytes`) and a third alias shape: shift one limb up
by one and its neighbour down by `radix`. Neither the dev target's wraparound rule nor the
`(q-1, r+b)` shift finds it directly.

**Expected oracle:** the witness mutator. `acir_vs_brillig` cannot see it.
