#!/bin/sh
set -eu

VFLAG=${VERBOSE:+-v}
BFLAG="-b ./target/program.json"
FLAGS="-c $CRS_PATH $VFLAG"

$BIN client_ivc_prove_output_all $FLAGS $BFLAG
$BIN verify_client_ivc $FLAGS
