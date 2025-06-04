#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

../../.github/scripts/cargo-binstall-install.sh

# Install wasm-pack 
cargo-binstall wasm-pack@0.13.1 -y --force
