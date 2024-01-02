#!/bin/bash
set -eu

cd /usr/src/noir
apt-get install libc++-dev -y
npx playwright install && npx playwright install-deps
yarn workspace integration-tests test