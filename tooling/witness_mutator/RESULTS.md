# Score against the bug corpus

The tool was developed against one entry, `div-quotient-bound`, and against the requirement that a
clean compiler produce no `HIGH` finding. It was then frozen at commit `9116422e09`, and only after
that run against the rest of the corpus. None of the held-out entries influenced a single line of
the tool.

## The table

| entry | label | mutator | which strategy found it | `acir_vs_brillig` oracle |
| --- | --- | --- | --- | --- |
| `div-quotient-bound` (development) | W | **3 / 20 inputs** | `quotient_over_modulus+1` | blind: honest execution is unchanged |
| `syn-drop-remainder-bound` | W | **11 / 20 inputs** | `honest-1+derived` | blind |
| `syn-drop-radix-limb-range` | W | **20 / 20 inputs** | `honest+1+derived` | blind |
| `div-remainder-bits` | W-candidate, expected verdict unknown | not found, 0 / 21 | — | blind |
| `signed-div-overflow` | A | not found — correct | — | **catches it**: ACIR exits 0, forced Brillig exits 1 |
| `msm-infinity-scalar-range` | A | not found — correct | — | **catches it**: ACIR exits 0, forced Brillig exits 1 |
| `bound-constraint-bits` | W in principle | excluded | — | — no program reaches the defect |

Both oracles are needed and neither subsumes the other. The three W entries are invisible to a
differential execution check, because an honest prover's run is byte-for-byte unchanged — the bug
only shows up when a prover chooses different hint outputs. The two A entries are invisible to the
mutator, because the witness stays unique — the circuit simply accepts an input it should have
rejected.

## Does it generalize beyond the entry it was built on?

Yes, on the evidence available. The two synthetic entries were caught by the **generic** strategy,
not by the hint-aware one, and neither resembles the development target:

- `syn-drop-remainder-bound` drops the `r < rhs` bound in `u8` division. The alias is a small
  integer shift: lower the quotient by one and let the constraint hand back the remainder. For
  input 0 the tool reports `w4: 6 -> 5` and `w5: 22 -> 49`, and `6·27 + 22 = 5·27 + 49` holds.
- `syn-drop-radix-limb-range` drops the per-limb range checks in byte decomposition. The alias
  moves value between neighbouring limbs, and the finding points at `std/field/mod.nr:175`, inside
  the stdlib's own decomposition. Found on every input.

The hit rate differs sharply between entries, which is a property of the bugs rather than of the
tool. A wraparound alias exists only when the shifted quotient still fits its range check (3 / 20
here); a limb alias exists for essentially every input (20 / 20).

## The one W miss

`div-remainder-bits` reintroduces #10721, whose original exploit is already blocked on today's
`master` by the `q <= q0` bound that #7895 added later. Its notes recorded the expected verdict as
unknown for exactly this reason, before the tool existed.

What the run establishes is that none of the candidate families finds a second witness here. What
it does not establish is whether one exists: by hand, an `r` in `[2^128, 2^129)` is still rejected
by the `r < rhs` bound, which suggests the circuit may remain unique despite the loosened range
check. Settling that needs a solver-based uniqueness check rather than a search, so the entry stays
open rather than being counted either way.

## Reproducing

```sh
git checkout 9116422e09   # the frozen tool
tooling/witness_mutator/run_corpus.sh <entry>
```

`msm-infinity-scalar-range` has no `program/` directory of its own — it uses
`test_programs/execution_failure/msm_infinity_scalar_out_of_range`, and on a clean compiler that
program fails to execute by design, so only the patched side can be searched.
