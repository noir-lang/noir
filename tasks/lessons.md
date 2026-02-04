# Lessons Learned

## Always assume the test suite passes on master

- **Failure mode**: Attempted to `git stash` and re-run tests on master to check if failures were pre-existing, wasting time and requiring user intervention.
- **Detection signal**: When branched off master and seeing test failures, the instinct was to verify against master rather than debug the regression.
- **Prevention rule**: If branched off of master, always assume the full test suite is passing. Any test failure is a regression caused by the current changes. Focus on debugging why the change broke existing tests rather than checking master.

## Brillig constant pre-allocation and inter-block liveness

- **Failure mode**: Excluding numeric constants from `initialize_constants` for MakeArray without checking whether those constants are live after the instruction. Constants live across blocks must be in `ssa_value_allocations` for the next block's `get_available_variable_allocations` to find them.
- **Detection signal**: "ICE: Value not found in cache" panic in `get_available_variable_allocations` during block transitions.
- **Prevention rule**: When skipping pre-allocation of constants, only skip those whose **last use** is at the current instruction (i.e., they are dead after it). Constants that remain live must still go through `convert_ssa_value` / `define_variable` so they are tracked in `ssa_value_allocations`.

## Avoid overly broad safety checks to mask bugs

- **Failure mode**: Added a blanket `is_allocated` guard to the dead variable cleanup for ALL instructions, which could silently skip cleanup of variables that should have been allocated, masking real bugs.
- **Detection signal**: Some test regressions were caused by leaked registers from variables whose cleanup was incorrectly skipped.
- **Prevention rule**: Safety checks (like `is_allocated`) should be scoped as narrowly as possible. Use explicit excluded sets tied to specific instructions rather than broad guards that affect all code paths.

## Use `nargo info --force-brillig` to verify no bytecode regressions

- **Failure mode**: Making changes to Brillig codegen (especially constant handling or array init) without checking opcode counts, leading to duplicate `Const` opcodes or other hidden regressions.
- **Detection signal**: Increased Brillig opcode counts in `nargo info --force-brillig --json` output compared to master.
- **Prevention rule**: When modifying Brillig codegen paths, always run `nargo info --force-brillig --json` on a few representative test programs and diff the Brillig opcode counts against master before pushing.
