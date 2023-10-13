#!/bin/bash
set -eu

DIST_TAG=$1

if [ -z "$DIST_TAG" ]; then
  echo "No dist tag provided."
  exit 0
fi

echo "Updating Aztec dependencies to tag $DIST_TAG"

TMP=$(mktemp)
for PKG in $(jq --raw-output ".dependencies | keys[] | select(contains(\"@aztec/\") and (. != \"@aztec/end-to-end\"))" package.json); do
  jq --arg v $DIST_TAG ".dependencies[\"$PKG\"] = \$v" package.json >$TMP && mv $TMP package.json
done

jq ".references = []" tsconfig.json >$TMP && mv $TMP tsconfig.json
