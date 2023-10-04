#!/bin/bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

extract_repo yarn-project /usr/src project
cd project/src/yarn-project

echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" > .npmrc

function deploy_package() {
    REPOSITORY=$1
    VERSION=$(extract_tag_version $REPOSITORY false)

    # If the commit tag itself has a dist-tag (e.g. v2.1.0-testnet.123), extract the dist-tag.
    TAG=$(echo "$VERSION" | grep -oP ".*-\K(.*)(?=\.\d+)" || true)
    TAG_ARG=""
    if [ -n "$TAG" ]; then
        TAG_ARG="--tag $TAG"
    fi

    readonly PUBLISHED_VERSION=$(npm show . version ${TAG_ARG:-} 2> /dev/null)
    readonly HIGHER_VERSION=$(npx semver ${VERSION} ${PUBLISHED_VERSION} | tail -1)

    # If there is already a published package equal to given version, assume this is a re-run of a deploy, and early out.
    if [ "$VERSION" == "$PUBLISHED_VERSION" ]; then
        echo "Tagged version $VERSION is equal to published version $PUBLISHED_VERSION. Skipping publish."
        exit 0
    fi

    # If the published version is > the given version, something's gone wrong.
    if [ "$VERSION" != "$HIGHER_VERSION" ]; then
        echo "Tagged version $VERSION is lower than published version $PUBLISHED_VERSION."
        exit 1
    fi

    # Update the package version in package.json.
    TMP=$(mktemp)
    jq --arg v $VERSION '.version = $v' package.json > $TMP && mv $TMP package.json

    if [ -z "${STANDALONE:-}" ]; then
    # Update each dependent @aztec package version in package.json.
    for PKG in $(jq --raw-output ".dependencies | keys[] | select(contains(\"@aztec/\"))" package.json); do
        jq --arg v $VERSION ".dependencies[\"$PKG\"] = \$v" package.json > $TMP && mv $TMP package.json
    done
    fi

    # Publish
    if [ -n "${COMMIT_TAG:-}" ] ; then 
        npm publish $TAG_ARG --access public
    else
        npm publish --dry-run $TAG_ARG --access public
    fi
}
deploy_package foundation
deploy_package circuits.js
deploy_package types
deploy_package aztec.js
deploy_package l1-artifacts
deploy_package ethereum
deploy_package noir-compiler
deploy_package noir-contracts
deploy_package cli
deploy_package pxe
deploy_package acir-simulator
deploy_package archiver
deploy_package merkle-tree
deploy_package p2p
deploy_package sequencer-client
deploy_package world-state
deploy_package key-store
deploy_package aztec-node
deploy_package aztec-sandbox
