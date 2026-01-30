#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.

BACKEND=${BACKEND:-bb}

cd $(dirname $0)

rm -rf ./target

./compile_and_profile_gates.sh