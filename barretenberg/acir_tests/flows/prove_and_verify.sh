#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}

# This is the fastest flow, because it only generates pk/vk once, gate count once, etc.
# It may not catch all class of bugs.
$BIN prove_and_verify $VFLAG -c $CRS_PATH -b ./target/acir.gz