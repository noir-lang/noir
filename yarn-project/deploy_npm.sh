#!/bin/bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

# Check we're on a release flow.
if [ -z "$COMMIT_TAG" ] && [ ! "$DRY_DEPLOY" -eq 1 ]; then
  echo "Not on a release flow, skipping deploy."
  exit 0
fi

extract_repo yarn-project /usr/src project
cd project/src/yarn-project

echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >.npmrc
# also copy npcrc into the l1-contracts directory
cp .npmrc ../l1-contracts

# This is to be used with the 'canary' tag for testing, and then 'latest' for making it public
DIST_TAG=${1:-"latest"}

function deploy_package() {
  REPOSITORY=$1
  cd $REPOSITORY

  PACKAGE_NAME=$(jq -r '.name' package.json)
  VERSION=$(extract_tag_version $REPOSITORY false)
  echo "Deploying $REPOSITORY $VERSION $DIST_TAG"

  if [ -n "$DIST_TAG" ]; then
    TAG_ARG="--tag $DIST_TAG"
  fi

  PUBLISHED_VERSION=$(npm show . version ${TAG_ARG:-} 2>/dev/null) || true
  HIGHER_VERSION=$(npx semver ${VERSION} ${PUBLISHED_VERSION} | tail -1)

  # Check if there is already a published package equal to given version, assume this is a re-run of a deploy
  if [ "$VERSION" == "$PUBLISHED_VERSION" ]; then
    echo "Tagged ${DIST_TAG:+ $DIST_TAG}version $VERSION is equal to published ${DIST_TAG:+ $DIST_TAG}version $PUBLISHED_VERSION."
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
      npm dist-tag add $PACKAGE_NAME@$VERSION $DIST_TAG
    else
      # Publish new version
      npm publish $TAG_ARG --access public
    fi
  fi

  # Back to root
  if [ "$REPOSITORY" == "../l1-contracts" ]; then
    cd ../yarn-project
  else
    cd ..
  fi
}

# New packages here should be added after the last package that they depend on
deploy_package foundation
deploy_package circuits.js
deploy_package types
deploy_package aztec.js
deploy_package l1-artifacts
deploy_package ethereum
deploy_package noir-compiler
deploy_package noir-contracts
deploy_package cli
deploy_package merkle-tree
deploy_package noir-protocol-circuits
deploy_package acir-simulator
deploy_package key-store
deploy_package pxe
deploy_package archiver
deploy_package p2p
deploy_package world-state
deploy_package sequencer-client
deploy_package aztec-node
deploy_package aztec-sandbox
deploy_package ../l1-contracts
