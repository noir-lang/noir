#!/bin/bash
# Run the types script for all files
./scripts/types.sh $(./scripts/get_all_contracts.sh)

# Remove the debug files as they are no longer needed and can cause prettier and build issues
rm -r ./target/debug*