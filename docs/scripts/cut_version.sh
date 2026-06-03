#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

VERSION=$1

# Pin the noirjs_app tutorial to this release's tested (noir_js, bb.js) pair before
# the snapshot is frozen. noir_js ships with the Noir release; bb.js is the version
# this release builds against (scripts/install_bb.sh, also used by bbup and the
# browser example test). At cut time the working tree already is the release, so the
# versions can be read straight from it without consulting git tags.
NOIR_JS_VERSION=$(jq -r '."."' ../.release-please-manifest.json)
BB_JS_VERSION=$(sed -nE 's/^VERSION="([^"]+)".*/\1/p' ../scripts/install_bb.sh)
TUTORIAL="docs/tutorials/noirjs_app.md"

if [ -z "$NOIR_JS_VERSION" ] || [ "$NOIR_JS_VERSION" = "null" ]; then
  echo "cut_version: could not read noir version from .release-please-manifest.json" >&2
  exit 1
fi
if [ -z "$BB_JS_VERSION" ]; then
  echo "cut_version: could not parse bb.js version from scripts/install_bb.sh" >&2
  exit 1
fi
if [ "${VERSION#v}" != "$NOIR_JS_VERSION" ]; then
  echo "cut_version: requested $VERSION but manifest noir version is $NOIR_JS_VERSION" >&2
  exit 1
fi

sed -i -E \
  -e "s|(@noir-lang/noir_js@)[^ ]+|\1${NOIR_JS_VERSION}|" \
  -e "s|(@aztec/bb\.js@)[^ ]+|\1${BB_JS_VERSION}|" \
  -e "s|(noirup -v )[0-9A-Za-z.-]+|\1${NOIR_JS_VERSION}|" \
  "$TUTORIAL"

# Fail loudly if an anchor moved, rather than freezing a half-synced snapshot.
grep -q "@noir-lang/noir_js@${NOIR_JS_VERSION}" "$TUTORIAL" || { echo "cut_version: failed to pin noir_js in $TUTORIAL" >&2; exit 1; }
grep -q "@aztec/bb.js@${BB_JS_VERSION}" "$TUTORIAL" || { echo "cut_version: failed to pin bb.js in $TUTORIAL" >&2; exit 1; }
grep -q "noirup -v ${NOIR_JS_VERSION}" "$TUTORIAL" || { echo "cut_version: failed to pin noirup version in $TUTORIAL" >&2; exit 1; }

# We assume that the new release tag has been made on github, so setStable.ts will add this to `versions.json`.
# We don't have a version of the docs for this release however (that's what we're doing right now!) so we need to remove it.
jq 'map(select(. != "'"$VERSION"'"))' versions.json > tmp.json && mv tmp.json versions.json

# We need to build the docs in order to perform all necessary preprocessing.
yarn build

# Finally cut the actual new docs version.
yarn docusaurus docs:version $VERSION
