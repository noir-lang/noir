#!/bin/bash
# This script builds the projects listed in build_mainifest.sh, terminating when it reaches PROJECT_NAME.
# If run from within a project, it will build only that project, unless env var ONLY_TARGET=false.
#
# Usage:
#   boostrap_docker.sh [PROJECT_NAME]
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

PROJECT_NAME=${1:-}

if [ -z "$PROJECT_NAME" ]; then
  echo "usage: $0 <project_name>"
  exit 1
fi

cd "$(dirname "$0")"

source ./build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null
build_local $PROJECT_NAME

if [ -z "$PROJECT_NAME" ]; then
  echo
  echo "Success! You could now run e.g.:"
  echo "  docker run -ti --rm aztecprotocol/end-to-end:latest e2e_private_token_contract.test"
fi
