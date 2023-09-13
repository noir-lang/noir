#!/bin/bash
# Compiles all noir contracts

source ./scripts/nargo_check.sh

echo "Checking noir version"
nargo_check

# Runs the compile scripts for all contracts.
echo "Compiling all contracts"

nargo compile --workspace