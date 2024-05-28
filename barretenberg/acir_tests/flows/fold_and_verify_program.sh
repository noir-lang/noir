#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}

$BIN fold_and_verify_program $VFLAG -c $CRS_PATH -b ./target/program.json
