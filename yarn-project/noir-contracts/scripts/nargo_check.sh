#!/bin/bash

# Check nargo version matches the expected one
nargo_check() {
    echo "Using $(nargo --version)"
    EXPECTED_VERSION=$(jq -r '.commit' ../noir-compiler/src/noir-version.json)
    FOUND_VERSION=$(nargo --version | grep -o 'git version hash: [0-9a-f]*' | cut -d' ' -f4)
    if [ "$EXPECTED_VERSION" != "$FOUND_VERSION" ]; then
        echo "Expected nargo version $EXPECTED_VERSION but found version $FOUND_VERSION. Aborting."

        # Get the directory of the script and the parent directory, where its meant to be run
        SCRIPT_DIR="$(dirname "$(realpath "$0")")"
        PARENT_DIR="$(dirname "$SCRIPT_DIR")"

        # bootstrap.sh calls this non-interactively, so we can't just ask for confirmation to autorun this script.
        echo "To fix the version issue, you can run the following command:"
        echo "cd $PARENT_DIR && scripts/install_noir.sh"
        exit 1
    fi
}
