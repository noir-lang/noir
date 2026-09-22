# Underconstraint bug corpus

A set of known-bad compilers, used to measure what a witness-mutation tool actually finds. Each
directory holds a patch that puts one defect into current `master`, a Noir program that reaches the
defective code, and notes recording what the defect is and which oracle is expected to catch it.

## The two labels

**W — a second witness exists.** With the inputs fixed, the constraints admit more than one
solution. A prover can choose a different intermediate value and still satisfy the circuit. This is
what a witness-mutation tool looks for.

**A — the circuit accepts an input it should reject.** A check was dropped, so a program that
should fail to execute now succeeds, but the witness is still unique. There is no second witness to
find, so a witness-mutation tool is *expected to miss these*. They are in the corpus as controls:
they belong to the existing `acir_vs_brillig` fuzzer, which sees honest ACIR execution succeed
where Brillig fails.

Scoring a W miss and an A miss the same way would make the tool look worse than it is, and would
invite tuning it toward bugs it structurally cannot see.

## Contents

| directory | origin | label | expected oracle | reintroduction verified by |
| --- | --- | --- | --- | --- |
| `div-quotient-bound` | `47ba6138f81` (#7895) | W (dev target) | witness mutator | `properly_constrains_quotient_when_truncating_fields` fails; 12 → 10 opcodes |
| `div-remainder-bits` | `a0f05179e5d` (#10721) | W-candidate | witness mutator | `truncate_field_to_128_bits` snapshot widens 128 → 129 bits; historical exploit masked |
| `bound-constraint-bits` | `38bb8a455b7` (#11654) | W in principle | — (excluded) | no trigger program found |
| `syn-drop-remainder-bound` | synthetic | W | witness mutator | 8 ACIR snapshots change; `r < rhs` opcodes gone |
| `syn-drop-radix-limb-range` | synthetic | W | witness mutator | 64 → 33 opcodes (31 range checks gone) |
| `msm-infinity-scalar-range` | `235e1a7c800` (#12885) | A | `acir_vs_brillig` | `nargo execute` goes from failing to succeeding |
| `signed-div-overflow` | `72c5842a449` (#9857), hand-ported | A | `acir_vs_brillig` | `nargo execute` goes from failing to succeeding |

`div-quotient-bound` is the development target. Everything else is held out: the tool is built and
tuned against the development target and against a false-positive run on clean `master`, then
frozen, and only then run over the rest.

## Why two synthetic entries

The historical W bugs found so far cluster in `euclidean_division_var`, and two of the three needed
hand-porting because the surrounding code has been rewritten since. A tool tuned on that one
function could score well while generalizing to nothing. The synthetic entries put the same class of
defect into different constraint shapes: a remainder bound whose alias is a small integer shift
`(q-1, r+b)`, and a radix decomposition whose alias moves value between neighbouring limbs. They are
labelled synthetic wherever they are reported, and they do not replace the historical entries.

## Running one

```sh
git apply tooling/witness_mutator/corpus/<name>/bug.patch
cargo build -p nargo_cli --bin nargo
rm -rf tooling/witness_mutator/corpus/<name>/program/target   # nargo caches compiled artifacts
./target/debug/nargo execute --program-dir tooling/witness_mutator/corpus/<name>/program
git checkout -- compiler acvm-repo
```

Clearing `target/` matters: without it `nargo` reuses the artifact compiled by whichever source tree
ran last, which silently reports the wrong result for both the patched and the clean run.

Programs with an `inputs/` directory carry the 20 inputs the tool should sweep, drawn from
`random.Random(20260922)`. `Prover.toml` holds the first of them so the program runs standalone.
