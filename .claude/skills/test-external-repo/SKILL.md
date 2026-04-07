---
name: test-external-repo
description: Run tests for external Noir libraries (bigcurve, bignum, etc.) locally without pushing to CI. Use when told about a regression in an external repo's test suite, or when you need to verify that changes don't break external libraries.
user-invocable: false
---

# Running External Repo Tests Locally

This skill lets you run the same external library tests that CI runs, but locally with any nargo binary. This is essential for diagnosing performance regressions without round-tripping through CI.

## Available Scripts

All scripts are in `.claude/scripts/`:

### `run-external-repo-test.sh`
Clones an external repo and runs `nargo test` with the same flags as CI.

```bash
# Basic usage — uses nargo from PATH
.claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve

# With a specific nargo binary
NARGO=/path/to/nargo .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve

# With a specific ref and subpath
.claude/scripts/run-external-repo-test.sh AztecProtocol/aztec-packages 2b1671a noir-projects/aztec-nr

# Reuse existing clone (faster for repeated runs)
KEEP_CLONE=1 NARGO=./target/release/nargo .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve
```

Output goes to `/tmp/external-test-results/`:
- `<slug>.jsonl` — test results (JSON lines from `nargo test --format json`)
- `<slug>.timing.json` — wall-clock timing

### `compare-external-repo-timing.sh`
Runs the test suite twice (baseline vs test binary) and prints a timing comparison.

```bash
NARGO_BASELINE=/tmp/nargo-master NARGO_TEST=./target/release/nargo \
  .claude/scripts/compare-external-repo-timing.sh noir-lang/noir_bigcurve
```

## Known External Libraries

The full list is in `EXTERNAL_NOIR_LIBRARIES.yml`. Key ones for performance testing:

| Library | Repo | Typical CI time |
|---------|------|-----------------|
| bigcurve | `noir-lang/noir_bigcurve` | ~600s timeout |
| bignum | `noir-lang/noir-bignum` | ~600s timeout |
| aztec-nr | `AztecProtocol/aztec-packages` (path: `noir-projects/aztec-nr`) | ~260s timeout |
| blob | `AztecProtocol/aztec-packages` (path: `noir-projects/noir-protocol-circuits/crates/blob`) | ~800s timeout |

## Workflow for Diagnosing a Regression

When told about a performance regression in an external test suite:

### Step 1: Build nargo binaries

Build nargo from both master and the PR branch:

```bash
# Build master baseline
git stash  # or use a worktree
git checkout origin/master
cargo build --release -p nargo_cli
cp target/release/nargo /tmp/nargo-master

# Build the PR branch
git checkout <pr-branch>
cargo build --release -p nargo_cli
# nargo is now at ./target/release/nargo
```

Or if you're already on the PR branch and just need a comparison:

```bash
# The current branch nargo
cargo build --release -p nargo_cli

# Get master nargo
git worktree add /tmp/noir-master origin/master
cd /tmp/noir-master && cargo build --release -p nargo_cli
# baseline is at /tmp/noir-master/target/release/nargo
```

### Step 2: Run comparison

```bash
NARGO_BASELINE=/tmp/nargo-master \
NARGO_TEST=./target/release/nargo \
  .claude/scripts/compare-external-repo-timing.sh noir-lang/noir_bigcurve
```

### Step 3: Profile if needed

For deeper profiling, build an instrumented nargo and run against the external repo:

```bash
# Build with profiling support
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release -p nargo_cli

# Run with perf/flamegraph
NARGO="perf record -g -- ./target/release/nargo" \
  .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve

# Or use the --profile-execution flag for nargo's built-in profiling
NARGO="./target/release/nargo" \
  .claude/scripts/run-external-repo-test.sh noir-lang/noir_bigcurve "" "" "--profile-execution"
```

### Step 4: Analyze per-test timing

The JSON output from `nargo test --format json` includes per-test timing. Compare the baseline and test JSONLs:

```bash
# After running comparison, results are in /tmp/external-test-results/
# baseline: noir-lang_noir_bigcurve.baseline.jsonl
# test:     noir-lang_noir_bigcurve.test.jsonl
```

## CI Behavior Reference

The CI script (`.github/scripts/run-external-repo-tests.sh`) does exactly:
1. Clone the external repo
2. `cd` into the project path
3. Strip `compiler_version` lines from all `Nargo.toml` files
4. Run: `nargo test --silence-warnings --skip-brillig-constraints-check --format json $NARGO_ARGS`
5. Record wall-clock time

Our scripts replicate this exactly. The `NARGO_IGNORE_TEST_FAILURES_FROM_FOREIGN_CALLS=true` env var is set in CI but not by our scripts — set it manually if needed.
