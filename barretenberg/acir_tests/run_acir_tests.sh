#!/bin/bash
# Env var overrides:
#   BIN: to specify a different binary to test with (e.g. bb.js or bb.js-dev).
#   VERBOSE: to enable logging for each test.
set -eu

BIN=${BIN:-../cpp/build/bin/bb}
FLOW=${FLOW:-prove_and_verify}
CRS_PATH=~/.bb-crs
BRANCH=master
VERBOSE=${VERBOSE:-}
TEST_NAMES=("$@")

FLOW_SCRIPT=$(realpath ./flows/${FLOW}.sh)

if [ -f $BIN ]; then
    BIN=$(realpath $BIN)
else
    BIN=$(realpath $(which $BIN))
fi

export BIN CRS_PATH VERBOSE BRANCH

./clone_test_vectors.sh

cd acir_tests

# Convert them to array
SKIP_ARRAY=(diamond_deps_0 workspace workspace_default_member)

function test() {
  cd $1

  set +e
  start=$(date +%s%3N)
  $FLOW_SCRIPT
  result=$?
  end=$(date +%s%3N)
  duration=$((end - start))
  set -eu

  if [ $result -eq 0 ]; then
    echo -e "\033[32mPASSED\033[0m ($duration ms)"
  else
    echo -e "\033[31mFAILED\033[0m"
    exit 1
  fi

  cd ..
}

if [ "${#TEST_NAMES[@]}" -ne 0 ]; then
  for NAMED_TEST in "${TEST_NAMES[@]}"; do
    echo -n "Testing $NAMED_TEST... "
    test $NAMED_TEST
  done
else
  for TEST_NAME in $(find -maxdepth 1 -type d -not -path '.' | sed 's|^\./||'); do
    echo -n "Testing $TEST_NAME... "

    if [[ " ${SKIP_ARRAY[@]} " =~ " $TEST_NAME" ]]; then
      echo -e "\033[33mSKIPPED\033[0m (hardcoded to skip)"
      continue
    fi

    if [[ ! -f ./$TEST_NAME/target/acir.gz || ! -f ./$TEST_NAME/target/witness.gz ]]; then
      echo -e "\033[33mSKIPPED\033[0m (uncompiled)"
      continue
    fi

    test $TEST_NAME
  done
fi
