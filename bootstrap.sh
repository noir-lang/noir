#!/usr/bin/env bash
set -eu

cd $(dirname "$0")

CMD=${1:-}

if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    git clean -fdx
    exit 0
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

# Attempt to just pull artefacts from CI and exit on success.
./bootstrap_cache.sh && exit

./scripts/bootstrap_native.sh
./scripts/bootstrap_packages.sh