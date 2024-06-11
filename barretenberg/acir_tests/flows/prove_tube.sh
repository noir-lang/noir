#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}
BFLAG="-b ./target/program.json"
FLAGS="-c $CRS_PATH $VFLAG"

$BIN client_ivc_prove_output_all $VFLAG -c $CRS_PATH -b ./target/program.json
$BIN prove_tube -k vk -p proof $FLAGS
