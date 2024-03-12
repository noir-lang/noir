#!/usr/bin/env bash
set -eu

BENCHMARK=${1:-goblin_bench}
COMMAND=${2:-./$BENCHMARK}
PRESET=${3:-clang16}
BUILD_DIR=${4:-build}


# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset $PRESET
cmake --build --preset $PRESET --target $BENCHMARK 

cd $BUILD_DIR
# Consistency with _wasm.sh targets / shorter $COMMAND.
cp ./bin/$BENCHMARK .
$COMMAND