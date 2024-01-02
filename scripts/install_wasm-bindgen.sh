#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

# Install wasm-bindgen-cli.
if [ "$(wasm-bindgen --version | cut -d' ' -f2)" != "0.2.86" ]; then
  echo "Building wasm-bindgen..."
  RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install -f wasm-bindgen-cli --version 0.2.86
fi
