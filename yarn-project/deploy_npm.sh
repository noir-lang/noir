#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

if [ -z "$COMMIT_TAG" ]; then
  echo "No commit tag, not deploying to npm."
  exit 0
fi

retry ecr_login
extract_repo yarn-project /usr/src project
cd project/src/yarn-project

echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >.npmrc

# This is to be used with the 'canary' tag for testing, and then 'latest' for making it public
DIST_TAG=${1:-"latest"}

function deploy_package() {
  REPOSITORY=$1
  cd $REPOSITORY

  PACKAGE_NAME=$(jq -r '.name' package.json)
  VERSION=$(extract_tag_version $REPOSITORY false)
  echo "Deploying $REPOSITORY $VERSION $DIST_TAG"

  # If the commit tag itself has a dist-tag (e.g. v2.1.0-testnet.123), extract the dist-tag.
  TAG=$(echo "$VERSION" | grep -oP ".*-\K(.*)(?=\.\d+)" || true)
  TAG_ARG=""
  if [ -n "$TAG" ]; then
    TAG_ARG="--tag $TAG"
  else
    TAG_ARG="--tag $DIST_TAG"
    TAG=$DIST_TAG
  fi

  PUBLISHED_VERSION=$(npm show . version ${TAG_ARG:-} 2>/dev/null) || true
  HIGHER_VERSION=$(npx semver ${VERSION} ${PUBLISHED_VERSION} | tail -1)

  # Check if there is already a published package equal to given version, assume this is a re-run of a deploy
  if [ "$VERSION" == "$PUBLISHED_VERSION" ]; then
    echo "Tagged $TAG version $VERSION is equal to published $TAG version $PUBLISHED_VERSION."
    echo "Skipping publish."
    exit 0
  fi

  # If the published version is > the given version, something's gone wrong.
  if [ "$VERSION" != "$HIGHER_VERSION" ]; then
    echo "Tagged version $VERSION is lower than published version $PUBLISHED_VERSION."
    exit 1
  fi

  # Update the package version in package.json.
  TMP=$(mktemp)
  jq --arg v $VERSION '.version = $v' package.json >$TMP && mv $TMP package.json

  if [ -z "${STANDALONE:-}" ]; then
    # Update each dependent @aztec package version in package.json.
    for PKG in $(jq --raw-output ".dependencies | keys[] | select(contains(\"@aztec/\"))" package.json); do
      jq --arg v $VERSION ".dependencies[\"$PKG\"] = \$v" package.json >$TMP && mv $TMP package.json
    done
  fi

  # Publish
  if [ "$DRY_DEPLOY" -eq 1 ]; then
    npm publish --dry-run $TAG_ARG --access public
  else
    # Check if version exists
    if npm view "$PACKAGE_NAME@$VERSION" version >/dev/null 2>&1; then
      # Tag the existing version
      npm dist-tag add $PACKAGE_NAME@$VERSION $TAG
    else
      # Publish new version
      npm publish $TAG_ARG --access public
    fi
  fi

  # Return to root
  cd ..
}

# New packages here should be added after the last package that they depend on
deploy_package foundation
deploy_package types
deploy_package circuits.js
deploy_package circuit-types
deploy_package protocol-contracts
deploy_package aztec.js
deploy_package entrypoints
deploy_package accounts
deploy_package l1-artifacts
deploy_package ethereum
deploy_package builder
deploy_package noir-contracts.js
deploy_package kv-store
deploy_package merkle-tree
deploy_package noir-protocol-circuits-types
deploy_package world-state
deploy_package simulator
deploy_package bb-prover
deploy_package key-store
deploy_package pxe
deploy_package archiver
deploy_package p2p
deploy_package prover-client
deploy_package sequencer-client
deploy_package bot
deploy_package prover-node
deploy_package aztec-node
deploy_package txe
