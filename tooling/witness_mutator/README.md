# noir-witness-mutator

Searches a compiled Noir program for a **second witness**: a different assignment of the
intermediate values that satisfies every constraint for the same inputs. A circuit should pin its
witness down once the inputs are fixed, so a second witness means a prover has a choice the program
did not intend — the practical meaning of "underconstrained".

```sh
nargo compile --program-dir <dir>
noir-witness-mutator --artifact-path <dir>/target/<pkg>.json --prover-file <dir>/Prover.toml
```

It exits non-zero when a compiler-inserted hint turns out to be underconstrained.

## How it works

Every witness the solver assigns is either an input, the output of an opcode that also constrains it
(`AssertZero`, black box, memory), or the output of a `BrilligCall`. Only the last kind is free, so
the search overrides hint outputs and asks the solver whether the result still satisfies the
program:

1. **Honest run.** Execute normally and record, per `BrilligCall` opcode, the values its outputs
   took.
2. **Candidates.** For one call site, move one output, optionally let the constraints solve for a
   second one, and keep the rest honest.
3. **Check.** Append a Brillig function that returns the candidate values, repoint that one call
   site at it, and re-solve the whole program. Nothing else changes, so the solver's own answer
   decides: if it completes, the candidate is a real second witness.
4. **Report.** Findings are graded by whose job it was to rule the second witness out. Each Brillig
   function carries its name into the artifact, and the compiler's own hints are the ones called
   `directive_*`, so the two cases are distinguishable without guessing:

   | grade | meaning |
   | --- | --- |
   | `HIGH` | a return value follows a **compiler-inserted** hint whose outputs are not pinned down: the emitted circuit is weaker than the program it came from |
   | `PROGRAM` | a return value follows an **`unconstrained fn` the program called**, with nothing constraining it. The circuit matches the source; the source trusts an unchecked value |
   | `WITNESS` | no return value moves, but the rest of the witness does — the proof does not pin down what the program computed |
   | `INERT` | only the hint's own outputs move and nothing follows them: the value is read by nothing |

   Only `HIGH` makes the tool exit non-zero. `PROGRAM` is the author's decision to report, not a
   compiler defect, and it is what Noir's own `check_for_missing_brillig_constraints` warns about
   statically.

   `WITNESS` exists because a return value is not the only thing a circuit can be about. A circuit
   whose statement is "I know a value with property P" often returns nothing at all, and a free
   witness there *is* the break, while a circuit that returns a result and leaves a scratch value
   free is fine. The difference is what the program claims to prove, which no tool can decide, so
   these are reported with their blast radius — how much of the rest of the witness moved — for a
   human to judge. A circuit with no return values says so in the report, since nothing in it can
   be graded by its effect on an output.

Because step 3 re-solves rather than reasoning about constraints, a `HIGH` finding is not a
heuristic: the alternative witness is printed and can be checked independently.

## How candidate values are chosen

A second witness is rarely near the honest value, and rarely a round number. Two families cover the
shapes that appear in practice, neither of which needs to know what a hint computes:

- **Near and edge values** — `v ± 1`, `0`, `1`, `p - 1`. These catch aliases one step away, such as a
  remainder that may exceed its divisor by one, which pairs with a quotient one lower.
- **Wraparound values** — for each constant `c` that multiplies the output in a constraint, `v`
  shifted by `floor(p / c)` and `ceil(p / c)`. Shifting a witness by either moves its product just
  past the modulus, so the constraint still holds in the field while the integer relation it stands
  for does not. `p / c` is not an integer and both neighbours matter: which one works depends on the
  other witnesses' honest values.

Each moved value is tried alone, and also with one other output of the same call **left free and
solved from an `AssertZero`**, exactly as the solver would. That second form is what finds aliases
whose partner value is not guessable — a quotient's matching remainder, for instance, is whatever
the recomposition constraint forces.

On top of those, the three compiler directives get candidates computed from **what they compute**.
Each stands for an integer relation the constraints can only check modulo `p`, so recomputing the
hint on `a + k*p` yields the outputs a dishonest prover would supply:

| directive | candidates |
| --- | --- |
| `directive_integer_quotient` | `(a + k*p) divmod b` |
| `directive_to_radix` | the limbs of `a + k*p` |
| `directive_invert` | any non-zero value, but only when the input is zero — the inverse of anything else is unique |

This family is the only one that can supply every output of a call at once, which is what a
many-limbed decomposition needs.

## What it does not do yet

- Only the first ACIR function is searched; programs that use `Call` opcodes are covered only in
  `main`.
- Hint-aware candidates cover only the three compiler directives; any other unconstrained call gets
  the generic families alone.
- Only one call site is overridden at a time, so it cannot find a witness that requires two hints to
  move together.

## Measured behaviour

Against `corpus/div-quotient-bound`, which removes the quotient bound that #7895 added:

| compiler | inputs with a `HIGH` finding |
| --- | --- |
| clean `master` | 0 / 20 |
| bug applied | 3 / 20 |

The 20 inputs are fixed random field elements; the tool is not told the malicious value. Three is
the expected shape of the result rather than a shortfall: a wraparound alias only exists when the
shifted quotient still fits its range check, which for this program excludes most inputs. What
matters for a bug hunt is that *some* input exposes it, and that a clean compiler yields nothing.

## Sweep over the test suite

`sweep.sh` runs the search over a `test_programs` directory. On unmodified `master`, over
`execution_success` (546 programs, 2000 candidates and 120s per program):

| grade | programs | what they are |
| --- | --- | --- |
| `HIGH` | 0 | no compiler-emitted circuit was found to be weaker than its source |
| `PROGRAM` | 55 | 49 have an `unconstrained fn main`, whose result nothing can constrain; the other 6 return an `unsafe` call's value unchecked |
| `WITNESS` | 3 | free values that move part of the witness without reaching an output |
| `INERT` | 52 | mostly `directive_invert` with a zero input, the expected benign case |
| none | 370 | |
| skipped | 66 | no `Prover.toml`, or honest execution needs an oracle transcript |

Zero `HIGH` on a clean compiler is the property that makes the grade worth acting on. The
`PROGRAM` findings are a useful check that the search works at all: it rediscovered, from execution
alone, the same class of program the compiler's static check flags.

## Score against the corpus

`RESULTS.md` holds the held-out scoring: three of four "second witness" entries found, both
"accepts a bad input" controls correctly missed and caught by the differential oracle instead, and
one entry left open. The tool was frozen before any held-out entry was run.

See `corpus/README.md` for the corpus itself and how it is labelled.
