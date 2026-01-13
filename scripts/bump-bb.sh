#!/bin/bash

# Strip leading 'v' if present so the script accepts both "v3.0.0" and "3.0.0"
BB_VERSION=${1#v}

sed -i.bak "s/^VERSION=.*/VERSION=\"$BB_VERSION\"/" ./scripts/install_bb.sh  && rm ./scripts/install_bb.sh.bak

# Update bb_proof_verification tag in Nargo.toml files (these require a leading 'v')
grep -rl 'bb_proof_verification' --include='Nargo.toml' . | while read -r file; do
    sed -i.bak '/bb_proof_verification/s/tag *= *"v[^"]*"/tag = "v'"$BB_VERSION"'"/' "$file" && rm "$file.bak"
done

tmp=$(mktemp)
INTEGRATION_TESTS_PACKAGE_JSON=./compiler/integration-tests/package.json
jq --arg v $BB_VERSION '.dependencies."@aztec/bb.js" = $v' $INTEGRATION_TESTS_PACKAGE_JSON > $tmp && mv $tmp $INTEGRATION_TESTS_PACKAGE_JSON

tmp=$(mktemp)
BROWSER_EXAMPLE_PACKAGE_JSON=./examples/browser/package.json
jq --arg v $BB_VERSION '.dependencies."@aztec/bb.js" = $v' $BROWSER_EXAMPLE_PACKAGE_JSON > $tmp && mv $tmp $BROWSER_EXAMPLE_PACKAGE_JSON

YARN_ENABLE_IMMUTABLE_INSTALLS=false yarn install
