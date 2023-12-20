#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/install_wasm-bindgen-new.sh
yarn workspace @noir-lang/acvm_js build
yarn workspace @noir-lang/acvm_js test
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/acvm_js test:browser
