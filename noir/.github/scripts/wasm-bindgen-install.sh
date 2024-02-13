#!/usr/bin/env bash
set -eu

cd $(dirname "$0")

./cargo-binstall-install.sh

# Install wasm-bindgen-cli.
if [ "$(wasm-bindgen --version | cut -d' ' -f2)" != "0.2.86" ]; then
  echo "Building wasm-bindgen..."
  cargo binstall wasm-bindgen-cli@0.2.86 --force --no-confirm
fi

