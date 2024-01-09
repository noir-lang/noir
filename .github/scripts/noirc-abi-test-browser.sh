#!/bin/bash
set -eu

npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/noirc_abi test:browser
