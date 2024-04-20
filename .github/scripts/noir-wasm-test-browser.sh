#!/bin/bash
set -eu

./.github/scripts/playwright-install.sh
yarn workspace @noir-lang/noir_wasm test:build_fixtures
yarn workspace @noir-lang/noir_wasm test:browser
