#!/bin/sh
set -eu

if [ -n "$VERBOSE" ]; then
  $BIN proof_as_fields -v -c $CRS_PATH -p "./proofs/$PROOF_NAME"
else
  $BIN proof_as_fields -c $CRS_PATH -p "./proofs/$PROOF_NAME"
fi