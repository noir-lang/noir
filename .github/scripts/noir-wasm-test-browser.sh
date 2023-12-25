#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/nargo_compile_wasm_fixtures.sh
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noir_wasm test:browser