#!/bin/bash
set -eu

./.github/scripts/playwright-install.sh
yarn workspace @noir-lang/acvm_js test:browser
