#!/usr/bin/env bash
set -eu

ROOT=$(realpath $(dirname "$0")/..)
cd $ROOT/noir-repo

./.github/scripts/wasm-bindgen-install.sh

# Set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=${COMMIT_HASH:-$(git rev-parse --verify HEAD)}

PROJECTS=(
  @noir-lang/acvm_js
  @noir-lang/types
  @noir-lang/noirc_abi
)
INCLUDE=$(printf " --include %s" "${PROJECTS[@]}")

yarn --immutable

yarn workspaces foreach --parallel --topological-dev --verbose $INCLUDE run build

# We create a folder called packages, that contains each package as it would be published to npm, named correctly.
# These can be useful for testing, or portaling into other projects.
yarn workspaces foreach --parallel $INCLUDE pack

cd $ROOT
rm -rf packages && mkdir -p packages
for PROJECT in "${PROJECTS[@]}"; do
  PPATH=$(cd noir-repo && yarn workspaces list --json | jq -r "select(.name==\"$PROJECT\").location")
  tar zxfv noir-repo/$PPATH/package.tgz -C packages && mv packages/package packages/${PROJECT#*/}
done

if [ -x $ROOT/scripts/fix_incremental_ts.sh ]; then
  $ROOT/scripts/fix_incremental_ts.sh
fi
