#!/usr/bin/env bash
set -eu

TEST_SRC=${TEST_SRC:-../../noir/noir-repo/test_programs/acir_artifacts}

if [ ! -d acir_tests ]; then
  cp -R $TEST_SRC acir_tests
fi