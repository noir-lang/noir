#!/usr/bin/env bash
set -eu

cd $(dirname $0)/../noir-repo

tmp=$(mktemp)
BACKEND_BARRETENBERG_PACKAGE_JSON=./tooling/noir_js_backend_barretenberg/package.json
jq '.dependencies."@aztec/bb.js" = "portal:../../../../barretenberg/ts"' $BACKEND_BARRETENBERG_PACKAGE_JSON > $tmp && mv $tmp $BACKEND_BARRETENBERG_PACKAGE_JSON

# This script runs in CI which enforces immutable installs by default,
# we then must turn this off in order to update yarn.lock. 
YARN_ENABLE_IMMUTABLE_INSTALLS=false yarn install

# Remove requirement for `wasm-opt` to be installed
sed -i "s/^require_command wasm-opt/#require_command wasm-opt/" ./tooling/noirc_abi_wasm/build.sh
sed -i "s/^require_command wasm-opt/#require_command wasm-opt/" ./acvm-repo/acvm_js/build.sh
