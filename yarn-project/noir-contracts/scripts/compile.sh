#!/usr/bin/env bash

set -euo pipefail;

# Compiles Aztec.nr contracts in parallel, bubbling any compilation errors

export self_dir=$(dirname "$(realpath $0)")
export COMPILER="$self_dir/../../noir-compiler/dest/cli.js"

build() {
  CONTRACT_NAME=$1
  CONTRACT_FOLDER="$self_dir/../src/contracts/${CONTRACT_NAME}_contract"
  echo "Compiling $CONTRACT_NAME..."
  rm -rf ${CONTRACT_FOLDER}/target

  node --no-warnings "$COMPILER" compile "$CONTRACT_FOLDER"
}

export -f build

# run 4 builds at a time
echo "$@" | xargs -n 1 -P $(nproc) bash -c 'build "$0"'
