#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}
BFLAG="-b ./target/acir.gz"
FLAGS="-c $CRS_PATH $VFLAG"

# Test we can perform the proof/verify flow.
# This ensures we test independent pk construction through real/garbage witness data paths.
$BIN prove -o proof $FLAGS $BFLAG
$BIN write_vk -o vk $FLAGS $BFLAG
$BIN verify -k vk -p proof $FLAGS
