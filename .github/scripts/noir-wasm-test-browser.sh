#!/bin/bash
set -eu

./scripts/nargo_compile_wasm_fixtures.sh
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noir_wasm test:browser