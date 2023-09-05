#!/bin/sh
set -eu

if [ -n "$VERBOSE" ]; then
  $BIN prove_and_verify -v -c $CRS_PATH -b ./target/acir.gz
else
  $BIN prove_and_verify -c $CRS_PATH -b ./target/acir.gz > /dev/null 2>&1
fi