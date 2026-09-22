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
   | `LOW` | only intermediate witnesses move. Some hints are legitimately free — the inverse hint of an `x != 0` check when `x` is zero, or any call under a false predicate |

   Only `HIGH` makes the tool exit non-zero. `PROGRAM` is the author's decision to report, not a
   compiler defect, and it is what Noir's own `check_for_missing_brillig_constraints` warns about
   statically.

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

## What it does not do yet

- Only the first ACIR function is searched; programs that use `Call` opcodes are covered only in
  `main`.
- No hint-aware strategy: candidate values ignore what a directive computes, so an alias is only
  found when the near, edge or wraparound families happen to contain it.
- Findings report the opcode location, not yet the Noir source line.
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
| `LOW` | 55 | 48 of them `directive_invert`, the expected benign case |
| none | 370 | |
| skipped | 66 | no `Prover.toml`, or honest execution needs an oracle transcript |

Zero `HIGH` on a clean compiler is the property that makes the grade worth acting on. The 55
`PROGRAM` findings are a useful check that the search works at all: it rediscovered, from execution
alone, the same class of program the compiler's static check flags.

See `corpus/README.md` for the rest of the corpus and how it is scored.
