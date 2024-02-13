#!/bin/bash
set -eu

./.github/scripts/playwright-install.sh
yarn workspace integration-tests test:browser