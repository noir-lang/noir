#!/bin/sh
set -eu

if [ -n "${VERBOSE:-}" ]; then
  VFLAG="-v"
else
  VFLAG=""
fi

BFLAG="-b ./target/acir.gz"
FLAGS="-c $CRS_PATH $VFLAG"

# Test we can perform the proof/verify flow.
$BIN gates $FLAGS $BFLAG > /dev/null
$BIN prove -o proof $FLAGS $BFLAG
$BIN write_vk -o vk $FLAGS $BFLAG
$BIN verify -k vk -p proof $FLAGS

# Check supplemental functions.
# Grep to determine success.
$BIN contract -k vk $BFLAG -o - | grep "Verification Key Hash" > /dev/null
# Use jq to determine success.
$BIN proof_as_fields -k vk -p proof -o - | jq . > /dev/null
$BIN vk_as_fields -k vk -o - > vk_as_fields | jq . > /dev/null