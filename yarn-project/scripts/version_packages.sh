#!/usr/bin/env bash
set -eu

if [ -n "$COMMIT_TAG" ]; then
  for workspace in $(yarn workspaces list --json | jq -r '.location'); do
    (cd $workspace && jq --arg v $COMMIT_TAG '.version = $v' package.json > _temp && mv _temp package.json)
  done
fi