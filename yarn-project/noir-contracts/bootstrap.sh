#!/bin/bash

# Install noir if it is not installed already
if ! command -v nargo &> /dev/null
then
    echo "Installing noir"
    ./scripts/install_noir.sh
fi


# Use yarn script to compile and create types
yarn 
yarn noir:build:all