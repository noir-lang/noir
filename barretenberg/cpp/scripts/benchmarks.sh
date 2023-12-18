#!/usr/bin/env bash
set -eu

# Move above script dir.
cd $(dirname $0)/..

# Configure and build.
cmake --preset clang16
cmake --build --preset clang16

cd build

# github markdown style, works in comments and descriptions
echo -e "<details><summary>Standard Plonk</summary>"
echo -e '\n```'
./bin/standard_plonk_bench | tee standard_plonk_bench.out
echo -e '```\n'
echo -e "</details>"
echo -e "<details><summary>Ultra Honk Round Breakdown</summary>"
echo -e '\n```'
./bin/ultra_honk_rounds_bench | tee ultra_honk_rounds_bench.out
echo -e '```\n'
echo -e "</details>"
echo -e "<details><summary>Ultra Plonk Round Breakdown</summary>"
echo -e '\n```'
./bin/ultra_plonk_rounds_bench | tee ultra_plonk_rounds_bench.out
echo -e '```\n'
echo -e "</details>"
echo -e "<details><summary>Ultra Honk</summary>"
echo -e '\n```'
./bin/ultra_honk_bench | tee ultra_honk_bench.out
echo -e '```\n'
echo -e "</details>"
echo -e "<details><summary>Ultra Plonk</summary>"
echo -e '\n```'
./bin/ultra_plonk_bench | tee ultra_plonk_bench.out
echo -e '```\n'
echo -e "</details>"
