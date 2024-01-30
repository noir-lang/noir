#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}
COMMAND=${2:-./bin/$BENCHMARK}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset clang16
cmake --build --preset clang16 --target $BENCHMARK 

cd build
# Consistency with _wasm.sh targets / shorter $COMMAND.
cp ./bin/$BENCHMARK .
$COMMAND