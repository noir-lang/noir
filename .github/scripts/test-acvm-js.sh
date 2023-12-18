#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/install_wasm-bindgen-new.sh
apt-get install -y jq
yarn workspace @noir-lang/acvm_js build
yarn workspace @noir-lang/acvm_js test