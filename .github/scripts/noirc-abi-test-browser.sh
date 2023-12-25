#!/bin/bash
set -eu

cd /usr/src/noir
npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noirc_abi test:browser
