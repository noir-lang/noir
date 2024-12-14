# Benchmarks 

To generate flamegraphs for the execution of a specific program, execute the following commands:

```shell
./scripts/benchmark_start.sh
cargo bench -p nargo_cli --bench criterion <test-program-name> -- --profile-time=30
./scripts/benchmark_stop.sh
```

Afterwards the flamegraph is available at `target/criterion/<test-program-name>_execute/profile/flamegraph.svg`

Alternatively, omit `<test-program-name>` to run profiling on all test programs defined in [utils.rs](./utils.rs).