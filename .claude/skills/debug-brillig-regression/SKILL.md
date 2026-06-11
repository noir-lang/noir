---
name: debug-brillig-regression
description: Debug Brillig opcode regressions between the current branch and a base branch (usually master). Measures bytecode/execution costs, captures SSA on both branches, diffs the optimization pipelines, and identifies which pass changes cause the regression. Use when a Noir program has worse Brillig performance on the current branch.
allowed-tools: Bash, Read, Grep, Glob, Write, Edit
---

# Debugging Brillig Regressions

Use this skill when a Noir program shows a Brillig opcode count or execution trace regression on the current branch compared to a base branch (usually `master`). The workflow measures the regression, captures full SSA pipelines on both branches, identifies which passes differ, and pinpoints the source of the regression.

## Inputs

- **Noir project path**: Directory containing `src/main.nr` and optionally `Prover.toml`
- **Base branch**: The branch to compare against (default: `master`)
- **Inliner aggressiveness** (optional): One of `max` (9223372036854775807), `zero` (0), or `min` (-9223372036854775808). Default: `max` (matches CI default). If the regression only shows at a specific inliner setting, use that setting.

## Setup

Set these variables for use throughout the workflow:

```bash
REPO_ROOT=$(git rev-parse --show-toplevel)
SKILL_DIR="$REPO_ROOT/.claude/skills/debug-brillig-regression/scripts"
BISECT_SKILL_DIR="$REPO_ROOT/.claude/skills/bisect-ssa-pass/scripts"
BASE_BRANCH=master  # or whatever the user specifies
INLINER=9223372036854775807  # max; adjust as needed
WORK_DIR=/tmp/brillig_regression_debug
PROJECT_DIR=/path/to/noir/project  # set to the actual project path
```

## Step 1: Build nargo on Both Branches

```bash
"$SKILL_DIR/build-both-branches.sh" "$BASE_BRANCH" "$WORK_DIR"
```

This builds nargo for the current branch and the base branch, saving binaries to `$WORK_DIR/nargo_base` and `$WORK_DIR/nargo_current`.

## Step 2: Measure the Regression

Run `nargo info` with both binaries against the same Noir project. Run these from the Noir project directory.

### Bytecode size (static opcode count)

```bash
cd "$PROJECT_DIR"

# Base branch
"$WORK_DIR/nargo_base" info --force-brillig --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, brillig_opcodes: .unconstrained_functions_opcodes, functions: [.unconstrained_functions[] | {name, opcodes}]}'

# Current branch
"$WORK_DIR/nargo_current" info --force-brillig --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, brillig_opcodes: .unconstrained_functions_opcodes, functions: [.unconstrained_functions[] | {name, opcodes}]}'
```

### Execution trace (dynamic opcode count — requires Prover.toml)

```bash
# Base branch
"$WORK_DIR/nargo_base" info --profile-execution --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, executed_opcodes: .unconstrained_functions_opcodes}'

# Current branch
"$WORK_DIR/nargo_current" info --profile-execution --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, executed_opcodes: .unconstrained_functions_opcodes}'
```

Record the numbers. If the current branch is worse, proceed.

### Bulk comparison across all test programs

To replicate the CI report across all `execution_success` test programs:

```bash
"$SKILL_DIR/bulk-compare.sh" "$WORK_DIR/nargo_base" "$WORK_DIR/nargo_current" "$INLINER" "$WORK_DIR"
```

Then diff the two reports to find which programs regressed:

```bash
"$SKILL_DIR/diff-reports.py" "$WORK_DIR/report_base.json" "$WORK_DIR/report_current.json"
```

## Step 3: Capture SSA on Both Branches

For the regressed program, capture the full SSA pipeline using both nargo binaries.

```bash
mkdir -p "$WORK_DIR"/{base,current}

cd "$PROJECT_DIR"

# Base branch SSA
"$WORK_DIR/nargo_base" compile --force-brillig --show-ssa --silence-warnings \
  --inliner-aggressiveness $INLINER 2>&1 | tee "$WORK_DIR/base/ssa_output.txt"

# Current branch SSA
"$WORK_DIR/nargo_current" compile --force-brillig --show-ssa --silence-warnings \
  --inliner-aggressiveness $INLINER 2>&1 | tee "$WORK_DIR/current/ssa_output.txt"
```

**Important**: Do not pipe `tee` through `head` or any truncating command.

### Split into per-pass files

```bash
"$BISECT_SKILL_DIR/split-ssa-passes.sh" "$WORK_DIR/base/ssa_output.txt" "$WORK_DIR/base/passes"
"$BISECT_SKILL_DIR/clean-ssa-files.sh" "$WORK_DIR/base/passes"

"$BISECT_SKILL_DIR/split-ssa-passes.sh" "$WORK_DIR/current/ssa_output.txt" "$WORK_DIR/current/passes"
"$BISECT_SKILL_DIR/clean-ssa-files.sh" "$WORK_DIR/current/passes"
```

## Step 4: Identify Which Passes Differ

The pass lists will likely differ between branches (the current branch may have new passes, removed passes, or reordered passes).

```bash
# Extract just pass names (strip numeric prefix and file extension)
ls "$WORK_DIR/base/passes/" | sed 's/^[0-9]*_//; s/\.ssa$//' > /tmp/passes_base.txt
ls "$WORK_DIR/current/passes/" | sed 's/^[0-9]*_//; s/\.ssa$//' > /tmp/passes_current.txt

# Show differences in pass lists
diff /tmp/passes_base.txt /tmp/passes_current.txt || true
```

This reveals:
- **New passes** on the current branch (likely source of regression)
- **Removed passes** (check if they were doing useful cleanup)
- **Reordered passes** (can affect optimization effectiveness)

## Step 5: Compare SSA at Key Points

### Find the divergence point

```bash
"$SKILL_DIR/find-divergence.sh" "$WORK_DIR/base/passes" "$WORK_DIR/current/passes"
```

### Focus on the divergence

Once you find the first pass where SSA differs, read the SSA before and after that pass on both branches. The pass immediately before the divergence should be identical; the diff right after shows what the changed/new pass introduced.

### Compare the final SSA

Always compare the last pass — this is what gets lowered to Brillig and directly explains the opcode difference:

```bash
BASE_LAST=$(ls "$WORK_DIR/base/passes/"*.ssa | tail -1)
CURR_LAST=$(ls "$WORK_DIR/current/passes/"*.ssa | tail -1)
diff "$BASE_LAST" "$CURR_LAST"
```

## Step 6: Measure Per-Function Impact

If the program has multiple functions, identify which function(s) regressed:

```bash
"$SKILL_DIR/measure-functions.sh" "$WORK_DIR/base/passes" "$WORK_DIR/current/passes"
```

## Step 7: Diagnose the Root Cause

With the SSA diffs in hand, analyze the regression. Common patterns:

### New pass introduces redundancy that later passes don't clean up

**Signal**: The new pass's output has more instructions, and the final SSA also has more.
**Fix**: Improve the new pass's output, or add/adjust a cleanup pass after it.

### New pass changes SSA shape, making a later pass less effective

**Signal**: The new pass output has similar or fewer instructions, but a later pass produces more on the current branch.
**Fix**: Investigate why the later pass is less effective with the new SSA shape. Common issues:
- More block parameters → more `mov` instructions in Brillig
- Different value numbering → constant folding can't merge as much
- Changed control flow → loop optimizations less effective

### New pass is correct but suboptimal for certain patterns

**Signal**: The pass does the right transformation but introduces unnecessary copies or parameters.
**Fix**: Add pattern-specific optimizations or skip certain variables.

## Tips

- **Start with the biggest regression**: If multiple programs regress, focus on the one with the largest absolute or percentage increase.
- **Check all three inliner settings**: A regression at one setting but improvement at another suggests the issue is in how the new pass interacts with inlining decisions.
- **Use `noir-ssa interpret`** to verify semantic correctness if you suspect the pass may also have a correctness bug (see the `bisect-ssa-pass` skill).
- **The `bisect-ssa-pass` skill** can complement this workflow if you need to bisect within the current branch's pipeline to find which specific pass produces suboptimal output.
