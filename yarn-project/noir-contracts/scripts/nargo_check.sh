#!/bin/bash

# Check nargo version matches the expected one
nargo_check() {
    echo "Using $(nargo --version)"
    EXPECTED_VERSION=$(jq -r '.commit' ../noir-compiler/src/noir-version.json)
    FOUND_VERSION=$(nargo --version | grep -oP 'git version hash: \K[0-9a-f]+')
    if [ "$EXPECTED_VERSION" != "$FOUND_VERSION" ]; then
      echo "Expected nargo version $EXPECTED_VERSION but found version $FOUND_VERSION. Aborting."
      exit 1
    fi
}
