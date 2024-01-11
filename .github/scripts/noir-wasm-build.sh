#!/bin/bash
set -eu

.github/scripts/install_wasm-bindgen.sh
yarn workspace @noir-lang/noir_wasm build