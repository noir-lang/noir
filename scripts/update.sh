#!/bin/bash
set -eu

# Script for running after updating the working copy with remote changes.
# Similar to bootstrap, but more lightweight and oriented towards working on end-to-end.

echo -e '\033[1mRebuild L1 contracts...\033[0m'
(cd l1-contracts && forge build)

echo -e '\n\033[1mUpdate npm deps...\033[0m'
(cd yarn-project && yarn install)

echo -e '\n\033[1mRebuild Noir contracts...\033[0m'
(cd yarn-project/noir-contracts && yarn noir:build:all 2> /dev/null)

echo -e '\n\033[1mRebuild circuits wasm...\033[0m'
(cd circuits/cpp && cmake --build --preset wasm -j --target aztec3-circuits.wasm)