#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

CMD=${1:-}
BUILD_CMD="build"

if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    git clean -fdx
    exit 0
  elif [ "$CMD" = "esm" ]; then
    BUILD_CMD="build:esm"
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

# Attempt to just pull artefacts from CI and exit on success.
[ -n "${USE_CACHE:-}" ] && ./bootstrap_cache.sh && exit

yarn install --immutable
echo "Building with command 'yarn $BUILD_CMD'..."
yarn $BUILD_CMD

# Make bin globally available.
npm link
echo "Barretenberg ts build successful"
