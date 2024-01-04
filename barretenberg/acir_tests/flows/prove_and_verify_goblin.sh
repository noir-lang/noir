#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}

$BIN prove_and_verify_goblin $VFLAG -c $CRS_PATH -b ./target/acir.gz

# This command can be used to run all of the tests in sequence with the debugger
# lldb-16 -o run -b -- $BIN prove_and_verify_goblin $VFLAG -c $CRS_PATH -b ./target/acir.gz