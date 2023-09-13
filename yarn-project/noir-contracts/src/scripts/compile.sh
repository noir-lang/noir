#!/bin/bash

# Example:

# ./compile.sh private_token ecdsa_account
# or
# yarn noir:build private_token ecdsa_account

# Enable strict mode:
# Exit on error (set -e), treat unset variables as an error (set -u),
# and propagate the exit status of the first failing command in a pipeline (set -o pipefail).
set -euo pipefail;

# Run build scripts
./scripts/compile.sh "$@"
echo "Generating types"
./scripts/types.sh "$@"