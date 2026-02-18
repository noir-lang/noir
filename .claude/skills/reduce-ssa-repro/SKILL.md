---
name: reduce-ssa-repro
description: Minimize an SSA file that triggers a bug in the noir-ssa pipeline, producing the smallest possible reproduction case. Use after bisecting to identify which SSA passes cause the issue.
---

# Reduce SSA Reproduction

Minimize an SSA file that triggers a bug in the `noir-ssa` pipeline. Use after the `bisect-ssa-pass` skill has identified which passes cause the issue.

## Setup

1. Build the `noir-ssa` CLI as a **debug binary**:
   ```bash
   cargo build -p noir_ssa_cli
   ```
   Always use debug builds (`target/debug/noir-ssa`). Many SSA invariant checks are `#[cfg(debug_assertions)]`-guarded and compiled out of release builds.

2. Set `SKILL_DIR` to this skill's directory (for script references):
   ```bash
   SKILL_DIR=<path-to-repo>/.claude/skills/reduce-ssa-repro
   ```

3. Gather from bisection:
   - The SSA file (`input.ssa`)
   - The passes that trigger the bug (corruption passes + detection pass)
   - The error pattern from stderr

## 1. Minimize the Pass Pipeline

Reduce the number of passes needed to trigger the bug. For each non-detection pass in the pipeline, try removing it:

```bash
# If the pipeline is: A → B → C → Detection
# Try removing each non-detection pass:
noir-ssa transform --source-path input.ssa --ssa-pass "B" --ssa-pass "C" --ssa-pass "Detection" -o /dev/null
noir-ssa transform --source-path input.ssa --ssa-pass "A" --ssa-pass "C" --ssa-pass "Detection" -o /dev/null
noir-ssa transform --source-path input.ssa --ssa-pass "A" --ssa-pass "B" --ssa-pass "Detection" -o /dev/null
```

When a pass produces valid output (not the buggy pass), **bake it into the input** to shrink both input and pipeline:

```bash
noir-ssa transform --source-path input.ssa --ssa-pass "A" -o after_A.ssa
# Verify crash reproduces without A:
noir-ssa transform --source-path after_A.ssa --ssa-pass "B" --ssa-pass "Detection" -o /dev/null
# If it still crashes:
cp after_A.ssa input.ssa
```

Repeat until no more passes can be removed.

## 2. Set Up the Reproduction Script

Use `$SKILL_DIR/scripts/reproduce_crash.sh`. It has two modes depending on whether `SSA_PASSES` is set:

**Multi-pass mode** — when corruption passes precede the detection pass:
```bash
SSA_PASSES="<pass>" DETECTION_PASS="<detection-pass>" $SKILL_DIR/scripts/reproduce_crash.sh input.ssa
SSA_PASSES="<pass-1>:<pass-2>" DETECTION_PASS="<detection-pass>" $SKILL_DIR/scripts/reproduce_crash.sh input.ssa
```
Validates: (1) input parses, (2) detection pass alone succeeds, (3) full pipeline crashes.

**Single-pass mode** — when a single pass crashes directly on the input (all intermediate passes were removed in step 1):
```bash
DETECTION_PASS="<crashing-pass>" $SKILL_DIR/scripts/reproduce_crash.sh input.ssa
```
Validates: (1) input parses, (2) the pass crashes.

### Choosing a detection pass

The detection pass is appended after the buggy passes. `noir-ssa transform` calls `normalize_ids` when formatting output after every pass, so any pass works for detecting corruption caught by `normalize_ids` (e.g., "Unmapped value" panics). `Simplifying` or `Inlining Brillig Calls` are lightweight choices.

The script defaults to `target/debug/noir-ssa` relative to the repo root. Override with `NOIR_SSA=/path/to/noir-ssa`.

**Run `reproduce_crash.sh` after every change** to `input.ssa` to verify the crash still reproduces.

## 3. Automated Reduction

Run reducer scripts from the same directory as `input.ssa`. Both `--passes` and `--error-pattern` are required. Pass all passes including the detection pass.

### Phase 1: Remove unused instructions

```bash
python3 $SKILL_DIR/scripts/reduce_instructions.py --passes "<pass>" "<detection-pass>" --error-pattern "<error text>"
```

Iterates over instructions, removes those whose results aren't referenced elsewhere, keeps only removals where the crash still reproduces.

### Phase 2: Collapse control flow

```bash
python3 $SKILL_DIR/scripts/reduce_branches.py --passes "<pass>" "<detection-pass>" --error-pattern "<error text>"
```

Converts `jmpif` to unconditional `jmp` (tries both targets). Re-runs Phase 1 after each successful collapse.

### Phase 3: Apply cleanup passes

Apply simplification passes one at a time. After each, verify the **same error pattern** still triggers:

```bash
noir-ssa transform --source-path input.ssa --ssa-pass "Simplifying" -o candidate.ssa
# Then verify:
SSA_PASSES="<pass>" DETECTION_PASS="<detection-pass>" $SKILL_DIR/scripts/reproduce_crash.sh candidate.ssa
# If same error, replace:
cp candidate.ssa input.ssa
```

Try these passes in order:
1. `Simplifying` — collapses trivial block chains
2. `Mem2Reg` — eliminates store/load pairs
3. `Dead Instruction Elimination` — removes dead instructions

**If a cleanup pass triggers a different error**: save that SSA separately as a potential second bug, but do not use it as `input.ssa`.

### Phase 4: Repeat

Re-run phases 1–3 until no more reductions are possible.

### Phase 5: Manual simplifications

Try each change one at a time, running `reproduce_crash.sh` after each:

- **Remove unused globals**: delete `gN` definitions not referenced in any function body
- **Simplify constants**: replace large numeric constants (`u128 671967...`) with small values (`u128 1`)
- **Remove unused functions**: if `f0` just calls `f1`, try `return` in `f0`; or inline `f1` into `f0`
- **Reduce loop bounds**: e.g., `lt vN, u32 3` → `lt vN, u32 1`. Test one change at a time — reductions that work individually may not compose
- **Remove function arguments**: remove the parameter from the signature, replace uses with a constant, remove the argument from all call sites. Arguments feeding control flow (`jmpif` conditions, loop bounds) are less likely to be removable
- **Remove return values**: replace `return vN` with `return`, update the function signature and callers

## 4. Completion Criteria

Reduction is done when:
- No automated reducer makes further progress
- No cleanup pass shrinks the input
- Manual simplifications have been attempted
- The minimized SSA is small enough to read and understand the bug pattern

## 5. Common Bug Patterns

After reduction, the minimized SSA reveals the structural trigger:

- **Unreachable value references**: a pass leaves instructions in unreachable blocks whose values are still referenced in reachable blocks. Detected by `normalize_value_ids` panicking with "Unmapped value".
- **Missing stores**: a pass removes a `store`, leaving a `load` that reads uninitialized memory. Detected by "loaded before it was first stored".
- **Changed semantics**: a pass changes program output. Detected by comparing interpreter results before and after.

## Reference

- **`noir-ssa check`**: parses SSA, normalizes IDs, removes unreachable blocks, prints canonical form. Use to clean up after manual edits.
- **`--ssa-pass` uses substring matching** (`contains()`), always matching the **first** pass with that name. Passes appearing multiple times may have different implementations at different pipeline positions.
- **Serialize to heal state**: `noir-ssa transform -o file.ssa` then re-read — isolates whether corruption is in DFG state or logical SSA structure.
