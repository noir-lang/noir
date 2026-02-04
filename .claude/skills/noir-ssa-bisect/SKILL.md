---
name: noir-ssa-bisect
description: Workflow for debugging SSA optimization bugs using the noir-ssa CLI. Use when bisecting fuzzer failures to identify which SSA pass introduces a bug.
allowed-tools: Bash, Read, Grep, Glob, Write
---

# Noir SSA Bisection Debugging

Use this skill when debugging SSA optimization bugs, particularly those found by the AST fuzzer. This workflow helps identify which SSA pass introduces a failure.

Helper scripts are in the `scripts/` directory of this skill.

## 1. Extracting Fuzzer Reproduction Cases from GitHub Actions

When a fuzzer test fails in CI, extract the reproduction case from the job logs:

```bash
# Fetch the job logs (replace JOB_ID with the actual job ID from the URL)
gh api repos/noir-lang/noir/actions/jobs/JOB_ID/logs 2>&1 | tee fuzzer_logs.txt
```

Search for the reproduction code in the logs:
```bash
grep -nE "unconstrained fn main|fn main" fuzzer_logs.txt
```

The fuzzer output includes:
- **AST**: The generated Noir source code (after "AST:" marker)
- **ABI Inputs**: The input values (after "ABI Inputs:" marker)
- **Seed**: The fuzzer seed for reproduction

Example log structure:
```
---
AST:
global G_A: i8 = -127_i8;
unconstrained fn main(a: pub i8) -> pub i8 {
    ...
}
---
ABI Inputs:
a = "-0x5c"
---
Seed: 0xc63ed07b00100000
```

Copy the Noir code to `src/main.nr` and create `Prover.toml` with the inputs.

**Note**: If the code uses `match` expressions, you'll need the `-Zenums` flag when compiling.

## 2. Compiling and Splitting SSA Passes

Compile with `--show-ssa` to output SSA after each optimization pass:

```bash
# Basic compilation (add -Zenums if using match expressions)
nargo compile --show-ssa 2>&1 | tee ssa_output.txt
```

**Important**: Do not pipe the output through `head` or other truncating commands (e.g., `| tee ssa_output.txt | head -50`). This truncates the file before all passes are written. Inspect the file separately after compilation completes.

Verify the output captured all passes (~49 expected):
```bash
grep -E "^After " ssa_output.txt
```

### Split into separate files

Use the provided script to split the output into one file per pass:

```bash
./scripts/split-ssa-passes.sh ssa_output.txt ssa_passes
```

Then clean up headers and diagnostics:

```bash
./scripts/clean-ssa-files.sh ssa_passes
```

This creates files like:
```
ssa_passes/01_Initial_SSA.ssa
ssa_passes/02_black_box_bypass_(1)_(step_1).ssa
ssa_passes/03_expand_signed_checks_(1)_(step_2).ssa
...
```

## 3. Using noir-ssa CLI to Bisect Failures

Build the SSA CLI tool (if not already built):
```bash
cargo build --release -p noir_ssa_cli
```

The binary is at `target/release/noir-ssa`.

### Bisecting to Find the Failing Pass

Use the bisect script to run interpretation on each pass:

```bash
./scripts/bisect-ssa.sh 'v0 = "-92"' ssa_passes /path/to/noir-ssa
```

**Input format**: SSA parameters are named `v0`, `v1`, etc. based on their order. Convert your Prover.toml inputs accordingly (e.g., `a = "-92"` becomes `v0 = "-92"`). Use `;` to separate multiple inputs.

Example output:
```
01_Initial_SSA.ssa: Ok(i8 1)
...
06_Inlining_simple_functions_(1)_(step_5).ssa: Ok(i8 1)
07_Mem2Reg_(1)_(step_6).ssa: Err(Reference value `*v8 = None` loaded before it was first stored to)
...

First failure: 07_Mem2Reg_(1)_(step_6).ssa
```

This identifies **Mem2Reg (step 6)** as the pass that introduced the bug.

### Manual Interpretation

You can also run interpretation manually on individual files:

```bash
noir-ssa interpret --source-path ssa_passes/06_Inlining_simple_functions_*.ssa --input-toml 'v0 = "-92"'
noir-ssa interpret --source-path ssa_passes/07_Mem2Reg_*.ssa --input-toml 'v0 = "-92"'
```

Additional options:
- `--trace`: Enable execution tracing
- `--input-path`: Read inputs from a TOML file instead of inline

### Comparing SSA Before/After

Once you identify the failing pass, compare the SSA files:
```bash
diff ssa_passes/06_*.ssa ssa_passes/07_*.ssa
```

Or read both files and look for the specific difference that causes the failure.

## 4. Creating Regression Tests

Once you've identified the failing pass, create a unit test to prevent regression. **Use the actual SSA from the `ssa_passes/` directory** rather than trying to manually simplify or recreate the pattern.

### Why use the SSA files directly?

- The SSA in `ssa_passes/` is the exact input that triggers the bug
- Manually simplifying can accidentally remove the triggering pattern
- Complex inputs (like arrays of references) are difficult to construct programmatically
- The SSA parser accepts the exact format already present in the files

### Creating the test

1. Read the SSA file from the pass **before** the failure (the last working pass):
   ```bash
   cat ssa_passes/06_Inlining_simple_functions_*.ssa
   ```

2. Extract the relevant function(s) that exhibit the bug

3. Add a test to the appropriate pass's test module (e.g., `compiler/noirc_evaluator/src/ssa/opt/mem2reg.rs`):

```rust
#[test]
fn regression_test_name() {
    let src = r#"
    // Paste the SSA function(s) from the ssa_passes file here
    brillig(inline) fn func_1 f0 {
      b0(v2: [&mut u1; 3]):
        // ... exact SSA from the file
        return v19
    }
    "#;

    // Choose the appropriate assertion based on the bug:
    let ssa = Ssa::from_str(src).unwrap();
    let result = ssa.mem2reg();
    // Then verify the result maintains correctness
}
```

### Test approaches

- **Interpret before and after**: Run the SSA interpreter on the SSA before and after the pass with the same inputs, and verify both produce the same result. This is a good general approach since optimization passes should preserve semantics.
- `assert_normalized_ssa_equals(src, expected)` — Verifies exact SSA output after transformation
- `assert_ssa_does_not_change(src, pass_fn)` — Verifies the pass doesn't modify the SSA (useful when a pass incorrectly removes instructions it shouldn't)

### Using `#[should_panic]` for known bugs

If documenting a bug that hasn't been fixed yet, use `#[should_panic(expected = "...")]` with a specific expected string from the assertion. This:
- Documents the bug exists
- Causes the test to fail (alerting you) once the bug is fixed
- Reminds you to convert it to a proper passing test

## Common Failure Patterns

- **"Reference value loaded before it was first stored to"**: An optimization pass incorrectly removed a `store` instruction, leaving a reference uninitialized
- **Different return values**: An optimization changed program semantics
- **Panic/crash**: Invalid SSA was generated

## Common Pitfalls

- **Do not pipe `tee` through `head`**: Running `| tee file.txt | head -50` truncates the file. Always let `tee` complete, then inspect the file separately.
- **Noir package names must use underscores**: Use `nargo new my_package`, not `my-package`.
- **SSA parameters are positional**: The first parameter becomes `v0`, second `v1`, etc. Map your `Prover.toml` inputs accordingly.

## Other noir-ssa Commands

```bash
# List available SSA passes
noir-ssa list

# Parse and validate SSA (prints normalized form)
noir-ssa check --source-path file.ssa

# Transform SSA by applying passes
noir-ssa transform --source-path file.ssa --passes "Mem2Reg,Simplifying"
```
