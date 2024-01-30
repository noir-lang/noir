#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset wasm-bench
cmake --build --preset wasm-bench --target $BENCHMARK 

cd build-wasm-bench
wasmtime run -Wthreads=y -Sthreads=y ./bin/$BENCHMARK