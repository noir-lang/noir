#!/bin/bash
set -eu

cd /usr/src/noir
.github/scripts/install_wasm-bindgen.sh
yarn workspace @noir-lang/noirc_abi build
