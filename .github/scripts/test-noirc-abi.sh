#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/install_wasm-bindgen-new.sh
yarn workspace @noir-lang/noirc_abi build
yarn workspace @noir-lang/noirc_abi test
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noirc_abi test:browser
