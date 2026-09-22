# noir-witness-mutator

Searches a compiled Noir program for a **second witness**: a different assignment of the
intermediate values that satisfies every constraint for the same inputs. A circuit should pin its
witness down once the inputs are fixed, so a second witness means a prover has a choice the program
did not intend — the practical meaning of "underconstrained".

```sh
nargo compile --program-dir <dir>
noir-witness-mutator --artifact-path <dir>/target/<pkg>.json --prover-file <dir>/Prover.toml
```

It exits non-zero when a finding changes a return value.

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
4. **Report.** A finding that changes a return value is `HIGH`, since a prover can choose the
   program's output. One that changes only intermediates is `LOW` — some hints are legitimately free,
   such as the inverse hint of an `x != 0` check when `x` is zero.

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
- No hint-aware strategy: the three compiler-inserted directives (`Inverse`, `Quotient`,
  `ToLeBytes`) are treated like any other unconstrained call.
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

See `corpus/README.md` for the rest of the corpus and how it is scored.
