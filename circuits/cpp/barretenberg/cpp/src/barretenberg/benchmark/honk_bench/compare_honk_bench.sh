#!/bin/bash

# This script is used to compare the results of honk_bench between baseline (master) and
# the branch from which the script is run. Simply check out the branch of interest, ensure 
# it is up to date with local master, and run the script.

echo -e '\nComparing Honk benchmarks between master and current branch:'
# Set some directories
BASE_DIR="$HOME/barretenberg/cpp"
BUILD_DIR="$BASE_DIR/build-bench"
BENCH_RESULTS_DIR="$BASE_DIR/tmp_bench_results"
BENCH_TOOLS_DIR="$BUILD_DIR/_deps/benchmark-src/tools"

# Install requirements (numpy + scipy) for comparison script if necessary.
# Note: By default, installation will occur in $HOME/.local/bin.
pip3 install -r $BUILD_DIR/_deps/benchmark-src/requirements.txt

# Create temporary directory for honk_bench results (json)
cd $BASE_DIR
mkdir $BENCH_RESULTS_DIR

# Checkout master, run honk_bench, save results in json format
echo -e '\nConfiguring and building honk_bench in master branch..'
git checkout master > /dev/null
rm -rf $BUILD_DIR
cmake --preset bench > /dev/null && cmake --build --preset bench --target honk_bench > /dev/null
MASTER_HONK_BENCH_RESULTS="$BENCH_RESULTS_DIR/honk_bench_results_master.json"
echo -e '\nRunning honk_bench in master..'
bin/honk_bench --benchmark_format=json > $MASTER_HONK_BENCH_RESULTS

# Checkout working branch (-), run honk_bench, save results in json format
echo -e '\nConfiguring and building honk_bench in current feature branch..'
git checkout -
rm -rf $BUILD_DIR
cmake --preset bench > /dev/null && cmake --build --preset bench --target honk_bench > /dev/null
BRANCH_HONK_BENCH_RESULTS="$BENCH_RESULTS_DIR/honk_bench_results_branch.json"
echo -e '\nRunning honk_bench in feature branch..'
bin/honk_bench --benchmark_format=json > $BRANCH_HONK_BENCH_RESULTS

# Call compare.py on the results (json) to get high level statistics. 
# See docs at https://github.com/google/benchmark/blob/main/docs/tools.md for more details.
$BENCH_TOOLS_DIR/compare.py benchmarks $MASTER_HONK_BENCH_RESULTS $BRANCH_HONK_BENCH_RESULTS

# Delete the temporary results directory and its contents
rm -r $BENCH_RESULTS_DIR