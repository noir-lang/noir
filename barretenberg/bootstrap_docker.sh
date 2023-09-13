#!/bin/bash
# This script builds the projects listed in build_mainifest.sh.

set -eu

COMMIT_HASH=$(git rev-parse HEAD)
source ./build-system/scripts/setup_env $COMMIT_HASH '' mainframe_$USER $(git rev-parse --show-toplevel)
build_local
