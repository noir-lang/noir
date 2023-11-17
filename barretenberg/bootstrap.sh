#!/bin/bash
set -eu

# Navigate to script folder
cd "$(dirname "$0")"

(cd cpp && ./bootstrap.sh)
(cd ts && yarn install --immutable && yarn build && npm link)
