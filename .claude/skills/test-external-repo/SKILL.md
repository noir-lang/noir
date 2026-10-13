---
name: test-external-repo
description: Run tests for external Noir libraries (bigcurve, bignum, etc.) locally without pushing to CI. Use when told about a regression in an external repo's test suite, or when you need to verify that changes don't break external libraries.
user-invocable: false
---

# Running External Repo Tests Locally

Run the same external library tests that CI runs, but locally with any nargo binary. The core test logic lives in `.github/scripts/run-external-repo-tests.sh` — our wrapper scripts set up env vars and delegate to it.

## Scripts

| Script | Purpose |
|--------|---------|
| `.claude/scripts/run-external-repo-test.sh` | Clones repo + delegates to CI script. Thin wrapper. |
| `.claude/scripts/compare-external-repo-timing.sh` | Runs baseline vs test binary, prints timing comparison. |
| `.claude/scripts/extract-test-timing.sh` | Extracts per-test timing from JSON output. Supports single-run and two-run diff. |

## Quick Reference

```bash
# Run a single repo's tests
NARGO=./target/release/nargo .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve

# Compare two binaries
NARGO_BASELINE=/tmp/nargo-master NARGO_TEST=./target/release/nargo \
  .claude/scripts/compare-external-repo-timing.sh noir-lang/noir_bigcurve

# Reuse clone on repeated runs
KEEP_CLONE=1 NARGO=./target/release/nargo .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve

# Extract per-test timing
.claude/scripts/extract-test-timing.sh /tmp/external-test-results/noir-lang_noir_bigcurve.jsonl

# Compare per-test timing between two runs
.claude/scripts/extract-test-timing.sh \
  /tmp/external-test-results/noir-lang_noir_bigcurve.baseline.jsonl \
  /tmp/external-test-results/noir-lang_noir_bigcurve.test.jsonl
```

## Known External Libraries

Full list in `EXTERNAL_NOIR_LIBRARIES.yml`. Key ones for performance:

| Library | Repo | Timeout |
|---------|------|---------|
| bigcurve | `noir-lang/noir_bigcurve` | 600s |
| bignum | `noir-lang/noir-bignum` | 600s |
| aztec-nr | `AztecProtocol/aztec-packages` (path: `noir-projects/aztec-nr`) | 260s |
| blob | `AztecProtocol/aztec-packages` (path: `noir-projects/noir-protocol-circuits/crates/blob`) | 800s |

## Workflow for Diagnosing a Regression

### 1. Build nargo binaries

```bash
# Build current branch
cargo build --release -p nargo_cli

# Build master baseline
git stash && git checkout origin/master
cargo build --release -p nargo_cli
cp target/release/nargo /tmp/nargo-master
git checkout - && git stash pop
```

### 2. Run comparison

```bash
NARGO_BASELINE=/tmp/nargo-master NARGO_TEST=./target/release/nargo \
  .claude/scripts/compare-external-repo-timing.sh noir-lang/noir_bigcurve
```

### 3. Analyze per-test timing

```bash
.claude/scripts/extract-test-timing.sh \
  /tmp/external-test-results/noir-lang_noir_bigcurve.baseline.jsonl \
  /tmp/external-test-results/noir-lang_noir_bigcurve.test.jsonl
```

### 4. Profile if needed

```bash
# nargo's built-in profiling
NARGO=./target/release/nargo \
  .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve "" "" "--profile-execution"
```

## Environment Variables

| Var | Default | Description |
|-----|---------|-------------|
| `NARGO` | `nargo` | Path to nargo binary |
| `CLONE_DIR` | `/tmp/external-repos` | Where repos get cloned |
| `KEEP_CLONE` | `0` | Set to 1 to reuse existing clone |
| `NARGO_IGNORE_TEST_FAILURES_FROM_FOREIGN_CALLS` | unset | Set to `true` to match CI behavior |
