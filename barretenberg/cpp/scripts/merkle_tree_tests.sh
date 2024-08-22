#!/usr/bin/env bash

set -e

# run commands relative to parent directory
cd $(dirname $0)/..

DEFAULT_TESTS=PersistedIndexedTreeTest.*:PersistedAppendOnlyTreeTest.*:LMDBStoreTest.*
TEST=${1:-$DEFAULT_TESTS}
PRESET=${PRESET:-clang16}

cmake --build --preset $PRESET --target crypto_merkle_tree_tests
./build/bin/crypto_merkle_tree_tests --gtest_filter=$TEST
