#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
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

echo -e "\033[1mGenerating constants files...\033[0m"
# Required to run remake-constants.
yarn workspace @aztec/foundation build
# Run remake constants before building Aztec.nr contracts or l1 contracts as they depend on files created by it.
yarn workspace @aztec/circuits.js remake-constants

echo -e "\033[1mSetting up compiler and building contracts...\033[0m"
# This is actually our code generation tool. Needed to build contract typescript wrappers.
echo "Building noir compiler..."
yarn workspace @aztec/noir-compiler build
# Builds noir contracts (TODO: move this stage pre yarn-project). Generates typescript wrappers.
echo "Building contracts from noir-contracts..."
yarn workspace @aztec/noir-contracts build:contracts
# Bundle compiled contracts into other packages
echo "Copying account contracts..."
yarn workspace @aztec/accounts build:copy-contracts
echo "Copying protocol contracts..."
yarn workspace @aztec/protocol-contracts build:copy-contracts
# Build protocol circuits. TODO: move pre yarn-project.
echo "Building circuits from noir-protocol-circuits..."
yarn workspace @aztec/noir-protocol-circuits build

echo -e "\033[1mBuilding all packages...\033[0m"
yarn build

echo
echo "Yarn project successfully built."
