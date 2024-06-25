#!/usr/bin/env bash
# Env var overrides:
#   BIN: to specify a different binary to test with (e.g. bb.js or bb.js-dev).
set -eu

BIN=${BIN:-../cpp/build/bin/bb}
CRS_PATH=~/.bb-crs
BRANCH=master
VERBOSE=${VERBOSE:-}
RECURSIVE=true
PROOF_NAME="proof_a"

if [ -f $BIN ]; then
    BIN=$(realpath $BIN)
else
    BIN=$(realpath $(which $BIN))
fi

export BRANCH

./clone_test_vectors.sh

cd acir_tests/assert_statement_recursive

PROOF_DIR=$PWD/proofs
PROOF_PATH=$PROOF_DIR/$PROOF_NAME
VFLAG=${VERBOSE:+-v}
RFLAG=${RECURSIVE:+-r}

echo "Write VK to file for assert_statement..."
$BIN write_vk_ultra_honk $VFLAG -c $CRS_PATH -o ./target/honk_vk

echo "Write VK as fields for recursion..."
$BIN vk_as_fields_ultra_honk $VFLAG -c $CRS_PATH -k ./target/honk_vk -o ./target/honk_vk_fields.json

echo "Generate proof to file..."
[ -d "$PROOF_DIR" ] || mkdir $PWD/proofs
[ -e "$PROOF_PATH" ] || touch $PROOF_PATH
$BIN prove_ultra_honk $VFLAG -c $CRS_PATH -b ./target/program.json -o "./proofs/honk_$PROOF_NAME"

echo "Write proof as fields for recursion..."
$BIN proof_as_fields_honk $VFLAG -c $CRS_PATH -p "./proofs/honk_$PROOF_NAME" -o "./proofs/honk_${PROOF_NAME}_fields.json"