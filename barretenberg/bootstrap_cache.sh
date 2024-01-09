#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving bb.wasm from remote cache...\033[0m"
extract_repo bb.js \
  /usr/src/barretenberg/cpp/build-wasm/bin ./cpp/build-wasm \
  /usr/src/barretenberg/cpp/build-wasm-threads/bin ./cpp/build-wasm-threads

echo -e "\033[1mBuilding ESM bb.ts...\033[0m"
(cd ts && ./bootstrap.sh esm)
