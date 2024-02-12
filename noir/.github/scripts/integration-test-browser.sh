#!/bin/bash
set -eu

npx playwright install && npx playwright install-deps
yarn workspace integration-tests test
