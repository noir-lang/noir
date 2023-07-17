#!/bin/bash
# Env var overrides:
#   BB: to specify a different binary to test with (e.g. bb.js or bb.js-dev).
#   VERBOSE: to enable logging for each test.

set -e

BB=$PWD/${BB:-../cpp/build/bin/bb}
CRS_PATH=~/.bb-crs
BRANCH=master

# Pull down the test vectors from the noir repo, if we don't have the folder already.
if [ ! -d acir_tests ]; then
  if [ -n "$TEST_SRC" ]; then
    cp -R $TEST_SRC acir_tests
  else
    rm -rf noir
    git clone -b $BRANCH --filter=blob:none --no-checkout https://github.com/noir-lang/noir.git
    cd noir
    git sparse-checkout init --cone
    git sparse-checkout set crates/nargo_cli/tests/test_data
    git checkout
    cd ..
    mv noir/crates/nargo_cli/tests/test_data acir_tests
    rm -rf noir
  fi
fi

cd acir_tests

# Remove excluded and expected-to-fail tests.
rm -rf bit_shifts_runtime comptime_fail poseidonsponge_x5_254 sha2_blocks sha2_byte diamond_deps_0 range_fail workspace*

function test() {
  echo -n "Testing $1... "
  cd $1
  set +e
  if [ -n "$VERBOSE" ]; then
    $BB prove_and_verify -v -c $CRS_PATH
  else
    $BB prove_and_verify -c $CRS_PATH > /dev/null 2>&1
  fi
  result=$?
  set -e

  if [ $result -eq 0 ]; then
    echo -e "\033[32mPASSED\033[0m"
  else
    echo -e "\033[31mFAILED\033[0m"
    # Run again verbose.
    $BB prove_and_verify -v -c $CRS_PATH
    exit 1
  fi

  cd ..
}

if [ -n "$1" ]; then
  test $1
else
  for DIR in $(find -maxdepth 1 -type d -not -path '.'); do
    test $DIR
  done
fi
