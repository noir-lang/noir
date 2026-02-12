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

## Step 1: Build nargo on Both Branches

Build nargo for the current branch first, save the binary, then checkout the base branch, build, save, and return.

```bash
REPO_ROOT=$(git rev-parse --show-toplevel)
CURRENT_BRANCH=$(git branch --show-current)
BASE_BRANCH=master  # or whatever the user specifies

# Build nargo for the current branch
cargo build --release -p nargo_cli 2>&1 | tail -3
cp target/release/nargo /tmp/nargo_current

# Build nargo for the base branch
git checkout "$BASE_BRANCH"
cargo build --release -p nargo_cli 2>&1 | tail -3
cp target/release/nargo /tmp/nargo_base

# Return to the working branch
git checkout "$CURRENT_BRANCH"
```

## Step 2: Measure the Regression

Run `nargo info` with both binaries against the same Noir project. Run these from the Noir project directory.

### Bytecode size (static opcode count)

```bash
INLINER=9223372036854775807  # max; adjust as needed

# Base branch
/tmp/nargo_base info --force-brillig --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, brillig_opcodes: .unconstrained_functions_opcodes, functions: [.unconstrained_functions[] | {name, opcodes}]}'

# Current branch
/tmp/nargo_current info --force-brillig --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, brillig_opcodes: .unconstrained_functions_opcodes, functions: [.unconstrained_functions[] | {name, opcodes}]}'
```

### Execution trace (dynamic opcode count — requires Prover.toml)

```bash
# Base branch
/tmp/nargo_base info --profile-execution --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, executed_opcodes: .unconstrained_functions_opcodes}'

# Current branch
/tmp/nargo_current info --profile-execution --json --silence-warnings \
  --inliner-aggressiveness $INLINER 2>/dev/null \
  | jq '.programs[] | {package: .package_name, executed_opcodes: .unconstrained_functions_opcodes}'
```

Record the numbers. If the current branch is worse, proceed.

### Bulk comparison across all test programs

To replicate the CI report across all `execution_success` test programs:

```bash
cd "$REPO_ROOT/test_programs"

# Using the base nargo
PATH_BAK="$PATH"
export PATH="/tmp:$PATH"
ln -sf /tmp/nargo_base /tmp/nargo
./gates_report_brillig.sh $INLINER
mv gates_report_brillig.json /tmp/report_base.json

# Using the current nargo
ln -sf /tmp/nargo_current /tmp/nargo
./gates_report_brillig.sh $INLINER
mv gates_report_brillig.json /tmp/report_current.json
export PATH="$PATH_BAK"
```

Then diff the two JSON reports to find which programs regressed:
```bash
python3 -c "
import json, sys
base = {p['package_name']: p for p in json.load(open('/tmp/report_base.json'))['programs']}
curr = {p['package_name']: p for p in json.load(open('/tmp/report_current.json'))['programs']}
for name in sorted(set(base) & set(curr)):
    b = base[name]['unconstrained_functions_opcodes']
    c = curr[name]['unconstrained_functions_opcodes']
    if c != b:
        delta = c - b
        pct = (delta / b * 100) if b else float('inf')
        marker = '!!!' if delta > 0 else ''
        print(f'{marker} {name}: {b} -> {c} ({delta:+d}, {pct:+.1f}%) {marker}')
"
```

## Step 3: Capture SSA on Both Branches

For the regressed program, capture the full SSA pipeline using both nargo binaries.

```bash
PROJECT_DIR=/path/to/noir/project
WORK_DIR=/tmp/brillig_regression_debug
mkdir -p "$WORK_DIR"/{base,current}

# Base branch SSA
cd "$PROJECT_DIR"
/tmp/nargo_base compile --force-brillig --show-ssa --silence-warnings \
  --inliner-aggressiveness $INLINER 2>&1 | tee "$WORK_DIR/base/ssa_output.txt"

# Current branch SSA
/tmp/nargo_current compile --force-brillig --show-ssa --silence-warnings \
  --inliner-aggressiveness $INLINER 2>&1 | tee "$WORK_DIR/current/ssa_output.txt"
```

**Important**: Do not pipe `tee` through `head` or any truncating command.

### Split into per-pass files

```bash
SKILL_DIR="$REPO_ROOT/.claude/skills/bisect-ssa-pass/scripts"

"$SKILL_DIR/split-ssa-passes.sh" "$WORK_DIR/base/ssa_output.txt" "$WORK_DIR/base/passes"
"$SKILL_DIR/clean-ssa-files.sh" "$WORK_DIR/base/passes"

"$SKILL_DIR/split-ssa-passes.sh" "$WORK_DIR/current/ssa_output.txt" "$WORK_DIR/current/passes"
"$SKILL_DIR/clean-ssa-files.sh" "$WORK_DIR/current/passes"
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

Passes shared between both branches should produce identical SSA up to the point where the pipelines diverge. Find where they first differ:

```bash
for base_file in "$WORK_DIR/base/passes/"*.ssa; do
    base_name=$(basename "$base_file" | sed 's/^[0-9]*_//')
    current_file=$(ls "$WORK_DIR/current/passes/"*"$base_name" 2>/dev/null | head -1)
    if [[ -n "$current_file" ]]; then
        if ! diff -q "$base_file" "$current_file" > /dev/null 2>&1; then
            echo "DIFFERS: $base_name"
        fi
    else
        echo "MISSING from current: $base_name"
    fi
done
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
for branch in base current; do
    echo "=== $branch ==="
    LAST=$(ls "$WORK_DIR/$branch/passes/"*.ssa | tail -1)
    awk '/^brillig.*fn / {name=$0; count=0} /=/ {count++} /^}/ {print name, ":", count, "instructions"}' "$LAST"
done
```

Count block parameters (these map directly to Brillig `mov` instructions):
```bash
for branch in base current; do
    echo "=== $branch block params ==="
    LAST=$(ls "$WORK_DIR/$branch/passes/"*.ssa | tail -1)
    grep -oP 'b\d+\([^)]+\)' "$LAST" | awk -F',' '{print NF}' | paste -sd+ | bc
done
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
