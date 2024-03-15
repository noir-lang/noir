#!/bin/sh
set -eu
# Enter script directory.
cd $(dirname $0)
./download_srs.sh "MAIN%20IGNITION" ignition/monomial $@