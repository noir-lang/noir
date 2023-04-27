#!/bin/bash

PLONK_FLAVOUR=${1:-"ultra"}
CIRCUIT_FLAVOUR=${2:-"blake"}
INPUTS=${3:-"1,2,3,4"}

INPUTS="$( sed 's/\\n//g' <<<"$INPUTS" )"

SRS_PATH="../cpp/srs_db/ignition"

# @note This needs to be updated to point to the generator
../cpp/build/bin/solidity_proof_gen $PLONK_FLAVOUR $CIRCUIT_FLAVOUR $SRS_PATH $INPUTS