#!/bin/bash
set -eu

cd $(dirname "$0")

./cargo-binstall-install.sh

cargo-binstall wasm-opt --version 0.116.0 -y --force
