# Testing Noir

The following is a list of locations where we can find tests, and the ways to run them.

## Lexing and parsing

The lexer unit tests can be found in [lexer.rs](compiler/noirc_frontend/src/lexer/lexer.rs), while the parser unit tests are spread out under the [parser module](compiler/noirc_frontend/src/parser/parser).

## Frontend

The frontend has [common compilation tests](compiler/noirc_frontend/src/tests.rs) as well as [tests per language feature](compiler/noirc_frontend/src/tests).

## Comptime

The [comptime tests](compiler/noirc_frontend/src/hir/comptime/tests.rs) include snippets of Noir code executed via interpreter.

## Formatter

Tests for `nargo fmt` include expected [inputs and outputs](tooling/nargo_fmt/tests), with test cases generated in [build.rs](tooling/nargo_fmt/build.rs) and executed in [execute.rs](tooling/nargo_fmt/tests/execute.rs).

## Noir `stdlib`

Any `#[test]` included in Noir's `std` library is automatically executed by [stdlib-tests.rs](tooling/nargo_cli/tests/stdlib-tests.rs).

Some functions in `std` are covered by property based tests in [stdlib-props.rs](tooling/nargo_cli/tests/stdlib-props.rs), which compare the execution of Noir snippets them against Rust equivalents.

Tests for a specific Noir library can be executed using some extended CLI options. For example the following command runs the tests in the `array` module:

```shell
cargo test -p nargo_cli --test stdlib-tests -- run_stdlib_tests array
```

Tests are executed with multiple compilation options, so there will be multiple groups of results about the same tests.

## Integration tests

The [test_programs](test_programs) directory contains sub-directories with small Noir projects, organized according to expected outcome.

For example the programs in [execution_success](test_programs/execution_success) are expected to succeed with `nargo execute` and return a result. These programs have a `Prover.toml` file with the inputs expected by the circuit in the project's `main.rs` file. If the `Prover.toml` file has a `return` item, it will be compared to the value returned by the program.

By contrast the `compile_success_*` variants are run with `nargo compile` and require no input files.

There are two kinds of projects:
* some cover a specific feature, e.g. [break_and_continue](/Users/aakoshh/Work/aztec/noir/test_programs/execution_success/break_and_continue)
* others are regression test for a specific bug ticket, in which case their name is typically `regression_<ticket-number>`, or `<feature>_regression_<ticket-number>`.

Similar to the format tests, integration tests cases are generated in [build.rs](tooling/nargo_cli/build.rs) and executed by [execute.rs](tooling/nargo_cli/tests/execute.rs), under the `nargo_cli` crate.

The following command executes all integration tests:

```shell
cargo test -p nargo_cli --test execute
```

Similar to the `stdlib` tests, some appear multiple times due to being executed with different compilation options. To only execute them once, by the default configuration, we can use the following variant:

```shell
cargo test -p nargo_cli --test execute forcebrillig_false_inliner_i64_min
```

A specific test can be similarly executed by name, for example the following command executes all projects that include `sha256` in their name:

```shell
cargo test -p nargo_cli --test execute sha256
```

When new tests are added, their output needs to be checked in using [cargo insta](https://crates.io/crates/cargo-insta), so any change in return values, bytecode or printed output causes a test failure until accepted.

## SSA passes

The SSA passes can be found in the [opt](compiler/noirc_evaluator/src/ssa/opt) module. Each module contains unit tests specific to their pass.

We can use the `--show-ssa` CLI option to print the SSA after each pass, or the `--show-ssa-pass` CLI option to limit the output to a specific passes. For example the following command would show SSA passes with either "Defunct" or "Simple" in their labels:

```console
$ cargo run -q -p nargo_cli -- compile --silence-warnings --force --show-ssa-pass Simple --show-ssa-pass Defunct
After Defunctionalization (1):
...

After Inlining simple functions (1):
brillig(inline) fn main f0 {
...
```

The list of passes in the default pipeline can be found in [ssa.rs](compiler/noirc_evaluator/src/ssa.rs), along with their labels. Some of them appear multiple times; the number in parentheses helps differentiate between repeated executions of the same pass.

There can be integration and regression tests which cover some of the SSA passes. To help find out which, we can use the [ssa_pass_impact.rs](tooling/nargo_cli/examples/ssa_pass_impact.rs) tool, which lists the tests under `execution_success` that have the most dramatic change during the passes matching the `--ssa-pass` label:

```shell
cargo run -p nargo_cli --example ssa_pass_impact -- --ssa-pass "Removing Unreachable Functions"
```

## Fuzz tests

The SSA passes, ACIR and Brillig are covered by [fuzz tests](tooling/ast_fuzzer/README.md).

The following commands executes the fuzz targets for a limited amount of time to get quick feedback during CI and development:

```shell
cargo test -p noir_ast_fuzzer --test smoke
cargo test -p noir_ast_fuzzer_fuzz arbtest
```

Should it find any problems, the test would print a _seed_ which we can use to replicate the test, for example:

```shell
NOIR_AST_FUZZER_SEED=0x6819c61400001000 cargo test -p noir_ast_fuzzer_fuzz comptime_vs_brillig
```

When the `NOIR_AST_FUZZER_SEED` is present, or when there is a non-panicky error, the tests will print out the Noir AST and the `Prover.toml` file which can be used to reproduce the problem as an integration test.
