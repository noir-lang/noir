#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.

BACKEND=${BACKEND:-bb}

rm -f ./src/contract.sol

./solidity_verifier.sh

if ! [ -f ./src/contract.sol ]; then
    printf '%s\n' "Contract not written to file" >&2
    exit 1
fi