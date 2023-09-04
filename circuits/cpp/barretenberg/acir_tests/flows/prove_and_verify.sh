#!/bin/sh
set -eu

NAME=$(basename $PWD)

if [ -n "$VERBOSE" ]; then
  $BIN prove_and_verify -v -c $CRS_PATH -b ./target/$NAME.bytecode
else
  $BIN prove_and_verify -c $CRS_PATH -b ./target/$NAME.bytecode > /dev/null 2>&1
fi