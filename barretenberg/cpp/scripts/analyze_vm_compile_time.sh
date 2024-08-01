#!/usr/bin/env bash
# This script summarises the compilation time for the vm
# The summary json file is outputted to $BUILD_DIR/avm_compilation_summary.json
# it takes in two params the preset(e.g. clang16, clang16-dbg) and a target (e.g. bb, vm) 
# it can be called like this => ./analyze_vm_compile_time.sh clang16 bb
set -eu
# So we can glob recursively
shopt -s globstar

PRESET="${1:-wasm-threads}"
TARGET="${2:-barretenberg.wasm}"

BUILD_DIR=build-$PRESET-compiler-profile

cd $(dirname $0)/..

# Run the analyse script if we dont already have the specific directory
if [ ! -d $BUILD_DIR ]; then 
    echo -e "\n$BUILD_DIR not found, running $(dirname $0)/analyze_compile_time.sh $PRESET $TARGET"
    ./scripts/analyze_compile_time.sh $PRESET $TARGET
else 
    echo -e "\n$BUILD_DIR found, using existing results"
fi 

# Run summary analysis
cd build-$PRESET-compiler-profile
pushd src/barretenberg/vm/CMakeFiles/vm_objects.dir/ > /dev/null 2>&1
# Process vm json profiling files and "simplify" them in a summary
jq -cn 'inputs | .traceEvents | map(select(.name == "Total ExecuteCompiler")) | map({ name: input_filename, "time(ms)": .args."avg ms"})' **/**.cpp.json | jq -s add > avm_compilation_summary.json
# Compute total time
TOTAL_TIME=$(jq 'map(."time(ms)") | add' avm_compilation_summary.json)
echo -e "\nVM Total Compilation time(ms): $TOTAL_TIME"
popd  > /dev/null 2>&1
mv ./src/barretenberg/vm/CMakeFiles/vm_objects.dir/avm_compilation_summary.json .
echo -e "Summary file outputted to $BUILD_DIR/avm_compilation_summary.json"
