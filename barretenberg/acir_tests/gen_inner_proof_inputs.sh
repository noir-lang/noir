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
$BIN write_vk $VFLAG -c $CRS_PATH -o

echo "Write VK as fields for recursion..."
$BIN vk_as_fields $VFLAG -c $CRS_PATH

echo "Generate proof to file..."
[ -d "$PROOF_DIR" ] || mkdir $PWD/proofs
[ -e "$PROOF_PATH" ] || touch $PROOF_PATH
$BIN prove $VFLAG -c $CRS_PATH -b ./target/acir.gz -o "./proofs/$PROOF_NAME" $RFLAG

echo "Write proof as fields for recursion..."
$BIN proof_as_fields $VFLAG -c $CRS_PATH -p "./proofs/$PROOF_NAME"

cat ./proofs/${PROOF_NAME}_fields.json
echo
