#!/bin/bash
set -eu

# Navigate to script folder
cd "$(dirname "$0")"

if [ "$(uname)" = "Darwin" ]; then
  # works around https://github.com/AztecProtocol/aztec3-packages/issues/158
    echo "Note: not sourcing nvm on Mac, see github #158"
else
  \. ~/.nvm/nvm.sh
fi
set +eu # nvm runs in our context - don't assume it's compatible with these flags
nvm install
set -eu

yarn install --immutable

# Build the necessary dependencies for Aztec.nr contracts typegen.
for package in "@aztec/foundation" "@aztec/noir-compiler"; do
  echo "Building $package"
  yarn workspace $package build
done

# Run remake constants before building Aztec.nr contracts or l1 contracts as they depend on files created by it.
yarn workspace @aztec/circuits.js remake-constants

(cd noir-contracts && ./bootstrap.sh)
(cd .. && l1-contracts/bootstrap.sh)

# We do not need to build individual packages, yarn build will build the root tsconfig.json
yarn build

echo
echo "Success! You can now e.g. run anvil and end-to-end tests"
