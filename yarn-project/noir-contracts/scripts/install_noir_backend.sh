#!/bin/bash

# Script to install the current barretenberg backend

set -eu

BACKEND_NAME="acvm-backend-barretenberg"

# If the backend is not installed, then install it
if [ -z $(nargo backend ls | grep $BACKEND_NAME) ]; then
    echo "Installing $BACKEND_NAME"
    nargo backend install $BACKEND_NAME https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.5.1/barretenberg-x86_64-linux-gnu.tar.gz
fi
