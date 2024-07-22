#!/usr/bin/env bash

PLONK_FLAVOUR=${1:-"ultra"}
CIRCUIT_FLAVOUR=${2:-"blake"}
INPUTS=${3:-"1,2,3,4"}

BIN="../cpp/build/bin/solidity_proof_gen"

INPUTS="$( sed 's/\\n//g' <<<"$INPUTS" )"

SRS_PATH="../cpp/srs_db/ignition"

# If the plonk flavour is honk, then run the honk generator
if [ "$PLONK_FLAVOUR" == "honk" ]; then
    BIN="../cpp/build/bin/honk_solidity_proof_gen"
fi

# @note This needs to be updated to point to the generator
$BIN $PLONK_FLAVOUR $CIRCUIT_FLAVOUR $SRS_PATH $INPUTS