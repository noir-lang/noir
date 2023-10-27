#!/bin/bash
# Compiles all noir contracts

source ./scripts/nargo_check.sh

echo "Checking noir version"
nargo_check

# Runs the compile scripts for all contracts.
echo "Compiling all contracts"

# ./scripts/compile.sh $(./scripts/get_all_contracts.sh)
nargo compile --workspace --no-backend
