#!/bin/bash
set -eu

npx playwright install && npx playwright install-deps
yarn workspace @noir-lang/acvm_js test:browser
