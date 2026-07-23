---
title: Tests
description: Learn how to use Nargo to test your Noir program in a quick and easy way
keywords: [Nargo, testing, Noir, compile, test]
---

You can test your Noir programs using Noir circuits.

Nargo will automatically compile and run any functions which have the decorator `#[test]` on them if
you run `nargo test`.

For example if you have a program like:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test]
fn test_add() {
    assert(add(2,2) == 4);
    assert(add(0,1) == 1);
    assert(add(1,0) == 1);
}
```

Running `nargo test` will test that the `test_add` function can be executed while satisfying all
the constraints which allows you to test that add returns the expected values.

## Tests that should fail

You can write tests that are expected to fail by using the decorator `#[test(should_fail)]`. For example:

```rust
fn add(x: u64, y: u64) -> u64 {
    x + y
}
#[test(should_fail)]
fn test_add() {
    assert(add(2,2) == 5);
}
```

You can be more specific and make it fail with a specific reason by using `should_fail_with = "<the reason for failure>"`:

```rust
fn main(african_swallow_avg_speed: u64) {
    assert(african_swallow_avg_speed == 65, "What is the airspeed velocity of an unladen swallow");
}

#[test]
fn test_king_arthur() {
    main(65);
}

#[test(should_fail_with = "What is the airspeed velocity of an unladen swallow")]
fn test_bridgekeeper() {
    main(32);
}
```

The string given to `should_fail_with` doesn't need to exactly match the failure reason, it just needs to be a substring of it:

```rust
fn main(african_swallow_avg_speed: u64) {
    assert(african_swallow_avg_speed == 65, "What is the airspeed velocity of an unladen swallow");
}

#[test]
fn test_king_arthur() {
    main(65);
}

#[test(should_fail_with = "airspeed velocity")]
fn test_bridgekeeper() {
    main(32);
}
```

## How tests are executed

A common misconception is that `nargo test` compiles everything to Brillig (the unconstrained
bytecode) for speed. It does not. **The way a test is written determines which runtime it runs
in, and Nargo never silently changes that runtime.** There are three possibilities:

| Test | How it's written | Compiled to | Executed by |
|------|------------------|-------------|-------------|
| Constrained test | a plain `#[test] fn` | ACIR (an arithmetic circuit) | the ACVM |
| Unconstrained test | `#[test] unconstrained fn` | [Brillig](../language/unconstrained.md) bytecode | the Brillig VM |
| Comptime test | a `#[test] fn` that exercises a [`comptime`](../language/comptime.md) function or block | nothing — it is interpreted | the compiler's comptime interpreter |

So a plain test exercises your code exactly as it will be proven (as a circuit), while an
`unconstrained fn` test exercises it the way unconstrained code actually runs:

```rust
// Runs as a circuit (ACIR), executed by the ACVM.
#[test]
fn constrained_test() {
    assert(add(2, 2) == 4);
}

// Runs as Brillig, executed by the Brillig VM.
#[test]
unconstrained fn unconstrained_test() {
    assert(add(2, 2) == 4);
}
```

### Why the runtime matters

Keeping these runtimes distinct matters because **Noir code can observe which runtime it is
running in**, so silently moving a test between them could hide real bugs.

- [`std::runtime::is_unconstrained()`](../libraries/standard_library/is_unconstrained.md)
  returns whether the current context is unconstrained, and it is resolved at compile time. A
  library that does `if is_unconstrained() { ... } else { ... }` therefore takes a *different
  branch* depending on the runtime. A constrained test exercises the constrained branch; the
  same test compiled to Brillig would only ever exercise the unconstrained branch, so a bug
  living in the constrained branch would go unnoticed.

- Values returned from unconstrained code are **not** automatically constrained when they
  flow back into ACIR — it is up to you to write the checks that verify them, and an ACIR
  function that forgets to do so is under-constrained even though it executes fine. Crucially,
  these verification checks are often written to run *only* in a constrained context and
  skipped in Brillig (commonly by branching on
  [`std::runtime::is_unconstrained()`](../libraries/standard_library/is_unconstrained.md), see
  [Unconstrained functions](../language/unconstrained.md)). So if you want to test that an
  ACIR function actually constrains its results correctly, the test must run in an ACIR
  context — running it as Brillig skips exactly the checks you are trying to exercise.

The practical rule: test your code in the runtime it will actually be compiled to. If a
function is constrained in production, keep its test constrained.

### Running constrained tests as unconstrained

Sometimes you *do* want to additionally check how a constrained test behaves when run as if it
were unconstrained — for example to compare results between the two runtimes. The
`--force-brillig` flag forces every function to be compiled to Brillig for the run:

```bash
nargo test --force-brillig
```

This is an explicit, opt-in override, not the default, and it changes what the test proves
(see [Why the runtime matters](#why-the-runtime-matters) above). Use it as an *additional*
check, not as a replacement for the normal constrained run.

### Testing comptime code

If you want to test a [`comptime`](../language/comptime.md) function, call it from a `comptime`
context inside a regular test. The body runs in the compiler's comptime interpreter rather
than being compiled to a circuit or to Brillig:

```rust
comptime fn double(x: u32) -> u32 {
    x * 2
}

#[test]
fn test_double() {
    comptime {
        assert_eq(double(2), 4);
    }
}
```

Note that `is_unconstrained()` returns `true` in a `comptime` context, since comptime
evaluation follows unconstrained semantics.

## Fuzz tests

A `#[test]` function that takes arguments is run as a fuzz test: instead of executing it once,
`nargo test` generates and mutates inputs looking for a set of arguments that makes it fail.

```rust
#[test]
fn test_basic(a: Field, b: Field) {
    assert(a + b == b + a);
}
```

Fuzz tests support `#[test(should_fail)]`, `#[test(should_fail_with = "...")]` and
`#[test(only_fail_with = "...")]` in the same way as ordinary tests. The underlying engine, the
dedicated `nargo fuzz` command, and differential fuzzing against an oracle are all described in
the [Fuzzer](./fuzzer.md) documentation.

The following fuzzing-specific options are available on `nargo test`:

```text
--no-fuzz
    Do not run fuzz tests (tests that have arguments)
--only-fuzz
    Only run fuzz tests (tests that have arguments)
--corpus-dir <CORPUS_DIR>
    If given, load/store fuzzer corpus from this folder
--minimized-corpus-dir <MINIMIZED_CORPUS_DIR>
    If given, perform corpus minimization instead of fuzzing and store results in the given folder
--fuzzing-failure-dir <FUZZING_FAILURE_DIR>
    If given, store the failing input in the given folder
--fuzz-timeout <FUZZ_TIMEOUT>
    Maximum time in seconds to spend fuzzing (default: 1 second)
--fuzz-max-executions <FUZZ_MAX_EXECUTIONS>
    Maximum number of executions to run for each fuzz test (default: 100000)
--fuzz-show-progress
    Show progress of fuzzing (default: false)
```

By default, the fuzzing corpus is saved in a temporary directory, but this can be changed. This allows you to resume fuzzing from the same corpus if the process is interrupted, if you want to run continuous fuzzing on your corpus, or if you want to use previous failures for regression testing.

## Coverage

Pass `--coverage` to `nargo test` to measure which lines of your library are exercised by its tests.

```bash
nargo test --coverage
```

Coverage is collected by running each argument-free test through the comptime interpreter and recording which expressions are evaluated. Fuzz tests (tests with arguments) are not included.

After the run, one `lcov.info` file is written per package:

| Layout | Output path |
|--------|-------------|
| Single-package | `target/coverage/lcov.info` |
| Multi-package workspace | `target/coverage/<package-name>/lcov.info` |

The output directory can be overridden with `--coverage-dir <DIR>`. The same single-vs-multi-package nesting applies relative to the chosen directory.

The files use [lcov trace file format](https://ltp.sourceforge.net/coverage/lcov/geninfo.1.php) and are compatible with standard coverage tools. For in-editor highlighting, install the [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) VS Code extension, which automatically picks up `lcov.info` files and highlights covered (green) and uncovered (red) lines in the editor.

## Mocking Oracles

When testing code that calls [oracles](../language/oracles.mdx), you can use `OracleMock` from `std::test` to provide return values without needing an actual oracle server.

### Basic usage

```rust
use std::test::OracleMock;

#[oracle(get_price)]
unconstrained fn get_price_oracle() -> Field {}

unconstrained fn get_price() -> Field {
    get_price_oracle()
}

#[test]
fn test_with_mock() {
    // Safety: testing context, return value is checked below
    unsafe {
        OracleMock::mock("get_price").returns(100);
        assert_eq(get_price(), 100);
    }
}
```

### Matching parameters

You can make a mock only respond to calls with specific parameters using `with_params`. The parameters are passed as a tuple:

```rust
#[oracle(get_balance)]
unconstrained fn get_balance_oracle(account: Field) -> Field {}

unconstrained fn get_balance(account: Field) -> Field {
    get_balance_oracle(account)
}

#[test]
fn test_multiple_accounts() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("get_balance").with_params((1,)).returns(100);
        OracleMock::mock("get_balance").with_params((2,)).returns(200);

        assert_eq(get_balance(1), 100);
        assert_eq(get_balance(2), 200);
    }
}
```

### Limiting invocations

Use `times` to limit how many times a mock can be invoked. After the limit is reached, the next mock in creation order is used:

```rust
#[test]
fn test_changing_values() {
    // Safety: testing context
    unsafe {
        OracleMock::mock("get_price").returns(100).times(2);
        OracleMock::mock("get_price").returns(200);

        assert_eq(get_price(), 100);  // First mock (call 1 of 2)
        assert_eq(get_price(), 100);  // First mock (call 2 of 2)
        assert_eq(get_price(), 200);  // First mock exhausted, falls through to second
    }
}
```

### OracleMock API reference

All methods are unconstrained.

| Method | Description |
|--------|-------------|
| `OracleMock::mock("name")` | Create a mock for the named oracle. Returns `Self` for chaining. |
| `.with_params(params)` | Only match calls with these parameters (passed as a tuple). Returns `Self`. |
| `.returns(value)` | Set the return value for matched calls. Returns `Self`. |
| `.times(n)` | Limit the mock to `n` invocations, after which it is skipped. Returns `Self`. |
| `.times_called()` | Returns how many times this mock has been invoked. |
| `.get_last_params()` | Returns the parameters from the most recent matching call. |
| `.clear()` | Remove this mock so it no longer matches any calls. |
