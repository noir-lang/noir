#!/bin/bash
# This script builds the projects listed in build_mainifest.sh, terminating when it reaches TARGET_PROJECT.
# If run from within a project, it will build only that project, unless ONLY_TARGET=false.

set -e

TARGET_PROJECT=$1
COMMIT_HASH=$(git rev-parse HEAD)

# If we're calling this script from within a project directory, that's the target project.
if [ -z "$TARGET_PROJECT" ]; then
  TARGET_PROJECT=$(git rev-parse --show-prefix)
  if [ -n "$TARGET_PROJECT" ]; then
    # We are in a project folder.
    ONLY_TARGET=${ONLY_TARGET:-true}
    TARGET_PROJECT=$(basename $TARGET_PROJECT)
    cd $(git rev-parse --show-cdup)
  fi
fi

source ./build-system/scripts/setup_env $COMMIT_HASH '' mainframe_$USER $(git rev-parse --show-toplevel)
build_local $TARGET_PROJECT $ONLY_TARGET