#!/bin/sh
# TODO(https://github.com/AztecProtocol/barretenberg/issues/813) We don't *actually* download grumpkin yet.
# this just generates grumpkin points and links in a place where run_acir_tests.sh expects it.
# The above issue tracks the final pieces here.
set -eu

# Enter build directory sibling to our script folder.
cd $(dirname $0)/../build
./bin/grumpkin_srs_gen 1048576
mkdir -p ~/.bb-crs
ln -s ../srs_db/grumpkin/monomial ~/.bb-crs/monomial