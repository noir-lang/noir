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

export cargoExtraArgs="--features noirc_driver/aztec"

cargo build --features="noirc_driver/aztec" --release
export PATH="${PATH}:/usr/src/noir/target/release/"

yarn
yarn build
npx playwright install
npx playwright install-deps

./scripts/test.sh
yarn test