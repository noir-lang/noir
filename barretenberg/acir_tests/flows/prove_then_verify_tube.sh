#!/bin/sh
set -eux

mkdir -p ./proofs

VFLAG=${VERBOSE:+-v}

$BIN client_ivc_prove_output_all $VFLAG -c $CRS_PATH -b ./target/program.json
$BIN prove_tube -k vk -p proof -c $CRS_PATH $VFLAG
$BIN verify_tube -k vk -p proof -c $CRS_PATH $VFLAG

