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

# Run remake bindings before building Aztec.nr contracts or l1 contracts as they depend on files created by it.
yarn --cwd circuits.js remake-bindings
yarn --cwd circuits.js remake-constants

# Build the necessary dependencies for Aztec.nr contracts typegen.
for DIR in foundation noir-compiler circuits.js; do
  echo "Building $DIR..."
  cd $DIR
  yarn build
  cd ..
done

(cd noir-contracts && ./bootstrap.sh)
(cd .. && l1-contracts/bootstrap.sh)

# Until we push .yarn/cache, we still need to install.
yarn
# We do not need to build individual packages, yarn build will build the root tsconfig.json
yarn build

echo
echo "Success! You can now e.g. run anvil and end-to-end tests"
