#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}
BFLAG="-b ./target/acir.gz"
FLAGS="-c $CRS_PATH $VFLAG"

# Test we can perform the proof/verify flow.
$BIN gates $FLAGS $BFLAG > /dev/null
$BIN prove -o proof $FLAGS $BFLAG
$BIN write_vk -o vk $FLAGS $BFLAG
$BIN write_pk -o pk $FLAGS $BFLAG
$BIN verify -k vk -p proof $FLAGS

# Check supplemental functions.
# Grep to determine success.
$BIN contract -k vk $BFLAG -o - | grep "Verification Key Hash" > /dev/null
# Use jq to determine success, and also check result not empty.
OUTPUT=$($BIN proof_as_fields -k vk -p proof -o - | jq .)
[ -n "$OUTPUT" ] || exit 1
OUTPUT=$($BIN vk_as_fields -k vk -o - | jq .)
[ -n "$OUTPUT" ] || exit 1