#!/bin/bash
set -eu

.github/scripts/wasm-bindgen-install.sh
yarn workspace @noir-lang/noirc_abi build
