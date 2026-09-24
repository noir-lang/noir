# Lean soundness proofs for ACIR integer gadgets

Machine-checked soundness of the constraints `AcirContext` emits for Euclidean
division (with and without a predicate), truncation and comparison
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`), correctness of the
SSA `expand_signed_math` emits for signed `lt`, and soundness of whole
functions as ACIR generation compiles them, both before and after the ACVM
optimization passes, plus a pin that keeps those proofs attached to the Rust
code.

## What you must review, and what you can ignore

The layout is the review policy, and `scripts/check.sh` enforces it in CI.

| Path | Status | Why |
|---|---|---|
| `AcirLean/Spec/` | **REVIEWED** | What an ACIR constraint and an SSA instruction mean, the golden printer, and the claims. Nothing checks these against intent. |
| `Check.lean`, `EmitTemplates.lean` | **REVIEWED** | The two entry points: the final check, and the golden-file writer. |
| `scripts/check.sh`, `.github/workflows/fv-lean.yml` | **REVIEWED** | The enforcement itself. |
| `compiler/.../acir_context/fv_templates.rs` | **REVIEWED** | The Rust half of the pin. |
| `AcirLean/Templates/` | pinned, ignore | Constraint lists and SSA, checked byte-for-byte against the Rust output. Plain definitions only. |
| `AcirLean/Proofs/` | machine-checked, ignore | Lean checks every proof, and nothing here can change what `Spec/` states. |
| `AcirLean/Examples/` | ignore | Demonstrations, not part of the claims. |

The whole promise is one proposition, `AllClaims` in `Spec/Claims.lean`.
`Check.lean` requires a proof of exactly that proposition, using only Lean's
three standard axioms. `check.sh` additionally fails if:

- `Spec/` imports anything outside `Spec/`, `Templates/` and Mathlib;
- `Templates/` contains anything but plain definitions, or imports anything
  beyond `Templates/`, `Spec/Semantics.lean`, `Spec/Ssa.lean` and Mathlib;
- any file uses `sorry`, `admit`, `axiom`, `native_decide`, `unsafe`,
  `implemented_by`, `@[extern` or a kernel-check bypass;
- `templates.golden` differs from what `Spec/Pin.lean` prints.

### Reading the reviewed Lean

The reviewed files are about 375 lines, mostly comments. The notation you need:

| Lean | Meaning |
|---|---|
| `σ : ℕ → F` | a witness assignment: witness index to field value |
| `x.val` | the integer value of field element `x`, in `[0, p)` |
| `∀ σ, A → B → C` | for every witness assignment, if `A` and `B` then `C` |
| `∃ σ, A ∧ B` | some witness assignment satisfies both `A` and `B` |
| `a / b`, `a % b` on `.val` | integer division and remainder |
| `∀ n ∈ pinnedWidths, …` | for `n` = 8, 16, 32, 64 and 128 |

`AllClaims` reads, for every pinned width:

1. any witness that satisfies the division constraints, with `a` and `b` of
   that width, has `q = a / b` and `r = a % b`, both with a constant predicate
   and with a predicate witness that is on;
2. any witness that satisfies the truncation constraints has `r = x mod 2^k`;
3. any witness that satisfies the comparison constraints, with `a` and `b` of
   that width, has `q = [a >= b]`;
4. each of those constraint lists has a satisfying witness, so the
   assumptions in 1–3 are not contradictory;
5. the SSA `expand_signed_math` produces for `lt` on `i<n>` returns `1` exactly
   when the first operand is less than the second as signed integers;
6. whole functions, as ACIR generation compiles them, enforce their
   parameters' types and return their SSA meaning for every satisfying
   witness, with no assumption on the inputs: `div` and `lt` on `u<n>`, a
   field truncated to `u<n>`, and signed `lt` on `i<n>` compiled end to end
   after `expand_signed_math`;
7. the same holds for the optimized circuits `nargo compile` ships
   (`acvm::compiler::optimize`: the general simplifier, the redundant-range
   optimizer and common-subexpression merging).

## What is proved

The claims cover the pinned widths (8, 16, 32, 64 and 128 bits). Most proofs
in `Proofs/` hold for every width: `divVarT_sound` and `divPredT_sound` for
`n ≤ 126`, `truncT_sound` for `2 ≤ k ≤ 125`, `moreThanEqT_sound` for
`1 ≤ m ≤ 128`, and `signedLtT_correct` for `n ≥ 1`; 128-bit division and
truncation take different branches and have their own proofs.
`Examples/Bug7895.lean` shows that truncation without the `q ≤ q0` bound
accepts a forged witness, and `Examples/RangeOptimizerBug.lean` shows the same
for a range optimizer that drops a parameter's range check.

The whole-function claims are proved by composition: `Templates/Programs.lean`
builds each compiled function from the gadget templates placed at new witness
indices (`Cstr.rename`), the pin checks that this is exactly what ACIR
generation emits, and `Proofs/Programs.lean` applies each gadget's theorem at
its new indices and chains the results.

Not yet covered:

- arbitrary programs: the whole-function claims cover the pinned functions
  only. A claim for every straight-line program needs a Lean model of how ACIR
  generation compiles each SSA instruction, and the pin can only compare that
  model with the Rust on a fixed corpus of programs;
- the ACVM optimization passes on programs outside the pinned corpus: the
  optimized circuits are checked program by program, not the passes in
  general;
- constant-divisor `div` on its own, signed `div` and `mod`, bitwise
  operations on more than one bit, and completeness.

Trusted: the Lean kernel and its three standard axioms, plus the reviewed
files above.

## How the proofs stay attached to the Rust

`Templates/` states each gadget's constraints and the pinned SSA as data, and
`Spec/Pin.lean` prints them: constraints in a canonical text form, SSA in the
syntax `Ssa`'s `Display` uses. Two checks meet at
`templates.golden`:

1. `scripts/check.sh` fails unless the Lean printout equals `templates.golden`.
2. `fv_templates.rs` (`cargo test -p noirc_evaluator --lib fv_templates`) runs
   the real gadgets, `expand_signed_math`, ACIR generation and the ACVM
   optimizer, and fails unless their output, printed the same way, equals
   `templates.golden`.

Together: the Rust emits exactly the constraint lists and SSA `AllClaims` is about. A
change to a pinned gadget fails the Rust test; making it pass means changing
`Templates/` to the new output and regenerating the golden
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
