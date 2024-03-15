#!/usr/bin/env bash
set -eu

if [ -n "$COMMIT_TAG" ]; then
  for workspace in $(yarn workspaces list --json | jq -r '.location'); do
    (cd $workspace && jq --arg v $COMMIT_TAG '.version = $v' package.json > _temp && mv _temp package.json)
    # allow for versioning already-built packages
    (cd $workspace && [ -f dest/package.json ] && jq --arg v $COMMIT_TAG '.version = $v' dest/package.json > _temp && mv _temp dest/package.json)
  done
fi