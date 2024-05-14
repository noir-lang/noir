#!/bin/bash
set -eu

cd $(dirname "$0")

./cargo-binstall-install.sh

cargo-binstall wasm-pack --version 0.12.1 -y
