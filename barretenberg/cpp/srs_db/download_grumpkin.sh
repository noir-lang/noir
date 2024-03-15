#!/bin/sh
# TODO(https://github.com/AztecProtocol/barretenberg/issues/898): Grumpkin needs to match new layout.
set -eu
# Enter script directory.
cd $(dirname $0)
./download_srs.sh "TEST%20GRUMPKIN" grumpkin/monomial 1 $@
mkdir -p  ~/.bb-crs
ln -s ../srs_db/grumpkin/monomial ~/.bb-crs/monomial