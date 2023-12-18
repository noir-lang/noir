#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/install_wasm-bindgen-new.sh
yarn workspace @noir-lang/noir_wasm build
./scripts/nargo_compile_wasm_fixtures.sh
yarn workspace @noir-lang/noir_wasm test:node
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noir_wasm test:browser
