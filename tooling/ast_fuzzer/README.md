# AST Fuzzer

The AST Fuzzer generates arbitrary _monomorphized_ AST `Program` instances and
executes them with various compilation strategies. For example we can:

* compare the execution of and AST with minimal SSA passes versus the normal SSA flow, to test that all SSA transformations preserve behavior
* perform metamorphic transformations on the AST that should preserve behavior, and check that the execution result with the full SSA flow is the same
* compare ACIR and Brillig executions of the same AST

The following command can be used to print some random AST to the console:

```shell
cargo run -p noir_ast_fuzzer --example sample
```

To run the fuzzer, pick one of the available targets:

```console
$ cd tooling/ast_fuzzer
$ cargo fuzz list
acir_vs_brillig
comptime_vs_brillig
min_vs_full
orig_vs_morph
```

and execute it with some time or execution limits:

```shell
cargo +nightly fuzz run acir_vs_brillig -- -runs=1000 -max_total_time=60 -max_len=1048576
```

If there is an error, `cargo fuzz` will capture the artifacts required for a repeated run under the `artifacts` directory, and will print the command to run it again, which can be done with something like this:

```shell
cargo +nightly fuzz run acir_vs_brillig fuzz/artifacts/acir_vs_brillig/crash-9270e36f612ed9022ede3496c97c24cebb6e2301
```

Note that `cargo fuzz` requires `nightly` build, which can be either turned on with the `cargo +nightly` flag, or by running `rustup default nightly`. Also note that `cargo fuzz run` automatically creates a `--release` build, there is no need for an explicit flag to be passed.

If the execution fails, the output will include the AST, the inputs, and the ACIR/Brillig opcodes.

## `arbtest`

To get quick feedback about whether there are any easy-to-discover bugs, we can run the following test:

```shell
cargo test -p noir_ast_fuzzer_fuzz arbtest
```

Unlike `cargo fuzz`, these don't "ramp up" the complexity of the code, but go full tilt from the beginning, and only run for a limited amount of time (e.g. 10 seconds).

Upon failure they print a hexadecimal `seed`, which can be used with the `NOIR_AST_FUZZER_SEED` env var to replicate the error.

If the compiler crashes during the generation of the SSA artifacts, the problematic program will be printed during attempts to reproduce the issue using a seed. The printing of all inputs can be turned on by setting `RUST_LOG=debug`.

## Minimizing Noir

At the moment test failures end up with one or two Noir-like AST printed on the console, with the corresponding ABI formatted inputs.
We can turn these into `nargo` projects to replicate the problem that causes the error.

Especially with the `arbtest` mentioned above, the Noir AST can be big, certainly much larger than what we would want to put in a bug ticket. If that is the case, we can try minimizing the example with the [Noir Minimizer](./minimizer/README.md) tool.