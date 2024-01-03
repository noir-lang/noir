#!/usr/bin/env bash
set -eu

# Check node version.
node_version=$(node -v | tr -d 'v')
major=${node_version%%.*}
rest=${node_version#*.}
minor=${rest%%.*}

if (( major < 18 || ( major == 18 && minor < 19 ) )); then
    echo "Node.js version is less than 18.19. Exiting."
    exit 1
fi

cd "$(dirname "$0")"

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

yarn install --immutable

# Run remake constants before building Aztec.nr contracts or l1 contracts as they depend on files created by it.
yarn workspace @aztec/circuits.js remake-constants
# This is actually our code generation tool. Needed to build contract typescript wrappers.
yarn workspace @aztec/noir-compiler build
# Builds noir contracts (TODO: move this stage pre yarn-project). Generates typescript wrappers.
yarn workspace @aztec/noir-contracts build:contracts
# TODO: Contracts should not be baked into aztec.js.
yarn workspace @aztec/aztec.js build:copy-contracts
# Build protocol circuits. TODO: move pre yarn-project.
yarn workspace @aztec/noir-protocol-circuits noir:build

yarn build

echo
echo "Success! You can now e.g. run anvil and end-to-end tests"
