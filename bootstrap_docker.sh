#!/bin/bash
# This script builds the projects listed in build_mainifest.sh, terminating when it reaches TARGET_PROJECT.
# If run from within a project, it will build only that project, unless env var ONLY_TARGET=false.
#
# To build everything in build_manifest.sh:
#   bootstrap_docker.sh
#
# To build all projects leading up to and including yarn-project-base:
#   bootstrap_docker.sh yarn-project-base
#
# To build just end-to-end:
#   cd yarn-project/end-to-end
#   ../../bootstrap_docker.sh
#
# To build all projects leading up to and including end-to-end, from within end-to-end:
#   cd yarn-project/end-to-end
#   ONLY_TARGET=false ../../bootstrap_docker.sh

set -e

TARGET_PROJECT=$1
COMMIT_HASH=$(git rev-parse HEAD)

# If we're calling this script from within a project directory, that's the target project.
if [ -z "$TARGET_PROJECT" ]; then
  PATH_PREFIX=$(git rev-parse --show-prefix)
  if [ -n "$PATH_PREFIX" ]; then
    # We are in a project folder.
    ONLY_TARGET=${ONLY_TARGET:-true}
    TARGET_PROJECT=$(basename $PATH_PREFIX)
    cd $(git rev-parse --show-cdup)
  fi
fi

source ./build-system/scripts/setup_env $COMMIT_HASH '' mainframe_$USER 
build_local $TARGET_PROJECT $ONLY_TARGET

if [ -z "$TARGET_PROJECT" ]; then
  echo
  echo "Success! You could now run e.g.:"
  echo "  docker run -ti --rm aztecprotocol/end-to-end:latest e2e_private_token_contract.test"
fi
