#!/usr/bin/env bash
set -eu

TARGET=${1:-goblin_bench}
FILTER=${2:-./"GoblinFull/1$"}
COMMAND=${2:-./$TARGET}

BUILD_OP_COUNT_TRACK_DIR=build\-op\-count-track

# Move above script dir.
cd $(dirname $0)/..

# Measure the benchmarks with ops counting
cmake --preset op-count-track
cmake --build --preset op-count-track --target $TARGET
# This can be run multithreaded
cd $BUILD_OP_COUNT_TRACK_DIR
./bin/$TARGET --benchmark_filter=$FILTER\
              --benchmark_out=$TARGET.json\
              --benchmark_out_format=json\
              --benchmark_counters_tabular=true\

# If needed, benchmark the basic Fr operations
FIELD_OP_COSTS=field_op_costs.json
if [ ! -f $FIELD_OP_COSTS ]; then
    cd ../
    FIELD_OPS_TARGET=fr_straight_bench
    cmake --preset clang16
    cmake --build --preset clang16 --target $FIELD_OPS_TARGET
    cd build
    ./bin/$FIELD_OPS_TARGET --benchmark_out=../$BUILD_OP_COUNT_TRACK_DIR/$FIELD_OP_COSTS \
                            --benchmark_out_format=json
fi

# Compute the singly-threaded benchmarks for comparison
cd ../
./scripts/benchmark_remote.sh goblin_bench "taskset -c 0 ./goblin_bench --benchmark_filter=Full/1$"

# Analyze the results
python3 ./scripts/compute_field_operations_time.py
