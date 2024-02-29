#!/bin/bash
set -eu

cd $(dirname "$0")/../noir-repo

./.github/scripts/wasm-bindgen-install.sh

# Set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=${COMMIT_HASH:-$(git rev-parse --verify HEAD)}

cargo build --release
export PATH="${PATH}:/usr/src/noir/noir-repo/target/release/"

yarn --immutable
yarn build
./.github/scripts/playwright-install.sh

yarn test
