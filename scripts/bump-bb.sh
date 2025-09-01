#!/bin/bash

BB_VERSION=$1

sed -i.bak "s/^VERSION=.*/VERSION=\"$BB_VERSION\"/" ./scripts/install_bb.sh  && rm ./scripts/install_bb.sh.bak

tmp=$(mktemp)
INTEGRATION_TESTS_PACKAGE_JSON=./compiler/integration-tests/package.json
jq --arg v $BB_VERSION '.dependencies."@aztec/bb.js" = $v' $INTEGRATION_TESTS_PACKAGE_JSON > $tmp && mv $tmp $INTEGRATION_TESTS_PACKAGE_JSON

tmp=$(mktemp)
BROWSER_EXAMPLE_PACKAGE_JSON=./examples/docs_tutorials/browser/package.json
jq --arg v $BB_VERSION '.dependencies."@aztec/bb.js" = $v' $BROWSER_EXAMPLE_PACKAGE_JSON > $tmp && mv $tmp $BROWSER_EXAMPLE_PACKAGE_JSON

YARN_ENABLE_IMMUTABLE_INSTALLS=false yarn install
