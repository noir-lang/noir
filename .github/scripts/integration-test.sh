#!/bin/bash
set -eu

apt-get install libc++-dev -y
npx playwright install && npx playwright install-deps
yarn workspace integration-tests test