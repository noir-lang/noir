#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset clang16
cmake --build --preset clang16 --target $BENCHMARK 

cd build
./bin/$BENCHMARK