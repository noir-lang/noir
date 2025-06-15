#!/bin/bash

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if we're already in sensei_stdlib directory
if [[ "$(basename "$PWD")" != "sensei_stdlib" ]]; then
    # Navigate to sensei_stdlib from the script directory
    cd "$SCRIPT_DIR/../sensei_stdlib" || {
        echo "Error: Could not find sensei_stdlib directory"
        exit 1
    }
fi

# Run nargo CLI with forwarded arguments
cargo run -p nargo_cli -- "$@"