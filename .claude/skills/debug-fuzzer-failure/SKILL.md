---
name: debug-fuzzer-failure
description: End-to-end workflow for debugging SSA fuzzer failures from CI. Regenerates a reproduction from the CI seed with `just fuzz-repro`, then bisects SSA passes to identify the bug. Use when a `pass_vs_prev` or similar fuzzer test fails in CI.
---

# Debugging SSA Fuzzer Failures

This skill provides the complete workflow for debugging SSA optimization bugs discovered by CI fuzzers. It combines two sub-skills:

1. **`extract-fuzzer-repro`** — Regenerate a Noir project from the CI seed with `just fuzz-repro`
2. **`bisect-ssa-pass`** — Bisect SSA passes to find the one that breaks semantics

## When to Use This Skill

Use this when:
- A `pass_vs_prev` fuzzer test fails in CI
- You have a GitHub Actions URL showing a fuzzer failure
- You need to go from "CI is red" to "I know which SSA pass has the bug"

**Note**: This workflow is tailored for `pass_vs_prev` failures, which detect SSA passes that break semantic preservation. Other fuzzers exist that test different properties and may require different debugging approaches. Identify the fuzzer type from the failing test name in the GitHub logs — it follows the format `targets::<fuzzer_type>::tests::fuzz_with_arbtest`. If the failure comes from a fuzzer other than `pass_vs_prev`, ask the developer for guidance on how to proceed.

## Workflow Overview

1. [ ] Regenerate the reproduction case from the CI seed (`extract-fuzzer-repro`)
2. [ ] Verify the failure reproduces locally
3. [ ] Bisect, analyze, and fix (`bisect-ssa-pass`)

## Step 1: Reproduce from the Seed

Use the `extract-fuzzer-repro` skill to get a local Noir project. The AST fuzzers are seeded, so the reliable path is to pull the **seed** (and the failing target) out of the CI log and regenerate the program with `just fuzz-repro`, rather than copying the printed AST by hand.

**Input**: GitHub Actions job URL (e.g., `https://github.com/noir-lang/noir/actions/runs/12345/job/67890`)

**Output**: A Noir project directory with:
- `src/main.nr` — The generated program that triggered the failure
- `Prover.toml` — Input values that cause the bug

Quick reference:
```bash
# Pull the seed (Seed: 0x... / NOIR_AST_FUZZER_SEED=0x...) and target
# (targets::<target>::tests::fuzz_with_arbtest) from the CI log, then:
just fuzz-repro 0x<seed> <target> ./repro
```
Reproduce on the same commit CI ran on, since the seed→program mapping can drift as the generator changes. The `extract-fuzzer-repro` skill has the full details and a log-scraping fallback for when the seed no longer reproduces.

## Step 2: Verify the Failure Reproduces Locally

Before bisecting, confirm the issue reproduces:

```bash
cd repro
nargo execute
```

If using experimental features (like enums/match), add the appropriate flags:
```bash
nargo execute -Zenums
```

## Step 3: Bisect and Fix

Use the `bisect-ssa-pass` skill to:
- Identify which optimization pass breaks semantics
- Analyze the incorrect transformation
- Create a regression test
- Fix the bug

## Tips

- The `pass_vs_prev` fuzzer compares interpretation results before and after each pass, so failures indicate semantic preservation bugs
- Keep the extracted project around until the fix is merged — you may need to re-test
- If the SSA is complex, focus on the specific function/block where the semantic change occurs

## Related Skills

- `extract-fuzzer-repro` — Detailed instructions for reproducing from the CI seed with `just fuzz-repro`
- `bisect-ssa-pass` — Detailed instructions for SSA bisection and regression tests
