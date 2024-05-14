#!/bin/bash
set -eu

./.github/scripts/playwright-install.sh
yarn workspace @noir-lang/noirc_abi test:browser
