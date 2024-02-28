import json
from pathlib import Path

PREFIX = Path("build-op-count-time")
IVC_BENCH_JSON = Path("client_ivc_bench.json")
BENCHMARK = "ClientIVCBench/Full/6"

# Single out an independent set of functions accounting for most of BENCHMARK's real_time
to_keep = [
    "construct_mock_function_circuit(t)",
    "construct_mock_folding_kernel(t)",
    "ProverInstance(Circuit&)(t)",
    "ProtogalaxyProver::fold_instances(t)",
    "Decider::construct_proof(t)",
    "ECCVMComposer::create_prover(t)",
    "GoblinTranslatorComposer::create_prover(t)",
    "ECCVMProver::construct_proof(t)",
    "GoblinTranslatorProver::construct_proof(t)",
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
MAX_LABEL_LENGTH = max(len(label) for label in to_keep)
column = {"function": "function", "ms": "ms", "%": "% sum"}
print(
    f"{column['function']:<{MAX_LABEL_LENGTH}}{column['ms']:>8}  {column['%']:>8}")
for key in to_keep:
    time_ms = bench[key]/1e6
    print(f"{key:<{MAX_LABEL_LENGTH}}{time_ms:>8.0f}  {time_ms/sum_of_kept_times_ms:>8.2%}")

# Validate that kept times account for most of the total measured time.
total_time_ms = bench["real_time"]
totals = '\nTotal time accounted for: {:.0f}ms/{:.0f}ms = {:.2%}'
totals = totals.format(
    sum_of_kept_times_ms, total_time_ms, sum_of_kept_times_ms/total_time_ms)
print(totals)
