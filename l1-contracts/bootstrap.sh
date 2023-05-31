#!/bin/bash
set -eu

cd "$(dirname "$0")"

# Clean
rm -rf broadcast cache out serve

# Install foundry.
. ./scripts/install_foundry.sh

# Install
forge install --no-commit

# Ensure libraries are at the correct version
git submodule update --init --recursive ./lib

# Compile contracts
forge build