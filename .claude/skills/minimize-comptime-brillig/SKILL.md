---
name: minimize-comptime-brillig
description: Workflow for minimizing comptime_vs_brillig fuzzer failures to create minimal reproduction cases. Use when a comptime_vs_brillig or comptime_vs_brillig_nargo fuzzer test fails and you need to simplify the generated program.
allowed-tools: Bash, Read, Edit, Write, Glob, Grep
---

# Minimizing comptime_vs_brillig Fuzzer Failures

Use this skill to reduce a complex fuzzer-generated program to a minimal reproduction case. A `comptime_vs_brillig` failure means the same unconstrained code produces different results when executed by the comptime interpreter (during compilation) versus the Brillig VM (at runtime).

Common scenarios:
- **Fuzzer failures**: The `comptime_vs_brillig` or `comptime_vs_brillig_nargo` fuzzer found a program with differing printed output between execution modes
- **Manual discovery**: An unconstrained function behaves differently when called from `comptime {}` versus at runtime


## Prerequisites

You need a Noir project with `src/main.nr` containing the fuzzer-generated program. The main function's body should be wrapped in a `comptime {}` block.

## 1. Testing for Differences

Run `test_diff.sh` from this skill's directory after each change, passing the project path:

```bash
.claude/skills/minimize-comptime-brillig/test_diff.sh path/to/project
```

Or from the project directory:
```bash
.claude/skills/minimize-comptime-brillig/test_diff.sh .
```

The script toggles between comptime and brillig execution and reports:
- **"DIFFERENT - bug still present"**: Your change preserved the bug, keep it
- **"SAME - bug not reproduced"**: Your change eliminated the bug, revert it

## 2. Minimization Strategy

### Phase 1: Remove Dead Code

- **Identify dead branches**: Look for `if false`, `while false`, loops that immediately break (`if idx == 0 { break }`), or conditions that are always true/false given the inputs
- **Trace from main**: Follow the actual execution path with the given inputs. Remove code in branches that are never taken
- **Remove unused functions**: If a function is passed as a parameter but never called, remove it and simplify the signature

### Phase 2: Simplify Functions

- **Remove unused parameters**: If a parameter isn't used (or only used in dead code), remove it from the signature and all call sites
- **Inline small functions**: If a function just wraps another call, inline it (but keep `entry`-style functions that convert values to references for comptime)
- **Simplify return values**: If return values aren't used, change functions to return `()`
- **Remove assertions after prints**: If `assert(false, ...)` comes after the differing `println`, try removing it

### Phase 3: Simplify Values

- **Replace large constants**: Change large Field values like `156799370756091332126270462529130485833` to simple values like `1`, `2`, `100`
- **Simplify arrays**: Replace `@[big1, big2, big3]` with `@[1, 2, 3]` or `@[1, 2]`
- **Reduce loop bounds/counters**: If there's a recursion limiter (like `ctx_limit`), try reducing it to find the minimum value that still reproduces the bug

### Phase 4: Structural Simplification

- **Flatten nested loops**: If nested loops don't contribute to the bug, replace them with simpler constructs
- **Remove block expressions**: Replace `{ { { x } } }` with just `x`
- **Simplify conditionals**: Replace `if false { a } else { b }` with just `b`

## 3. Patterns to Preserve

When minimizing, these patterns often need to be preserved:

- **Mutable reference passing**: The `entry(mut n: u32) { func(&mut n) }` pattern is often needed because comptime can't directly pass `&mut` to unconstrained functions
- **Recursive calls with slice mutation**: Bugs often involve recursive functions where a slice is modified (e.g., `push_front`) before the recursive call, and the original value is used after
- **Counter decrements**: If the bug only appears at certain recursion depths, the counter logic needs to be preserved

## Common Pitfalls

- **Slice aliasing in recursion**: After `recursive(c.push_front(x), n)` returns, `c` may have wrong values in Brillig but correct values in comptime
- **Reference semantics differences**: How `&mut` parameters are handled may differ between interpreters
- **Array/slice memory management**: The underlying memory model for dynamic arrays may behave differently
