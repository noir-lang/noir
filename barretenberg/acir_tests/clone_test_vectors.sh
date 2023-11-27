#!/bin/bash
set -eu

# Pull down the test vectors from the noir repo, if we don't have the folder already.
if [ ! -d acir_tests ]; then
  if [ -n "${TEST_SRC:-}" ]; then
    cp -R $TEST_SRC acir_tests
  else
    rm -rf noir
    git clone -b $BRANCH --filter=blob:none --no-checkout https://github.com/noir-lang/noir.git
    cd noir
    git sparse-checkout init --cone
    git sparse-checkout set test_programs/acir_artifacts
    git checkout
    cd ..
    mv noir/test_programs/acir_artifacts acir_tests
    rm -rf noir
  fi
fi
