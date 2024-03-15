#!/bin/bash
set -eu

.github/scripts/wasm-pack-install.sh
yarn workspace @noir-lang/types build
yarn workspace @noir-lang/noir_wasm build
