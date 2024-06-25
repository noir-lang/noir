#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

VERSION=$1

# We assume that the new release tag has been made on github, so setStable.ts will add this to `versions.json`.
# We don't have a version of the docs for this release however (that's what we're doing right now!) so we need to remove it.
jq 'map(select(. != "'"$VERSION"'"))' versions.json > tmp.json && mv tmp.json versions.json

# We need to build the docs in order to perform all necessary preprocessing.
yarn build

# Finally cut the actual new docs version.
yarn docusaurus docs:version $VERSION
