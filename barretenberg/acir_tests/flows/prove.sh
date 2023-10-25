#!/bin/sh
set -eu

echo -n "INSIDE PROVE SCRIPT"
echo -n "$RECURSIVE"

if [ -n "$VERBOSE" ]; then
  if [ -n "$RECURSIVE" ]; then
    $BIN prove -v -c $CRS_PATH -b ./target/acir.gz -o "./proofs/$PROOF_NAME" -r
  else 
    $BIN prove -v -c $CRS_PATH -b ./target/acir.gz -o "./proofs/$PROOF_NAME"
  fi
else
  if [ -n "$RECURSIVE" ]; then
    echo -n "HERE"

    $BIN prove -v -c $CRS_PATH -b ./target/acir.gz -o "./proofs/$PROOF_NAME" -r
  else 
    $BIN prove -c $CRS_PATH -b ./target/acir.gz -o "./proofs/$PROOF_NAME"
  fi
fi