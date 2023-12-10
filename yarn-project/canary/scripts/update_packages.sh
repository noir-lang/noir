#!/usr/bin/env bash
set -eu

DIST_TAG=$1
E2E_DIR=$2

if [ -z "$DIST_TAG" ]; then
  echo "No dist tag provided."
  exit 0
fi

echo "Updating Aztec dependencies to tag $DIST_TAG"

TMP=$(mktemp)
# Update NPM dependencies to dist tag
for PKG in $(jq --raw-output ".dependencies | keys[] | select(contains(\"@aztec/\") and (. != \"@aztec/end-to-end\"))" package.json); do
  jq --arg v $DIST_TAG ".dependencies[\"$PKG\"] = \$v" package.json >$TMP && mv $TMP package.json
done

# Update end-to-end to local dependency
TMP=$(mktemp)
jq --arg dir "file:$E2E_DIR" '.dependencies["@aztec/end-to-end"] = $dir' package.json >$TMP && mv $TMP package.json

jq ".references = []" tsconfig.json >$TMP && mv $TMP tsconfig.json
