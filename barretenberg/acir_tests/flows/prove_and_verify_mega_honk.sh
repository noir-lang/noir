#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}

$BIN prove_and_verify_mega_honk $VFLAG -c $CRS_PATH -b ./target/program.json
