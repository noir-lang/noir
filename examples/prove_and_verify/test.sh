#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.

BACKEND=${BACKEND:-bb}

rm -rf ./target ./proofs

./prove_and_verify.sh