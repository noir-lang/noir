#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

./.github/scripts/wasm-bindgen-install.sh

# If this project has been subrepod into another project, set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
if [ -f ".gitrepo" ]; then
  export GIT_COMMIT=$(awk '/commit =/ {print $3}' .gitrepo)
else
  export GIT_COMMIT=$(git rev-parse --verify HEAD)
fi

yarn --immutable
yarn build

# We create a folder called packages, that contains each package as it would be published to npm, named correctly.
# These can be useful for testing, or portaling into other projects.
yarn workspaces foreach pack

rm -rf packages && mkdir -p packages
tar zxfv acvm-repo/acvm_js/package.tgz -C packages && mv packages/package packages/acvm_js
tar zxfv compiler/wasm/package.tgz -C packages && mv packages/package packages/noir_wasm
tar zxfv tooling/noir_codegen/package.tgz -C packages && mv packages/package packages/noir_codegen
tar zxfv tooling/noir_js/package.tgz -C packages && mv packages/package packages/noir_js
tar zxfv tooling/noir_js_backend_barretenberg/package.tgz -C packages && mv packages/package packages/backend_barretenberg
tar zxfv tooling/noir_js_types/package.tgz -C packages && mv packages/package packages/types
tar zxfv tooling/noirc_abi_wasm/package.tgz -C packages && mv packages/package packages/noirc_abi
