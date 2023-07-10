#!/bin/bash
# Env var overrides:
#   BB: to specify a different binary to test with (e.g. bb.js or bb.js-dev).
#   VERBOSE: to enable logging for each test.

set -e

BB=$PWD/${BB:-../cpp/build/bin/bb}
ATBBC=$PWD/acir-to-bberg-circuit/target/release/acir-to-bberg-circuit
CRS_PATH=~/.bb-crs

# Pull down the test vectors from the noir repo, if we don't have the folder already.
if [ ! -d acir_tests ]; then
  rm -rf noir
  git clone --filter=blob:none --no-checkout https://github.com/noir-lang/noir.git
  cd noir
  git sparse-checkout init --cone
  git sparse-checkout set crates/nargo_cli/tests/test_data
  git checkout
  cd ..
  mv noir/crates/nargo_cli/tests/test_data acir_tests
  rm -rf noir
fi

# Get the tool to convert acir to bb constraint buf, if we don't have it already.
if [ ! -f $ATBBC ]; then
  cd acir-to-bberg-circuit
  cargo build --release
  cd ..
fi

cd acir_tests

# Remove excluded and expected-to-fail tests.
rm -rf bit_shifts_runtime comptime_fail poseidonsponge_x5_254 sha2_blocks sha2_byte diamond_deps_0 range_fail

function test() {
  echo -n "Testing $1... "
  cd $1
  if [ ! -f target/translated ]; then
    $ATBBC
    touch target/translated
  fi
  set +e
  if [ -n "$VERBOSE" ]; then
    $BB prove_and_verify -v -c $CRS_PATH
  else
    $BB prove_and_verify -c $CRS_PATH > /dev/null 2>&1
  fi
  result=$?
  set -e
  cd ..

  if [ $result -eq 0 ]; then
    echo -e "\033[32mPASSED\033[0m"
  else
    echo -e "\033[31mFAILED\033[0m"
  fi
}

if [ -n "$1" ]; then
  test $1
else
  for DIR in $(find -maxdepth 1 -type d -not -path '.'); do
    test $DIR
  done
fi
