#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

# Install binstall
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install wasm-bindgen-cli.
if [ "$(wasm-bindgen --version | cut -d' ' -f2)" != "0.2.86" ]; then
  echo "Building wasm-bindgen..."
  cargo binstall wasm-bindgen-cli@0.2.86 --force --no-confirm
fi

