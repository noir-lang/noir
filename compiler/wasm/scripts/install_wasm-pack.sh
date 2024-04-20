#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

# Install wasm-pack 
CARGO_BINSTALL_CHECK=$(./scripts/command-check.sh cargo-binstall)
if [ $CARGO_BINSTALL_CHECK != "true" ]; then
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
fi

cargo-binstall wasm-pack@0.12.1 -y --force
