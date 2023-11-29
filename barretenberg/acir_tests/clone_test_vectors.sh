#!/bin/bash
set -eu

TEST_SRC=${TEST_SRC:-../../noir/tooling/nargo_cli/tests/acir_artifacts}

if [ ! -d acir_tests ]; then
  cp -R $TEST_SRC acir_tests
fi