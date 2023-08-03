#!/bin/bash
set -eu

export CLEAN=${1:-}

cd "$(dirname "$0")"

# Remove all untracked files and directories.
if [ -n "$CLEAN" ]; then
  if [ "$CLEAN" = "clean" ]; then
    echo "WARNING: This will erase *all* untracked files, including hooks and submodules."
    echo -n "Continue? [y/n] "
    read user_input
    if [ "$user_input" != "y" ] && [ "$user_input" != "Y" ]; then
      exit 1
    fi
    rm -rf .git/hooks/*
    git clean -fd
    for SUBMODULE in $(git config --file .gitmodules --get-regexp path | awk '{print $2}'); do
      rm -rf $SUBMODULE
    done
    git submodule update --init --recursive
    exit 0
  else
    echo "Unknown command: $CLEAN"
    exit 1
  fi
fi

git submodule update --init --recursive

if [ ! -f ~/.nvm/nvm.sh ]; then
  echo "Nvm not found at ~/.nvm"
  exit 1
fi

circuits/cpp/bootstrap.sh

if [ "$(uname)" = "Darwin" ]; then
  # works around https://github.com/AztecProtocol/aztec3-packages/issues/158
  echo "Note: not sourcing nvm on Mac, see github #158"
else
  \. ~/.nvm/nvm.sh
fi
nvm install

cd yarn-project
yarn install --immutable

# Build the necessary dependencies for noir contracts typegen.
for DIR in foundation noir-compiler circuits.js; do
  echo "Building $DIR..."
  cd $DIR
  yarn build
  cd ..
done

# Run remake bindings before building noir contracts or l1 contracts as they depend on files created by it.
yarn --cwd circuits.js remake-bindings
yarn --cwd circuits.js remake-constants

(cd noir-contracts && ./bootstrap.sh)
(cd .. && l1-contracts/bootstrap.sh)

# Until we push .yarn/cache, we still need to install.
yarn
# We do not need to build individual packages, yarn build will build the root tsconfig.json
yarn build
cd ..

echo
echo "Success! You could now run e.g.: ./scripts/tmux-splits e2e_deploy_contract"
