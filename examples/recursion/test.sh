#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.

BACKEND=${BACKEND:-bb}

./generate_recursive_proof.sh