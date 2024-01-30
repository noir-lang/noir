#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}
COMMAND=${2:-./bin/$BENCHMARK}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset wasm-bench
cmake --build --preset wasm-bench --target $BENCHMARK 

cd build-wasm-bench
# Consistency with _wasm.sh targets / shorter $COMMAND.
cp ./bin/$BENCHMARK .
wasmtime run -Wthreads=y -Sthreads=y $COMMAND