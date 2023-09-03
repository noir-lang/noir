#!/bin/bash
# Script to install noirup and the latest aztec nargo
set -eu

VERSION="aztec"

# Install nargo
noirup -v $VERSION
