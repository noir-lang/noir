#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}
COMMAND=${2:-./bin/$BENCHMARK}
HARDWARE_CONCURRENCY=${HARDWARE_CONCURRENCY:-16}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset wasm-threads
cmake --build --preset wasm-threads --target $BENCHMARK

cd build-wasm-threads
# Consistency with _wasm.sh targets / shorter $COMMAND.
cp ./bin/$BENCHMARK .
wasmtime run --env HARDWARE_CONCURRENCY=$HARDWARE_CONCURRENCY -Wthreads=y -Sthreads=y --dir=.. $COMMAND