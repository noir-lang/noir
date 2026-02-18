---
name: reduce-ssa-repro
description: Minimize an SSA file that triggers a bug in the noir-ssa pipeline, producing the smallest possible reproduction case. Use after bisecting to identify which SSA passes cause the issue.
---

# Reduce SSA Reproduction

Minimize an SSA file that triggers a bug in the `noir-ssa` pipeline, producing the smallest possible reproduction case. Use this skill after bisecting to identify which SSA passes cause the issue (see the `bisect-ssa-pass` skill).

Helper scripts are in the `scripts/` directory of this skill.

## Prerequisites

You need:
- An SSA file (`input.ssa`) that, when processed by a specific sequence of passes, triggers a crash or incorrect behavior
- The `noir-ssa` CLI built: `cargo build --release -p noir_ssa_cli`
- Knowledge of which passes trigger the bug (from bisection)

## 1. Minimize the Pass Pipeline

Before reducing the SSA, minimize the set of passes needed to trigger the bug. Test subsets systematically:

```bash
# If the pipeline is: A → B → C → Detection
# Try removing each non-detection pass:
noir-ssa transform --source-path input.ssa --ssa-pass "B" --ssa-pass "C" --ssa-pass "Detection" -o /dev/null
noir-ssa transform --source-path input.ssa --ssa-pass "A" --ssa-pass "C" --ssa-pass "Detection" -o /dev/null
noir-ssa transform --source-path input.ssa --ssa-pass "A" --ssa-pass "B" --ssa-pass "Detection" -o /dev/null
```

When a pass in the pipeline produces valid output (i.e., it's not the buggy pass), run it and **replace `input.ssa` with its output**, removing that pass from the pipeline:

```bash
noir-ssa transform --source-path input.ssa --ssa-pass "A" -o after_A.ssa
# Verify the crash still reproduces without A:
noir-ssa transform --source-path after_A.ssa --ssa-pass "B" --ssa-pass "Detection" -o /dev/null
# If it crashes, replace:
cp after_A.ssa input.ssa
```

This shrinks both the input and the pipeline simultaneously. Repeat until no more passes can be removed.

## 2. Set Up the Reproduction Script

Use `scripts/reproduce_crash.sh` from this skill's directory. It validates three invariants:

1. **Input parses** (`check`): ensures the SSA file is syntactically valid
2. **Detection pass alone succeeds**: proves the bug is introduced by the preceding passes, not pre-existing in the input
3. **Full pipeline crashes**: the actual bug reproduction

Configure it via environment variables (`SSA_PASSES` and `DETECTION_PASS` are required):

```bash
# Single pass before detection:
SSA_PASSES="<pass>" DETECTION_PASS="<detection-pass>" ./scripts/reproduce_crash.sh input.ssa

# Multiple passes (colon-separated):
SSA_PASSES="<pass-1>:<pass-2>" DETECTION_PASS="<detection-pass>" ./scripts/reproduce_crash.sh input.ssa
```

The "detection pass" is any pass appended after the buggy passes. `noir-ssa transform` calls `normalize_ids` when formatting output after every pass, so any pass will trigger detection for corruption caught by `normalize_ids` (e.g., "Unmapped value" panics). `Simplifying` or `Inlining Brillig Calls` are reasonable lightweight choices. The detection pass is always appended to the end of the pipeline.

The script defaults to `target/release/noir-ssa` relative to the repo root. Override with `NOIR_SSA=/path/to/noir-ssa`.

**Always run the reproduction script after every change** to the input SSA to ensure the crash still reproduces and the invariants still hold.

## 3. Automated Reduction

Use the reducer scripts from this skill's `scripts/` directory. Run them in the same directory as `input.ssa`.

Both `--passes` and `--error-pattern` are required for the reducer scripts. The passes should include the detection pass at the end.

### Phase 1: Remove unused instructions

```bash
python3 scripts/reduce_instructions.py --passes "<pass>" "<detection-pass>" --error-pattern "<error text>"
```

This script iterates over every instruction in the SSA, tries removing it, and checks if:
- The SSA still parses (`noir-ssa check`)
- The crash still reproduces (the pass pipeline)

It removes instructions whose result values are not referenced elsewhere.

### Phase 2: Collapse control flow

```bash
python3 scripts/reduce_branches.py --passes "<pass>" "<detection-pass>" --error-pattern "<error text>"
```

This script tries:
- Converting `jmpif` branches to unconditional `jmp` (to either the then or else target)
- After each successful change, re-runs Phase 1 to clean up newly-unused instructions

### Phase 3: Apply cleanup passes

After automated reduction, apply SSA passes that simplify the input without changing semantics. Try each one, verify the crash still reproduces (with the **same error pattern**), then replace `input.ssa`:

```bash
# Collapse trivial block chains
noir-ssa transform --source-path input.ssa --ssa-pass "Simplifying" -o candidate.ssa

# Eliminate store/load pairs
noir-ssa transform --source-path input.ssa --ssa-pass "Mem2Reg" -o candidate.ssa

# Remove dead instructions
noir-ssa transform --source-path input.ssa --ssa-pass "Dead Instruction Elimination" -o candidate.ssa
```

After each, verify the original bug still reproduces:
```bash
SSA_PASSES="<pass>" DETECTION_PASS="<detection-pass>" ./scripts/reproduce_crash.sh candidate.ssa
# If it crashes with the same error, replace:
cp candidate.ssa input.ssa
```

**Important**: Always verify the error pattern still matches the original bug after applying a cleanup pass. Some passes change the SSA structure enough to trigger a *different* bug. If a cleanup pass triggers a different crash, save that SSA separately as a potential second bug to investigate later, but don't use it as the new `input.ssa`.

### Phase 4: Re-run reducers on simplified input

The simplified SSA may expose new reduction opportunities. Repeat phases 1-3 until no more reductions are possible.

### Phase 5: Manual simplifications

Try these manually:
- **Remove unused globals**: check which `gN` values appear in function bodies; remove those that don't
- **Simplify constants**: replace large numeric constants (e.g., `u128 671967...`) with small values (`u128 1`, `u128 2`)
- **Remove unused functions**: if `f0` just calls `f1`, try making `f0` just `return`; or merge `f1` into `f0`
- **Reduce loop bounds**: try reducing loop iteration counts (e.g., `lt vN, u32 3` → `lt vN, u32 1`). Apply one change at a time and re-verify — reductions that work individually may not work in combination, since loops interact through shared values

After each change, run `reproduce_crash.sh` to verify.

## 4. Understanding the Reduction

After reduction, the minimized SSA reveals the structural pattern that triggers the bug. Common patterns:

- **Unreachable value references**: A pass creates instructions in blocks that become unreachable, but values from those blocks are still referenced in reachable blocks. Detected by `normalize_value_ids` panicking with "Unmapped value".
- **Missing stores**: A pass removes a `store` instruction, leaving a `load` that reads uninitialized memory. Detected by the interpreter panicking with "loaded before it was first stored".
- **Changed semantics**: A pass transforms code in a way that changes the program's output. Detected by comparing interpreter results before and after.

## Tips

- **`noir-ssa check`** parses SSA text, normalizes IDs, and prints the canonical form. It removes unreachable blocks. Use it to clean up after manual edits.
- **`noir-ssa transform --ssa-pass "Simplifying"`** runs `simplify_cfg` which merges blocks connected by unconditional jumps. Essential after the branch reducer creates long chains of trivial blocks.
- **The `--ssa-pass` flag uses substring matching** via `contains()`, and always matches the **first** pass with that name in the pipeline. Passes that appear multiple times (like `Mem2Reg`, `Inlining`, `Dead Instruction Elimination`) may have different implementations at different pipeline positions. The first match is always returned.
- **Serialize between passes to heal state**: If a bug only manifests in-memory (not through text round-trip), serializing intermediate SSA via `noir-ssa transform -o file.ssa` followed by re-reading can isolate whether the corruption is in the DFG state or the logical SSA structure.
