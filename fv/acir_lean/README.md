# Lean soundness proofs for ACIR integer gadgets

Machine-checked soundness of the constraints `AcirContext` emits for Euclidean
division (with and without a predicate), truncation and comparison
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`), correctness of the
SSA `expand_signed_math` emits for signed `lt`, and soundness of whole
functions as ACIR generation compiles them, both before and after the ACVM
optimization passes, plus a pin that keeps those proofs attached to the Rust
code. A checker proved sound once also covers 124 real programs from
`test_programs/execution_success`, as `nargo compile` ships them.

## What you must review, and what you can ignore

The layout is the review policy, and `scripts/check.sh` enforces it in CI.

| Path | Status | Why |
|---|---|---|
| `AcirLean/Spec/` | **REVIEWED** | What an ACIR constraint and an SSA instruction mean, the golden printer, and the claims. Nothing checks these against intent. |
| `Check.lean`, `EmitTemplates.lean`, `EmitPrograms.lean` | **REVIEWED** | The entry points: the final check, and the two golden-file writers. |
| `scripts/check.sh`, `.github/workflows/fv-lean.yml` | **REVIEWED** | The enforcement itself. |
| `compiler/.../acir_context/fv_templates.rs` | **REVIEWED** | The Rust half of the pin. |
| `scripts/regen_programs.sh`, `.github/workflows/fv-test-programs.yml` | **REVIEWED** | Rebuild `test_programs.golden` from `nargo compile` output; CI fails if the committed copy is stale. |
| `AcirLean/Templates/` | pinned, ignore | Constraint lists, SSA and test programs, checked byte-for-byte against the golden files. Plain definitions only. |
| `AcirLean/Proofs/` | machine-checked, ignore | Lean checks every proof, and nothing here can change what `Spec/` states. |
| `AcirLean/Examples/` | ignore | Demonstrations, not part of the claims. |

The whole promise is one proposition, `AllClaims` in `Spec/Claims.lean`.
`Check.lean` requires a proof of exactly that proposition, using only Lean's
three standard axioms. `check.sh` additionally fails if:

- `Spec/` imports anything outside `Spec/`, `Templates/` and Mathlib;
- `Templates/` contains anything but plain definitions, or imports anything
  beyond `Templates/`, `Spec/Semantics.lean`, `Spec/Ssa.lean`,
  `Spec/Programs.lean`, `Spec/Programs2.lean` and Mathlib;
- any file uses `sorry`, `admit`, `axiom`, `native_decide`, `unsafe`,
  `implemented_by`, `@[extern` or a kernel-check bypass;
- `templates.golden` or `test_programs.golden` differs from what
  `Spec/Pin.lean` prints.

### Reading the reviewed Lean

The reviewed Lean is about 750 lines, mostly comments. The notation you need:

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
   optimizer and common-subexpression merging);
8. signed `div` and `mod` on `i8`–`i64`, as shipped (`expand_signed_math`,
   ACIR generation and the optimizer), accept only a nonzero divisor and not
   `MIN / -1`, and return the truncating signed quotient or remainder
   (`Int.tdiv`, `Int.tmod`) in two's complement.

9. every program in the corpus (`Templates/Corpus.lean`: straight-line
   programs of `div` and `lt` on `u8`, `u64` and `u128`), as shipped,
   enforces its parameters' types and returns what the program computes, and
   the witness ACVM solved for it satisfies its circuit;
10. every scalar program from `test_programs/execution_success` in
    `Templates/TestPrograms.lean`, except the three in `uncoveredPrograms`, is
    implemented by the circuit `nargo compile` ships for it: for every
    witness satisfying the circuit, the inputs fit their parameter types, the
    final SSA runs without failing on them (no overflow, no zero divisor, no
    failed `constrain` or `range_check`), and the circuit's return witnesses
    hold what it returns (`ProgSpec2` in `Spec/Programs2.lean`).

The `eq` gadget these use is sound only because the BN254 scalar field modulus
is prime; `Proofs/Prime.lean` proves that with a Pratt certificate.

### The checker

Claim 9 is not proved program by program. `Proofs/Checker.lean` defines
`checkProg P C`, which walks program `P`'s instructions, tracks which witness
(or `1 - w`) holds each value, and requires each instruction's proved gadget
template, placed on a block of fresh witnesses, to appear among circuit `C`'s
constraints (in a canonical form proved to preserve meaning). `checkProg_sound`
proves once that acceptance implies `SoundFn C (ProgSpec P)`; the corpus claim
is then one evaluation of `checkProg` over the corpus. Adding programs to the
corpus needs no new proof, only programs the checker accepts.

The checker only accepts circuits it can prove sound. Rebuilding the corpus
with the `r < b` constraint removed from `euclidean_division_var`, it accepts
4 of the 66 circuits: the lone `lt` programs at widths 8 and 64, where that
constraint was a repeat of a range check already present.

### The checker for real programs

Claim 10 comes from a second checker, `checkProg2` in `Proofs/Checker2.lean`,
proved sound once in `Proofs/Checker2Sound.lean`. It takes the final SSA of a
program whose `main` is one block of scalar instructions (`add`, `sub`, `mul`,
`div`, `mod`, `lt`, `eq`, `not`, `cast`, `truncate`, `constrain`,
`range_check`, checked or unchecked, over `Field`, `u<n>` and `i<n>`) and the
optimized circuit `nargo compile` ships. For each SSA value it keeps some
polynomials over the circuit's witnesses that evaluate to it, and bounds on its
integer value. Each instruction is accepted by a local rule instead of a fixed
template, because the optimizer merges, reorders and drops constraints:

- arithmetic with no possible overflow, from the operands' bounds, stays a
  polynomial; otherwise a witness equal to the result must be range-checked
  below `2^n` (for `sub`, below `p - b`'s bound, so a wrapped result fails it);
- `div`, `mod`, `lt` and `truncate` need witnesses `q`, `r` with a constraint
  `a = b q + r` (or `2^m + a - b = 2^m q + r`), bounds on `q` and `r` that rule
  out wraparound, and `r < b`; truncating a `Field` also needs the `q ≤ p / 2^k`
  bound and, when `q` equals it, the remainder bound — the checks
  `Examples/Bug7895.lean` is about;
- `eq` needs the inverse gadget, `constrain` a constraint that equates both
  sides, and a return value a constraint that equates it with its witness;
- a range check may be missing when the circuit fixes the witness to a
  constant, since the optimizer drops such range checks as implied.

Which witnesses play which role is found by untrusted searches, and every
candidate is checked against the circuit before a rule uses it. Of the 544
execution-success programs that `nargo compile` builds, 127 are in the scalar
subset (the rest use arrays, references, several ACIR functions, calls, black
boxes or several blocks; the reason for each is in `test_programs.outside`). The checker accepts 124 of them. The
three it does not are listed in `uncoveredPrograms` in `Spec/Claims.lean`
with the reason. Removing any single constraint from the 124 circuits makes
the checker reject in 378 of 385 cases; the other 7 constraints are
redundant (a repeated constraint, a range check implied by another bound, and
`b · inv = 1` in a division that already proves `r < b`).

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

- programs outside the scalar subset: arrays and ACIR memory, references,
  calls, black boxes and control flow in the checker;
- the ACVM optimization passes on programs outside the pinned corpus: the
  optimized circuits are checked program by program, not the passes in
  general;
- bitwise operations on more than one bit, and completeness (an honest witness
  exists) for the test programs.

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

After a change to the corpus programs in `fv_templates.rs`, regenerate the
Lean data from the Rust output (the converter is untrusted; the pin checks its
output):

```sh
cargo test -p noirc_evaluator --lib fv_templates 2>&1 | sed -n '/--- emitted ---/,/^note:/p' | sed '1d;$d' | sed '$d' > /tmp/emitted.txt
(cd fv/acir_lean && ./scripts/gen_corpus.py /tmp/emitted.txt > AcirLean/Templates/Corpus.lean)
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

To rebuild the test-program data from the current compiler (about 6 minutes:
it builds `nargo`, compiles every execution-success program, and prints each
shipped circuit through `fv_templates.rs`), then re-check the proofs:

```sh
just fv-regen       # the programs already proved: fixes a failing `FV test programs` job
just fv-regen-all   # also adds test programs that are not in the proofs yet
just fv-check       # only re-check the proofs (what `FV Lean` runs)
```

Adding a test program needs none of these: CI only checks the programs already
in the proofs. `just fv-regen-all` brings new ones in; if the checker rejects
one, `fv-check` fails until it is listed, with the reason, in
`uncoveredPrograms` (`Spec/Claims.lean`).

This writes `Templates/TestPrograms.lean`, `test_programs.golden` and
`test_programs.outside`. Two CI checks keep them honest:

- `FV test programs` (`.github/workflows/fv-test-programs.yml`) rebuilds the
  programs already in the proofs on every pull request, in parallel with the
  other workflows, and fails if any of their SSA or circuits changed. A compiler
  change that alters one of them therefore has to commit the rebuilt data
  (`just fv-regen`).
- `FV Lean` (`check.sh`) requires the Lean data to print exactly
  `test_programs.golden`, and the checker to accept every program outside
  `uncoveredPrograms`, so rebuilt data with a circuit the checker cannot prove
  sound fails there.

To see a soundness bug caught, delete the `q ≤ q0` bound in
`euclidean_division_var` (the `bound_constraint_with_offset(quotient_var,
q0_var, …)` call) and run the Rust test: it fails, and `AcirLean/Examples/Bug7895.lean` shows
why the Lean side cannot be updated to match.
