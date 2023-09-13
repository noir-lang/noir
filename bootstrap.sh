#!/bin/bash
set -eu

export CMD=${1:-}

cd "$(dirname "$0")"

# Lightweight bootstrap. Run `./bootstrap.sh clean` to bypass.
if [ -f .bootstrapped ]; then
  echo -e '\033[1mRebuild L1 contracts...\033[0m'
  (cd l1-contracts && forge build)

  echo -e '\n\033[1mUpdate npm deps...\033[0m'
  (cd yarn-project && yarn install)

  echo -e '\n\033[1mRebuild Noir contracts...\033[0m'
  (cd yarn-project/noir-contracts && yarn noir:build:all 2> /dev/null)

  echo -e '\n\033[1mRebuild barretenberg wasm...\033[0m'
  (cd barretenberg/cpp && cmake --build --preset wasm && cmake --build --preset wasm-threads)

  echo -e '\n\033[1mRebuild circuits wasm...\033[0m'
  (cd circuits/cpp && cmake --build --preset wasm -j --target aztec3-circuits.wasm)

  exit 0
fi

# Remove all untracked files and directories.
if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    echo "WARNING: This will erase *all* untracked files, including hooks and submodules."
    echo -n "Continue? [y/n] "
    read user_input
    if [ "$user_input" != "y" ] && [ "$user_input" != "Y" ]; then
      exit 1
    fi
    rm .bootstrapped
    rm -rf .git/hooks/*
    rm -rf .git/modules/*
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

# Install pre-commit git hooks.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "(cd barretenberg/cpp && ./format.sh staged)" > $HOOKS_DIR/pre-commit
echo "(cd circuits/cpp && ./format.sh staged)" >> $HOOKS_DIR/pre-commit
chmod +x $HOOKS_DIR/pre-commit

barretenberg/cpp/bootstrap.sh
circuits/cpp/bootstrap.sh
yarn-project/bootstrap.sh

touch .bootstrapped