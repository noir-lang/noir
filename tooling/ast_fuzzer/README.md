# AST Fuzzer

The AST Fuzzer generates arbitrary _monomorphized_ AST `Program` instances and
executes them with various compilation strategies. For example we can:

* compare the execution of and AST with minimal SSA passes versus the normal SSA flow, to test that all SSA transformations preserve behavior
* perform random mutations on the AST that should preserve behavior, and check that the execution result with the full SSA flow is the same
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
init_vs_final
orig_vs_mutant
```

and execute it with some time or execution limits:

```shell
cargo +nightly fuzz run init_vs_final -- -runs=1000 -max_total_time=60 -max_len=1048576
```

If there is an error, `cargo fuzz` will capture the artifacts required for a repeated run under the `artifacts` directory, and will print the command to run it again, which can be done with something like this:

```shell
NOIR_AST_FUZZER_DEBUG=1 cargo +nightly fuzz run -O init_vs_final fuzz/artifacts/init_vs_final/crash-fa077fcded882761fcf273eda7f429a833a80a7d
```

Note that `cargo fuzz` requires `nightly` build, which can be either turned on with the `cargo +nightly` flag, or by running `rustup default nightly`. Also note that `cargo fuzz run` automatically creates a `--release` build, there is no need for an explicit flag to be passed.

The `NOIR_AST_FUZZER_DEBUG` env var can be used to print the AST before compilation.