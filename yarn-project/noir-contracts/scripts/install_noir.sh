#!/bin/bash
# Script to install noirup and the latest aztec nargo
set -eu

VERSION="${VERSION:-$(jq -r '.tag' ../noir-compiler/src/noir-version.json)}"

# Install nargo
noirup -v $VERSION
