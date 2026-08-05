---
name: noir-ssa-tests
description: Guide for writing SSA pass unit tests in noirc_evaluator. Use when adding, writing, or reviewing tests for an SSA optimization pass — regression tests for miscompilations, no-op tests, snapshot tests of pass output, or tests that must prove a pass preserves execution semantics.
user-invocable: false
---

# Writing SSA Pass Unit Tests

Tests for SSA passes live in an inline `mod tests` at the bottom of the pass's own
file under `compiler/noirc_evaluator/src/ssa/opt/`. Shared helpers live in
`compiler/noirc_evaluator/src/ssa/opt/mod.rs`.

The default shape of a test is: parse textual SSA with `Ssa::from_str`, run **one**
pass, then assert on the result.

## Assert on behavior, not just shape

A snapshot pins the *shape* of the output SSA. It does not tell you the output is
**correct** — a miscompiling pass produces a stable, plausible-looking snapshot that
happily gets `cargo insta accept`ed. Most SSA bugs worth a regression test are
miscompilations: the program still compiles, it just computes the wrong answer or
stops failing where it should.

So prefer `assert_pass_does_not_affect_execution` and pair it with a snapshot. It
runs the SSA interpreter before and after the pass and asserts the results match,
then hands back the transformed SSA so you can snapshot it too:

```rust
let (ssa, result) =
    assert_pass_does_not_affect_execution(ssa, inputs, |ssa| ssa.flatten_cfg());

// 1. Behavior: the interpreted result is what we expect (and the pass preserved it).
assert_eq!(result.unwrap(), vec![false_value]);

// 2. Shape: the pass did the transformation we intended.
assert_ssa_snapshot!(ssa, @r"
acir(inline) fn main f0 {
  b0(v0: u1):
    enable_side_effects v0
    ...
");
```

This is the single most underused helper in the SSA test suite. Reach for it whenever
the pass under test can change what the program computes — which is nearly always. It
also gives the red step something meaningful to fail on: against the unfixed pass the
execution assertion fails with a different *result*, where a snapshot only shows a
different shape.

## Helpers

All in `ssa::opt` (`compiler/noirc_evaluator/src/ssa/opt/mod.rs`). The execution
helper is private to the `opt` module tree, so it is importable from any pass module
via `use crate::ssa::opt::assert_pass_does_not_affect_execution;` but not from
outside `ssa/opt/`.

| Helper | Use when |
|--------|----------|
| `assert_pass_does_not_affect_execution(ssa, inputs, pass) -> (Ssa, Result<Vec<Value>, InterpreterError>)` | **Default for any pass that can alter semantics.** Interprets `main` before and after `pass`, asserts the results are equal, returns the transformed SSA and the result for further assertions. |
| `assert_ssa_snapshot!(ssa, @r"...")` | Pin the printed output SSA. Normalizes value ids and prints without locations. Accepts `ssa` or `&mut ssa`. |
| `assert_ssa_does_not_change(src, pass)` | Negative test: the pass must leave this input completely alone. |
| `assert_normalized_ssa_equals(ssa, expected)` | Compare against an explicit `&str` rather than an inline snapshot — useful when the expected SSA is the same `src` you already have in a variable. |
| `assert_ssa_does_not_change_after_simplifying(src)` | Assert the parser's instruction simplification leaves the input unchanged (uses `Ssa::from_str_simplifying`). |

`assert_pass_does_not_affect_execution` deep-copies the inputs for each run via
`Value::snapshot_args`. Do not hand-roll `ssa.interpret(inputs.clone())` twice: a
shallow clone shares an array's backing `Shared<Vec<Value>>`, so an in-place
`array_set` in the first run corrupts the second run's inputs and masks exactly the
copy-on-write bug you were testing for.

## Patterns

### Preserved failure, not just preserved value

The helper compares `Result`s, so it also pins *failure* behavior. A pass must not
turn a failing program into a passing one, nor change **which** constraint fails —
a classic loop-invariant-code-motion bug. Assert the specific error:

```rust
let (ssa, execution_result) = assert_pass_does_not_affect_execution(ssa, inputs, |ssa| {
    ssa.loop_invariant_code_motion()
});

let Err(InterpreterError::ConstrainEqFailed { lhs_id, .. }) = execution_result else {
    panic!("Expected ConstrainEqFailed");
};
// Make sure it is the constrain on v8 that failed, not the later one.
assert_eq!(lhs_id.to_u32(), 8);

assert_normalized_ssa_equals(ssa, src);
```

### Path-sensitive bugs need one run per path

If the bug is that one control-flow path observes another path's value, a single
input proves nothing. Run the helper once per input, re-parsing `src` each time,
and snapshot only once:

```rust
let run_unroll = |ssa: Ssa| -> Ssa {
    let (ssa, errors) = try_unroll_loops(ssa);
    assert_eq!(errors.len(), 0, "Unroll should have no errors");
    ssa
};

let (ssa, result) = assert_pass_does_not_affect_execution(
    Ssa::from_str(src).unwrap(), vec![Value::bool(true)], run_unroll);
assert_eq!(result, Ok(vec![Value::field(1_u128.into())]), "v0 = 1 observes the b3 edge");

let (_, result) = assert_pass_does_not_affect_execution(
    Ssa::from_str(src).unwrap(), vec![Value::bool(false)], run_unroll);
assert_eq!(result, Ok(vec![Value::field(2_u128.into())]), "v0 = 0 observes the b4 edge");

assert_ssa_snapshot!(ssa, @r"...");
```

### Prerequisite passes go through the helper too

When the pass under test depends on an earlier analysis, run that analysis through
the same helper rather than around it — it is also a pass that must not change
behavior, and threading the returned SSA keeps the chain honest:

```rust
let (ssa, _) =
    assert_pass_does_not_affect_execution(ssa, inputs.clone(), |ssa| ssa.purity_analysis());

let (_, execution_result) = assert_pass_does_not_affect_execution(ssa, inputs, |ssa| {
    ssa.fold_constants_using_constraints(MIN_ITER)
});
assert!(execution_result.is_ok());
```

### No-op test

When the point of the test is that the pass must *not* fire, the one-liner is enough
— there is no behavior change to catch because there is no change:

```rust
assert_ssa_does_not_change(src, Ssa::remove_truncate_after_range_check);
```

## Building interpreter inputs

`inputs: Vec<Value>` must match `main`'s parameters in order and type, or the
interpreter errors before it reaches your bug.

| Constructor | Notes |
|-------------|-------|
| `Value::bool(b)`, `Value::field(f)`, `Value::u8/u16/u32/u64/u128(n)`, `Value::i8/…/i64(n)` | `crate::ssa::interpreter::value::Value`. Shortest form — prefer these. |
| `Value::from_constant(field, NumericType::unsigned(32))` | Returns `IResult`, needs `.unwrap()`. Use for a type computed at test-write time. |
| `from_constant(2_u128.into(), NumericType::unsigned(32))` | Unwrapping test convenience from `crate::ssa::interpreter::tests`. |
| `Value::array(elements, vec![Type::field()])` | Second argument is the *element* types, not the array type. |

`vec![]` is correct and common when `main` takes no parameters — the helper still
catches passes that break a self-contained program.

## Writing the `src` string

- Get realistic SSA out of a real program rather than inventing it:
  `cargo run -q -p nargo_cli -- compile --silence-warnings --force --show-ssa-pass <label>`.
  Pass labels and the default pipeline order are in
  `compiler/noirc_evaluator/src/ssa.rs`.
- The grammar is defined by `compiler/noirc_evaluator/src/ssa/parser/`. When unsure
  whether syntax is legal (`predicate_pure`, `truncate v0 to 32 bits, max_bit_size: 254`,
  `jmpif v0 then: b1(), else: b2()`), grep for an existing instance or check the
  lexer/token/ast modules — don't guess.
- The runtime prefix matters: `acir(inline)` and `brillig(inline)` take different
  paths in most passes. Test both when the pass behaves differently per runtime.
- `//` comments in `src` are allowed and are stripped before comparison, so use them
  to mark why a block exists ("Make sure the optimization is applied across blocks").
- Use a raw string (`r#"…"#`) when the SSA contains `"`.
- `assert_normalized_ssa_equals` and `assert_ssa_does_not_change` parse and normalize
  the *expected* string too, so it can use any block/value naming. An
  `assert_ssa_snapshot!` body is a literal string comparison against the normalized
  printer output — let `cargo insta` generate it rather than hand-writing ids.
- Write a doc comment on the test naming the issue it regresses, what the bug did,
  and what the assertions prove. Existing tests use
  `/// Regression for noir-claude#1381: …` or `// Regression for #9451`.

## When the execution helper doesn't apply

Fall back to a snapshot-only test, and say why in a comment:

- The pass is not semantics-preserving by design (e.g. it deliberately changes what
  `main` returns), or it operates on SSA the interpreter can't run.
- `main` calls a foreign function — the interpreter returns
  `UnknownForeignFunctionCall`.
- The program is unbounded enough to hit the interpreter's step budget
  (`InterpreterError::OutOfBudget`).
- The pass is purely structural over things `interpret` never observes (e.g. metadata
  or function-ordering passes).

Note that a `Result` mismatch is still a valid assertion when the program *fails*:
you do not need a passing program to use the helper.
