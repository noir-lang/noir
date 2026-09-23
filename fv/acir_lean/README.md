# Lean soundness proofs for ACIR integer gadgets

Machine-checked soundness of the constraints `AcirContext` emits for Euclidean
division, truncation and comparison
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`), for every bit
width, plus a pin that keeps those proofs attached to the Rust code.

## What is proved

The prover chooses the Brillig outputs (quotient, remainder, inverses) freely,
so every theorem quantifies over them. Each theorem's hypotheses are exactly
the constraints the gadget emits, plus the operand-width premise its callers
guarantee.

| Theorem | Gadget path | Conclusion |
|---|---|---|
| `div_var_sound` | `euclidean_division_var`, non-constant divisor, `n ≤ 126`, predicate 1 | `q = a / b`, `r = a % b` |
| `div_var128_sound` | same, `n = 128` (upper-halves guard) | same |
| `div_const_sound` | constant divisor, no overflow branch | `q = a / c`, `r = a % c` |
| `div_const_overflow_sound`, `truncate_field_sound` | constant divisor with the `q ≤ p / c` guard; `truncate_var(x, k, 254)` for `2 ≤ k ≤ 125` | `r = x % 2^k` |
| `more_than_eq_sound` | `more_than_eq_var`, `m ≤ 128` | `q = [a ≥ b]` |
| `divVarT_sound`, `truncT_sound` | the pinned constraint lists (`Template.lean`) | as above |
| `buggy_truncation_not_sound` | `truncT` without the `q ≤ q0` bound | a kernel-checked forged witness: that bound is necessary |

Not yet covered: a witness predicate (division under an `if`), constant
divisors with `p / c < 2^128` (e.g. `Field` to `u128`), and signed operations,
which are lowered in SSA (`remove_bit_shifts`) rather than here.

Trusted: the Lean kernel and its three standard axioms, ACIR semantics
(`RANGE k` means `val < 2^k`, `AssertZero` means `= 0` over BN254 `Fr`), and
the caller-side operand ranges. `scripts/check.sh` rejects `sorry`, `admit`,
`axiom` and `native_decide`.

## How the proofs stay attached to the Rust

`Template.lean` states each gadget's constraints as data, proves soundness
over that data, and prints it in a canonical text form. Two checks meet at
`templates.golden`:

1. `scripts/check.sh` builds every proof and fails unless the Lean printout
   equals `templates.golden`.
2. `fv_templates.rs` (`cargo test -p noirc_evaluator --lib fv_templates`) runs
   the real gadgets and fails unless their constraints, printed the same way,
   equal `templates.golden`.

Together: the Rust emits exactly the constraints the theorems are about. A
change to a pinned gadget fails the Rust test; making it pass means changing
`Template.lean` to the new constraints and regenerating the golden file from
Lean, and that only builds if the proofs still go through. An unsound change
cannot be re-proved (see `Bug7895.lean`).

## Running locally

Install Lean via elan (the toolchain version comes from `lean-toolchain`):

```sh
curl -sSfL https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y --default-toolchain none
```

Then, from the repository root:

```sh
(cd fv/acir_lean && ./scripts/check.sh)                  # ~2 min first run (Mathlib cache download), ~20 s after
cargo test -p noirc_evaluator --lib fv_templates         # Rust side of the pin
```

After an intentional change to a pinned gadget:

```sh
cd fv/acir_lean
# edit AcirLean/Template.lean to the new constraints, fix the proofs, then:
lake env lean --run EmitTemplates.lean templates.golden
./scripts/check.sh
```

Never edit `templates.golden` by hand: `check.sh` regenerates it from Lean and
fails on any difference.

To see a soundness bug caught, delete the `q ≤ q0` bound in
`euclidean_division_var` (the `bound_constraint_with_offset(quotient_var,
q0_var, …)` call) and run the Rust test: it fails, and `Bug7895.lean` shows
why the Lean side cannot be updated to match.
