# Lean soundness proofs for ACIR integer gadgets

Machine-checked soundness of the constraints `AcirContext` emits for Euclidean
division (with and without a predicate), truncation and comparison
(`compiler/noirc_evaluator/src/acir/acir_context/mod.rs`), correctness of the
SSA `expand_signed_math` emits for signed `lt`, and soundness of whole
functions as ACIR generation compiles them, both before and after the ACVM
optimization passes, plus a pin that keeps those proofs attached to the Rust
code. A checker proved sound once also covers 146 real programs from
`test_programs/execution_success`, as `nargo compile` ships them.

## What you must review, and what you can ignore

The layout is the review policy, and `scripts/check.sh` enforces it in CI.
**[`REVIEWING.md`](REVIEWING.md) walks through every reviewed Lean definition
for a reader who does not know Lean.**

| Path | Status | Why |
|---|---|---|
| `AcirLean/Spec/` | **REVIEWED** | What an ACIR constraint and an SSA instruction mean, the golden printer, and the claims. Nothing checks these against intent. |
| `Check.lean`, `EmitTemplates.lean`, `EmitPrograms.lean`, `EmitSemantics.lean` | **REVIEWED** | The entry points: the final check, and the three golden-file writers. |
| `scripts/check.sh`, `scripts/check_reviewing.py`, `scripts/check_templates.py`, `.github/workflows/fv-lean.yml` | **REVIEWED** | The enforcement itself. |
| `compiler/.../acir_context/fv_templates.rs`, `fv_semantics.rs` | **REVIEWED** | The Rust halves of the pins. |
| `scripts/regen_programs.sh`, `.github/workflows/fv-test-programs.yml` | **REVIEWED** | Rebuild `test_programs.golden` from `nargo compile` output; CI fails if the committed copy is stale. |
| `AcirLean/Templates/` | pinned, ignore | Opcode lists, SSA and test programs, checked byte-for-byte against the golden files. Plain definitions only. |
| `AcirLean/Proofs/` | machine-checked, ignore | Lean checks every proof, and nothing here can change what `Spec/` states. |
| `AcirLean/Examples/` | ignore | Demonstrations, not part of the claims. |

The whole promise is one proposition, `AllClaims` in `Spec/Claims.lean`.
`Check.lean` requires a proof of exactly that proposition, using only Lean's
three standard axioms. `check.sh` additionally fails if:

- `Spec/` imports anything outside `Spec/`, `Templates/` and Mathlib;
- `Templates/` contains anything but plain definitions (`scripts/check_templates.py`:
  no keyword other than `def` and `abbrev` declares anything, and no definition
  has a dotted name such as `Int.tdiv` that could stand in for one the claims
  use), or imports anything beyond `Templates/`, `Spec/Semantics.lean`,
  `Spec/Ssa.lean`, `Spec/Corpus.lean`, `Spec/SsaSemantics.lean` and Mathlib;
- any file, `lakefile.toml` included, uses `sorry`, `admit`, `axiom`,
  `native_decide`, `unsafe`, `implemented_by`, `@[extern` or a kernel-check
  bypass, or `lakefile.toml` passes options to Lean;
- `templates.golden` or `test_programs.golden` differs from what
  `Spec/Pin.lean` prints, or `ssa_semantics.golden` from what
  `EmitSemantics.lean` prints;
- `REVIEWING.md` quotes a line that is no longer in the reviewed Lean, or does
  not mention one of its definitions.

### Reading the reviewed Lean

The reviewed Lean is about 750 lines, mostly comments. [`REVIEWING.md`](REVIEWING.md)
explains it line by line; the short version of the notation:

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
   witness, with no assumption on the inputs: `div` (rejecting a zero
   divisor) and `lt` on `u<n>`, a
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
10. every program from `test_programs/execution_success` in
    `Templates/TestPrograms.lean`, except those in `uncoveredPrograms`, is
    implemented by the circuit `nargo compile` ships for it: for every
    witness satisfying the circuit, the inputs fit their parameter types, the
    final SSA runs without failing on them (no overflow, no zero divisor, no
    failed `constrain` or `range_check`), and the circuit's return witnesses
    hold what it returns (`ProgramSpec` in `Spec/SsaSemantics.lean`); and the
    witness `nargo execute` solves from its `Prover.toml` satisfies that
    circuit, so the claim cannot hold just because the circuit is
    contradictory.

The `eq` gadget these use is sound only because the BN254 scalar field modulus
is prime; `Proofs/Prime.lean` proves that with a Pratt certificate.

### The checker

Claim 9 is not proved program by program. `Proofs/Checker.lean` defines
`checkProg P C`, which walks program `P`'s instructions, tracks which witness
(or `1 - w`) holds each value, and requires each instruction's proved gadget
template, placed on a block of fresh witnesses, to appear among circuit `C`'s
constraints (in a canonical form proved to preserve meaning). `checkProg_sound`
proves once that acceptance implies `SoundFunction C (CorpusSpec P)`; the corpus claim
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

- unchecked `add`, `sub` and `mul` are field arithmetic, as in Noir's SSA
  interpreter: the result stays a polynomial, with bounds only where it cannot
  wrap around `p`;
- checked arithmetic with no possible overflow, from the operands' bounds,
  stays a polynomial; otherwise a witness equal to the result must be
  range-checked below `2^n` (for `sub`, below `p - b`'s bound, so a wrapped
  result fails it);
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
candidate is checked against the circuit before a rule uses it. Of the 560
execution-success programs that `nargo compile` builds, 150 are in the
supported subset: one block of scalar instructions, plus arrays of scalars and
tuples read and written at constant indices. The rest use arrays at dynamic
indices (ACIR memory), references, several ACIR functions, calls, black boxes,
several blocks, or more than 250 instructions; the reason for each is in
`test_programs.outside`. The checker accepts 146 of them. The four it does not
are listed in `uncoveredPrograms` in `Spec/Claims.lean` with the reason.
Removing any single constraint from the 125 circuits without arrays makes the
checker reject in 387 of 394 cases;
the other 7 constraints are
redundant (a repeated constraint, a range check implied by another bound, and
`b · inv = 1` in a division that already proves `r < b`).

## What is proved

The claims cover the pinned widths (8, 16, 32, 64 and 128 bits). Most proofs
in `Proofs/` hold for every width: `divVarGadget_sound` and `divPredGadget_sound` for
`n ≤ 126`, `truncateGadget_sound` for `2 ≤ k ≤ 125`, `moreThanEqGadget_sound` for
`1 ≤ m ≤ 128`, and `signedLtSsa_correct` for `n ≥ 1`; 128-bit division and
truncation take different branches and have their own proofs.
`Examples/Bug7895.lean` shows that truncation without the `q ≤ q0` bound
accepts a forged witness, and `Examples/RangeOptimizerBug.lean` shows the same
for a range optimizer that drops a parameter's range check.

The whole-function claims are proved by composition: `Templates/Programs.lean`
builds each compiled function from the gadget templates placed at new witness
indices (`Opcode.rename`), the pin checks that this is exactly what ACIR
generation emits, and `Proofs/Programs.lean` applies each gadget's theorem at
its new indices and chains the results.

Not yet covered:

- programs outside the supported subset: arrays at dynamic indices (ACIR
  memory), nested arrays, references,
  calls, black boxes and control flow in the checker;
- the ACVM optimization passes on programs outside the pinned corpus: the
  optimized circuits are checked program by program, not the passes in
  general;
- bitwise operations on more than one bit;
- completeness: that every valid input has a witness the circuit accepts.
  What is proved alongside soundness is non-vacuity (`Satisfiable`: some
  witness meets every constraint), which rules out a contradictory circuit
  making a claim trivially true, but not a circuit that rejects some valid
  inputs.

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

## How the SSA meaning stays attached to Noir

`Spec/SsaSemantics.lean` states by hand what each SSA instruction computes, and
every claim about a test program rests on it. A mistake there would make the
proofs prove the wrong thing, so it is tested against Noir's own reference
semantics, the SSA interpreter:

1. `EmitSemantics.lean` runs `Instruction.run` on a fixed grid (every binary
   operation, checked and unchecked, `not`, `cast`, `truncate`, `constrain`
   and `range_check`, on `Field`, `u1`, `u8`…`u128` and `i8`…`i64`, over edge
   values such as `0`, `2^n - 1`, `2^n` and `p - 1`) and writes the results to
   `ssa_semantics.golden`. `check.sh` fails unless the file is current.
2. `fv_semantics.rs` (`cargo test -p noirc_evaluator --lib fv_semantics`) runs
   each of those functions and calls through Noir's SSA parser, validator and
   interpreter, and fails unless every result, including every failure, is the
   same.

The grid leaves out only what never reaches ACIR generation: signed checked
arithmetic and signed `div`, `mod` and `lt` (rewritten by
`expand_signed_math`), and SSA the validator rejects. After a change to
`Instruction.run` or to the interpreter:

```sh
cd fv/acir_lean && lake env lean --run EmitSemantics.lean ssa_semantics.golden
cargo test -p noirc_evaluator --lib fv_semantics
```

## Running locally

Install Lean via elan (the toolchain version comes from `lean-toolchain`):

```sh
curl -sSfL https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y --default-toolchain none
```

Then, from the repository root:

```sh
(cd fv/acir_lean && ./scripts/check.sh)                  # ~2 min first run (Mathlib cache download), ~20 s after
cargo test -p noirc_evaluator --lib fv_                  # Rust side of the pins
```

After a change to the corpus programs in `fv_templates.rs`, or a compiler
change that alters their circuits or ACVM's witness for them, regenerate the
Lean data from the Rust output (the converter is untrusted; the pin checks its
output):

```sh
just fv-regen-corpus
```

When `integer_gadgets_match_lean_templates` fails, its message lists the
sections of `templates.golden` that changed and what to do for each.

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
in the proofs, the ones `testProgramNames` (`Spec/Coverage.lean`) lists.
`just fv-regen-all` brings new ones in and adds them to that list, a change to
the reviewed spec; if the checker rejects one, `fv-check` fails until it is
listed, with the reason, in `uncoveredPrograms` (`Spec/Claims.lean`).

`just fv-regen` never drops a program. If one of the listed programs no longer
compiles or no longer fits the supported subset, it fails and writes nothing;
taking the program out of the claims means deleting its name from
`testProgramNames`, which a reviewer sees.

This writes `Templates/TestPrograms.lean`, `test_programs.golden` and
`test_programs.outside`. Two CI checks keep them honest:

- `FV test programs` (`.github/workflows/fv-test-programs.yml`) rebuilds the
  programs already in the proofs on every pull request, in parallel with the
  other workflows, and fails if any of their SSA or circuits changed. A compiler
  change that alters one of them therefore has to commit the rebuilt data
  (`just fv-regen`).
- `FV Lean` (`check.sh`) requires the Lean data to print exactly
  `test_programs.golden`, to hold exactly the programs `testProgramNames`
  lists, and the checker to accept every program outside
  `uncoveredPrograms`, so rebuilt data with a circuit the checker cannot prove
  sound, or with a program missing, fails there.

To see a soundness bug caught, delete the `q ≤ q0` bound in
`euclidean_division_var` (the `bound_constraint_with_offset(quotient_var,
q0_var, …)` call) and run the Rust test: it fails, and `AcirLean/Examples/Bug7895.lean` shows
why the Lean side cannot be updated to match.
