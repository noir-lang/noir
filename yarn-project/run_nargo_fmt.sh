#!/usr/bin/env bash

# Note: This script formats the files multiple times when the given project is included in a workspace.
#       Tackling this became a time sink, so I decided to leave it as is for now.

set -e

# We set the executable path as if we were in CI
nargo_executable="/usr/src/noir/target/release/nargo"

# Check if nargo_executable exists and is executable
if [ ! -x "$nargo_executable" ]; then
    # If not, we try to set a nargo path as if the script was run locally
    nargo_executable="$(git rev-parse --show-toplevel)/noir/target/release/nargo"

    if [ ! -x "$nargo_executable" ]; then
        echo "Error: nargo executable not found"
        exit 1
    fi
fi

# Find all Nargo.toml files and run 'nargo fmt'
find . -name "Nargo.toml" | while read -r file; do
    # Extract the directory from the file path
    dir=$(dirname "$file")
    
    # Change into the directory
    cd "$dir" || exit

    # Run 'nargo fmt' in the directory and pass in the input param 1
    "$nargo_executable" fmt $1

    # Change back to the original directory
    cd - > /dev/null
done
