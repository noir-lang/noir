---
name: noir-optimize-acir
description: Workflow for measuring and optimizing the ACIR circuit size of a constrained Noir program. Use when asked to optimize a Noir program's gate count or circuit size.
allowed-tools: Bash, Read, Grep, Glob
---

# ACIR Optimization Loop

This workflow targets **ACIR circuit size** for constrained Noir programs. It does not apply to `unconstrained` (Brillig) functions — Brillig runs on a conventional VM where standard profiling and algorithmic improvements apply instead, and `bb gates` won't reflect Brillig performance.

## Measuring Circuit Size

### Binary projects

Compile the program and measure gate count with:

```bash
nargo compile && bb gates -b ./target/<package>.json
```

### Library projects

Libraries cannot be compiled with `nargo compile`. Instead, mark the functions you want to measure with `#[export]` and use `nargo export`:

```bash
nargo export && bb gates -b ./export/<function_name>.json
```

Artifacts are written to the `export/` directory and named after the exported function (not the package).

---

If `bb` is not available, ask the user for their backend's equivalent command. Other backends should have a similar CLI interface.

The output contains two fields:
- `circuit_size`: the actual gate count after backend compilation. This determines **proving time**, which is generally the bottleneck.
- `acir_opcodes`: number of ACIR operations. This affects **execution time** (witness generation). A change can reduce opcodes without affecting circuit size or vice versa — both matter, but prioritize `circuit_size` when they conflict.

Always record a baseline of both metrics before making changes.

## Optimization Loop

1. **Baseline**: compile and record `circuit_size`.
2. **Apply one change** at a time.
3. **Recompile and measure**: compare `circuit_size` to the baseline.
4. **Revert if worse**: if `circuit_size` increased or stayed the same, undo the change. Not every "optimization" helps — the compiler may already handle it, or the overhead of the new approach may outweigh the savings.
5. **Repeat** from step 2 with the next candidate change.

## What to Try

Candidate optimizations roughly ordered by impact:

- **Hint and verify**: replace expensive in-circuit computation with an unconstrained hint and constrained verification. This is the highest-impact optimization for most programs.
- **Reduce what you hint**: if you're hinting intermediate values (selectors, masks, indices), see if you can hint only the final result and verify it directly.
- **Hoist assertions out of branches**: replace `if c { assert_eq(x, a) } else { assert_eq(x, b) }` with `assert_eq(x, if c { a } else { b })`.
- **Simplify comparisons**: inequality checks (`<`, `<=`) cost more than equality (`==`). But don't introduce extra state to avoid them — measure first.

## What Not to Try

- **Don't hint division or modular arithmetic**: the compiler already injects unconstrained helpers for these.
- **Don't hand-roll conditional selects**: `if/else` expressions compile to the same circuit as `c * (a - b) + b`.
- **Don't replace `<=` with flag tracking without measuring**: adding mutable state across loop iterations can produce more gates than a simple comparison.
