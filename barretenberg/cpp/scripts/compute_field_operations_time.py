import json
from pathlib import Path

PREFIX = Path("build-op-count")
OPS_BENCH = Path("field_op_costs.json")
GOBLIN_BENCH_JSON = Path("goblin_bench.json")
BENCHMARK = "GoblinBench/GoblinFull/1"

# We will populate time per operation for a subset of the operations
# For accurate counting, we must select operations that do not call other
# operations on the list.
ns_per_op = {}
to_keep = [
    "asm_add_with_coarse_reduction",
    "asm_conditional_negate",
    "asm_mul_with_coarse_reduction",
    # "asm_reduce_once",
    "asm_self_add_with_coarse_reduction",
    "asm_self_mul_with_coarse_reduction",
    "asm_self_reduce_once",
    "asm_self_sqr_with_coarse_reduction",
    "asm_self_sub_with_coarse_reduction",
    "asm_sqr_with_coarse_reduction",
    # "mul",
    # "self_mul",
    # "add",
    # "self_add",
    # "sub",
    # "self_sub",
    # "invert", // mostly just self_sqr and *=
    # "self_neg",
    # "self_reduce_once",
    # "self_to_montgomery_form",
    # "self_sqr",
    # "sqr",
]

# read the measuremens of the basic field operations
with open(PREFIX/OPS_BENCH, "r") as read_file:
    read_result = json.load(read_file)
    for bench in read_result["benchmarks"]:
        if bench["name"] in to_keep:
            ns_per_op[bench["name"]] = bench["real_time"]

with open(PREFIX/GOBLIN_BENCH_JSON, "r") as read_file:
    read_result = json.load(read_file)
    for bench in read_result["benchmarks"]:
        if bench["name"] == BENCHMARK:
            mct = bench

total_time = 0

for (key, time) in ns_per_op.items():
    full_key = "fr::" + key
    if (full_key in mct.keys()):
        count = int(mct[full_key])
        if (count is not None):
            print(f'aggregating { count } counts of {key} at time {ns_per_op[key]} ns.')
            total_time += count * ns_per_op[key]

total_time /= 1e9

print(f'Time spent on field ops: {round(total_time, 3)}s.')
