#!/bin/bash
set -eu

cd $(dirname "$0")/..

./scripts/install_wasm-bindgen.sh

# If this project has been subrepod into another project, set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
if [ -f ".gitrepo" ]; then
  export GIT_COMMIT=$(awk '/commit =/ {print $3}' .gitrepo)
else
  export GIT_COMMIT=$(git rev-parse --verify HEAD)
fi

cargo build --release
export PATH="${PATH}:/usr/src/noir/target/release/"

yarn --immutable
yarn build
npx playwright install
npx playwright install-deps

./scripts/test.sh
yarn test
