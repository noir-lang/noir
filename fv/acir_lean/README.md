# Lean soundness proofs for ACIR integer gadgets

Machine-checked soundness of the constraints `AcirContext` emits for Euclidean
division, truncation and comparison
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`), for every bit
width, plus a pin that keeps those proofs attached to the Rust code.

## What you must review, and what you can ignore

The layout is the review policy. `scripts/check.sh` enforces it in CI, and
`.github/CODEOWNERS` routes changes to the reviewed files to their owners.

| Path | Status | Why |
|---|---|---|
| `AcirLean/Spec/` | **REVIEWED** | What a constraint means, the golden printer, and the claims. Nothing checks these against intent. |
| `Check.lean`, `EmitTemplates.lean` | **REVIEWED** | The two entry points: the final check, and the golden-file writer. |
| `scripts/check.sh`, `.github/workflows/fv-lean.yml` | **REVIEWED** | The enforcement itself. |
| `compiler/.../acir_context/fv_templates.rs` | **REVIEWED** | The Rust half of the pin. |
| `AcirLean/Templates/` | pinned, ignore | Constraint lists, checked byte-for-byte against the Rust output. Plain definitions only. |
| `AcirLean/Proofs/` | machine-checked, ignore | Lean checks every proof, and nothing here can change what `Spec/` states. |
| `AcirLean/Examples/` | ignore | Demonstrations, not part of the claims. |

The whole promise is one proposition, `AllClaims` in `Spec/Claims.lean`.
`Check.lean` requires a proof of exactly that proposition, using only Lean's
three standard axioms. `check.sh` additionally fails if:

- `Spec/` imports anything outside `Spec/`, `Templates/` and Mathlib;
- `Templates/` contains anything but plain definitions, or imports anything
  beyond `Spec/Semantics.lean` and Mathlib;
- any file uses `sorry`, `admit`, `axiom`, `native_decide`, `unsafe`,
  `implemented_by`, `@[extern` or a kernel-check bypass;
- `templates.golden` differs from what `Spec/Pin.lean` prints.

### Reading the reviewed Lean

The reviewed files are about 180 lines, mostly comments. The notation you need:

| Lean | Meaning |
|---|---|
| `σ : ℕ → F` | a witness assignment: witness index to field value |
| `x.val` | the integer value of field element `x`, in `[0, p)` |
| `∀ σ, A → B → C` | for every witness assignment, if `A` and `B` then `C` |
| `∃ σ, A ∧ B` | some witness assignment satisfies both `A` and `B` |
| `a / b`, `a % b` on `.val` | integer division and remainder |
| `∀ n ∈ pinnedWidths, …` | for `n` = 8, 16, 32 and 64 |

`AllClaims` reads: for every pinned width, (1) any witness that satisfies the
division constraints, with `a` and `b` of that width, has `q = a / b` and
`r = a % b`; (2) any witness that satisfies the truncation constraints has
`r = x mod 2^k`; and (3) each constraint list has at least one satisfying
witness, so the assumptions in (1) and (2) are not contradictory.

## What is proved

The claims cover the pinned widths (8, 16, 32 and 64 bits). The proofs in
`Proofs/` hold for every width (`divVarT_sound` for `n ≤ 126`, `truncT_sound`
for `2 ≤ k ≤ 125`); pinning another width extends the claims to it.

`Proofs/` also proves soundness of u128 division (`div_var128_sound`), the
no-overflow constant divisor (`div_const_sound`) and comparison
(`more_than_eq_sound`). Those are not pinned yet, so they are not claims.
`Examples/Bug7895.lean` shows that truncation without the `q ≤ q0` bound
accepts a forged witness.

Not yet covered: a witness predicate (division under an `if`), constant
divisors with `p / c < 2^128` (e.g. `Field` to `u128`), and signed operations,
which are lowered in SSA (`remove_bit_shifts`) rather than here.

Trusted: the Lean kernel and its three standard axioms, plus the reviewed
files above.

## How the proofs stay attached to the Rust

`Templates/Gadgets.lean` states each gadget's constraints as data, and
`Spec/Pin.lean` prints them in a canonical text form. Two checks meet at
`templates.golden`:

1. `scripts/check.sh` fails unless the Lean printout equals `templates.golden`.
2. `fv_templates.rs` (`cargo test -p noirc_evaluator --lib fv_templates`) runs
   the real gadgets and fails unless their constraints, printed the same way,
   equal `templates.golden`.

Together: the Rust emits exactly the constraint lists `AllClaims` is about. A
change to a pinned gadget fails the Rust test; making it pass means changing
`Templates/Gadgets.lean` to the new constraints and regenerating the golden
file from Lean, and that only passes `Check.lean` if `AllClaims` is still
provable. An unsound change cannot be (see `Examples/Bug7895.lean`).

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
# edit AcirLean/Templates/Gadgets.lean to the new constraints, fix AcirLean/Proofs/, then:
lake env lean --run EmitTemplates.lean templates.golden
./scripts/check.sh
```

Never edit `templates.golden` by hand: `check.sh` regenerates it from Lean and
fails on any difference.

To see a soundness bug caught, delete the `q ≤ q0` bound in
`euclidean_division_var` (the `bound_constraint_with_offset(quotient_var,
q0_var, …)` call) and run the Rust test: it fails, and `AcirLean/Examples/Bug7895.lean` shows
why the Lean side cannot be updated to match.
