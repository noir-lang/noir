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

# Docusaurus only builds the versions listed in `versions.json`, which `scripts/setStable.ts`
# caps at NUMBER_OF_VERSIONS_TO_SHOW. Snapshots older than that are never built, routed or
# linked, so drop them here instead of carrying one more directory per release forever.
# Keep more than are served so that raising NUMBER_OF_VERSIONS_TO_SHOW does not immediately
# require re-cutting docs that were already deleted.
VERSIONS_TO_KEEP=8

ls -1 versioned_docs | sort -rV | tail -n +$((VERSIONS_TO_KEEP + 1)) | while read -r dir; do
  echo "Pruning docs snapshot that is no longer served: $dir"
  rm -rf "versioned_docs/$dir" "versioned_sidebars/${dir}-sidebars.json"
done
