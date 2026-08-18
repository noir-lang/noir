---
name: extract-fuzzer-repro
description: Reproduce an AST-fuzzer failure locally from its CI seed. Use when a CI fuzzer test (e.g. `pass_vs_prev`) fails and you need a local Noir project to debug. Prefer `just fuzz-repro <seed>` over hand-copying the AST out of the logs.
---

# Reproduce an AST-fuzzer failure locally

The AST fuzzers (`pass_vs_prev`, `acir_vs_brillig`, `min_vs_full`, `orig_vs_morph`, `comptime_vs_brillig_direct`, `comptime_vs_brillig_nargo`, `valid_after_pass`) are seeded. The fastest and most reliable reproduction is to **regenerate the failing program from its seed** with `just fuzz-repro`, which sets the env vars, runs the target, prints the failing AST, and — given an output directory — writes a runnable `nargo` project for you. This avoids copying a possibly-truncated AST out of the CI log by hand.

## 1. Get the seed and target from CI

From the GitHub Actions URL (`.../runs/RUN_ID/job/JOB_ID`), fetch the log and pull out the seed and the failing test name:

```bash
gh api repos/noir-lang/noir/actions/jobs/JOB_ID/logs 2>&1 | tee fuzzer_logs.txt

# Seed: printed on failure as `Seed: 0x...` and `NOIR_AST_FUZZER_SEED=0x...`
grep -oE '(NOIR_AST_FUZZER_SEED=|Seed:[[:space:]]*)0x[0-9a-fA-F]+' fuzzer_logs.txt \
  | grep -oE '0x[0-9a-fA-F]+' | sort -u

# Target: the nextest FAIL line, `targets::<target>::tests::fuzz_with_arbtest`
grep -oE 'targets::[a-z_]+::tests::fuzz_with_arbtest' fuzzer_logs.txt | sort -u
```

CI's `.github/scripts/extract-fuzz-seeds.sh` produces the same `seeds.txt` and a `seeds-by-test.tsv` (test↔seed pairs) from a captured run log, if you have that.

## 2. Check out the commit CI ran on

The seed→program mapping is deterministic **for a given compiler revision** but can drift as the AST generator changes. Reproduce on the same commit the fuzzer ran on:

```bash
git fetch origin <ci-sha> && git checkout <ci-sha>
```

If the fuzzer ran on an up-to-date `master`, your current `master` is usually fine.

## 3. Reproduce with `just fuzz-repro`

```bash
# <target> is optional — omit it to try every target until one reproduces.
# <out-dir> makes the harness also emit a runnable nargo project.
just fuzz-repro 0xc63ed07b00100000 pass_vs_prev ./repro
```

This prints the failing AST (and, for a comparison failure, the ABI inputs) and writes `./repro/{Nargo.toml, src/main.nr, Prover.toml}`. When a target compares more than one program (`orig_vs_morph`) each is written under `ast_1/`, `ast_2/`, sharing the same `Prover.toml`. The env vars it sets — `NOIR_AST_FUZZER_SEED`, `NOIR_AST_FUZZER_EMIT_PROJECT`, `NOIR_AST_FUZZER_SHOW_SSA`, `RUST_LOG=debug` — are documented in `tooling/ast_fuzzer/README.md`.

## 4. Verify the reproduction

```bash
cd repro
nargo execute        # add feature flags (e.g. -Zenums for match) if the program/log needs them
```

This should reproduce the failure. For `pass_vs_prev` (a semantic-preservation failure) continue with the `bisect-ssa-pass` skill using this project.

## Fallback: copy the AST from the log

If the seed no longer reproduces (e.g. the generator drifted and you cannot check out the original commit), the failing AST and inputs are still printed verbatim in the log. Look for the `--- AST:` (or `--- Failing AST:`) and `--- Inputs:` / `ABI Inputs:` sections, copy the AST into `src/main.nr` and the inputs into `Prover.toml`, then verify as above.

Note in both cases: for the compiled targets the rendered `main.nr` is *best-effort* Noir (the exact string the fuzzer uses to build integration tests) and may need small tweaks to parse; the comptime targets emit exact Noir source, and the `Prover.toml` inputs are always exact.
