#!/bin/bash

BB_VERSION=$1

sed -i.bak "s/^VERSION=.*/VERSION=\"$BB_VERSION\"/" ./scripts/install_bb.sh  && rm ./scripts/install_bb.sh.bak

tmp=$(mktemp)
BACKEND_BARRETENBERG_PACKAGE_JSON=./tooling/noir_js_backend_barretenberg/package.json
jq --arg v $BB_VERSION '.dependencies."@aztec/bb.js" = $v' $BACKEND_BARRETENBERG_PACKAGE_JSON > $tmp && mv $tmp $BACKEND_BARRETENBERG_PACKAGE_JSON

YARN_ENABLE_IMMUTABLE_INSTALLS=false yarn install
