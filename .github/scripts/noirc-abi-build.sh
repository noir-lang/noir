#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/install_wasm-bindgen-new.sh
yarn workspace @noir-lang/noirc_abi build
