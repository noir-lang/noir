#!/bin/sh
set -eu

mkdir -p ./proofs

VFLAG=${VERBOSE:+-v}
BFLAG="-b ./target/program.json"
FLAGS="-c $CRS_PATH $VFLAG"

# Test we can perform the proof/verify flow.
# This ensures we test independent pk construction through real/garbage witness data paths.
$BIN prove_mega_honk -o proof $FLAGS $BFLAG
$BIN write_vk_mega_honk -o vk $FLAGS $BFLAG
$BIN verify_mega_honk -k vk -p proof $FLAGS
