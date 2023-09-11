#!/bin/bash

# Install noir if it is not installed already
if ! command -v noirup &> /dev/null
then
    echo "Installing noir"
    source ./scripts/install_noirup.sh
fi

# Update noir
./scripts/install_noir.sh
./scripts/install_noir_backend.sh

# Use yarn script to compile and create types
yarn 
yarn noir:build:all