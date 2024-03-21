#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}

$BIN prove_and_verify_goblin_ultra_honk $VFLAG -c $CRS_PATH -b ./target/acir.gz