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
cargo +nightly fuzz run init_vs_final --release -- -runs=1000 -max_total_time=60
```