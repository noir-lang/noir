#!/bin/bash
set -e

export CLEAN=$1

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
    exit 0
  else
    echo "Unknown command: $CLEAN"
    exit 1
  fi
fi

if [ ! -f ~/.nvm/nvm.sh ]; then
  echo "Nvm not found at ~/.nvm"
  exit 1
fi

\. ~/.nvm/nvm.sh
nvm install

git submodule update --init --recursive

yarn install --immutable
yarn build

echo
echo "Success!"
