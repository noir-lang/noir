# Benchmarks 

To generate flamegraphs for the execution of a specific benchmar, execute the following commands:

```shell
./scripts/benchmark_start.sh
cargo bench -p brillig_vm --bench criterion -- --profile-time=30
./scripts/benchmark_stop.sh
```
