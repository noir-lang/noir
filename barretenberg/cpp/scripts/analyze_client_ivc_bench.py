import json
from pathlib import Path

PREFIX = Path("build-op-count-time")
IVC_BENCH_JSON = Path("client_ivc_bench.json")
BENCHMARK = "ClientIVCBench/Full/6"

# Single out an independent set of functions accounting for most of BENCHMARK's real_time
to_keep = [
    "construct_circuits(t)",
    "ProverInstance(Circuit&)(t)",
    "ProtogalaxyProver::fold_instances(t)",
    "Decider::construct_proof(t)",
    "ECCVMProver(CircuitBuilder&)(t)",
    "ECCVMProver::construct_proof(t)",
    "TranslatorProver::construct_proof(t)",
    "Goblin::merge(t)"
]
with open(PREFIX/IVC_BENCH_JSON, "r") as read_file:
    read_result = json.load(read_file)
    for _bench in read_result["benchmarks"]:
        if _bench["name"] == BENCHMARK:
            bench = _bench
bench_components = dict(filter(lambda x: x[0] in to_keep, bench.items()))

# For each kept time, get the proportion over all kept times.
sum_of_kept_times_ms = sum(float(time)
                           for _, time in bench_components.items())/1e6
max_label_length = max(len(label) for label in to_keep)
column = {"function": "function", "ms": "ms", "%": "% sum"}
print(
    f"{column['function']:<{max_label_length}}{column['ms']:>8}  {column['%']:>8}")
for key in to_keep:
    if key not in bench:
        time_ms = 0
    else:
        time_ms = bench[key]/1e6
    print(f"{key:<{max_label_length}}{time_ms:>8.0f}  {time_ms/sum_of_kept_times_ms:>8.2%}")

# Validate that kept times account for most of the total measured time.
total_time_ms = bench["real_time"]
totals = '\nTotal time accounted for: {:.0f}ms/{:.0f}ms = {:.2%}'
totals = totals.format(
    sum_of_kept_times_ms, total_time_ms, sum_of_kept_times_ms/total_time_ms)
print(totals)

print("\nMajor contributors:")
print(
    f"{column['function']:<{max_label_length}}{column['ms']:>8}  {column['%']:>7}")
for key in ['commit(t)', 'compute_combiner(t)', 'compute_perturbator(t)', 'compute_univariate(t)']:
    if key not in bench:
        time_ms = 0
    else:
        time_ms = bench[key]/1e6
    print(f"{key:<{max_label_length}}{time_ms:>8.0f} {time_ms/sum_of_kept_times_ms:>8.2%}")

print('\nBreakdown of ProtogalaxyProver::fold_instances:')
protogalaxy_round_labels = [
    "ProtoGalaxyProver_::preparation_round(t)",
    "ProtoGalaxyProver_::perturbator_round(t)",
    "ProtoGalaxyProver_::combiner_quotient_round(t)",
    "ProtoGalaxyProver_::accumulator_update_round(t)"
]
max_label_length = max(len(label) for label in protogalaxy_round_labels)
for key in protogalaxy_round_labels:
    if key not in bench:
        time_ms = 0
    else:
        time_ms = bench[key]/1e6
    total_time_ms = bench["ProtogalaxyProver::fold_instances(t)"]/1e6
    print(f"{key:<{max_label_length}}{time_ms:>8.0f}  {time_ms/total_time_ms:>8.2%}")


# Extract a set of components from the benchmark data and display timings and relative percentages 
def print_contributions(prefix, ivc_bench_json, bench_name, components):

    # Read JSON file and extract benchmark
    try:
        with open(prefix / ivc_bench_json, "r") as read_file:
            read_result = json.load(read_file)
            bench = next((_bench for _bench in read_result["benchmarks"] if _bench["name"] == bench_name), None)
            if not bench:
                raise ValueError(f"Benchmark '{bench_name}' not found in the JSON file.")
    except FileNotFoundError:
        print(f"File not found: {prefix / ivc_bench_json}")
        return
    
    # Filter and sum up kept times
    bench_components = {key: bench[key] for key in components if key in bench}
    sum_of_kept_times_ms = sum(float(time) for time in bench_components.values()) / 1e6
    print(f"Total time accounted for (ms): {sum_of_kept_times_ms:>8.0f}")

    # Print results
    max_label_length = max(len(label) for label in components)
    column_headers = {"operation": "operation", "ms": "ms", "%": "% sum"}
    print(f"{column_headers['operation']:<{max_label_length}}{column_headers['ms']:>8}  {column_headers['%']:>8}")

    for key in components:
        time_ms = bench_components.get(key, 0) / 1e6
        percentage = time_ms / sum_of_kept_times_ms if sum_of_kept_times_ms > 0 else 0
        print(f"{key:<{max_label_length}}{time_ms:>8.0f}  {percentage:>8.2%}")

relations = [
    "Arithmetic::accumulate(t)",
    "Permutation::accumulate(t)",
    "Lookup::accumulate(t)",
    "DeltaRange::accumulate(t)",
    "Elliptic::accumulate(t)",
    "Auxiliary::accumulate(t)",
    "EccOp::accumulate(t)",
    "DatabusRead::accumulate(t)",
    "PoseidonExt::accumulate(t)",
    "PoseidonInt::accumulate(t)",
]

print('\nRelation contributions (times to be interpreted relatively):')
print_contributions(PREFIX, IVC_BENCH_JSON, BENCHMARK, relations)

commitments = [
    "COMMIT::wires(t)",
    "COMMIT::z_perm(t)",
    "COMMIT::databus(t)",
    "COMMIT::ecc_op_wires(t)",
    "COMMIT::lookup_inverses(t)",
    "COMMIT::databus_inverses(t)",
    "COMMIT::lookup_counts_tags(t)",
]

print('\nCommitment contributions:')
print_contributions(PREFIX, IVC_BENCH_JSON, BENCHMARK, commitments)