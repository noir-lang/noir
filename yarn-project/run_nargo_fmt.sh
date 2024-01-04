#!/bin/bash

nargo_executable="$(git rev-parse --show-toplevel)/noir/target/release/nargo"

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
