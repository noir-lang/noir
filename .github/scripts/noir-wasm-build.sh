#!/bin/bash
set -eu

.github/scripts/wasm-bindgen-install.sh
.github/scripts/wasm-pack-install.sh
yarn workspace @noir-lang/noir_wasm build
