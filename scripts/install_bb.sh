#!/bin/bash

# We use this script just for CI so we assume we're running on x86 linux

mkdir -p $HOME/.barretenberg
curl -o ./barretenberg-aarch64-apple-darwin.tar.gz -L https://github.com/AztecProtocol/aztec-packages/releases/download/aztec-packages-v0.38.0/barretenberg-aarch64-apple-darwin.tar.gz
tar -xvf ./barretenberg-aarch64-apple-darwin.tar.gz -C $HOME/.barretenberg/
echo 'export PATH=$PATH:$HOME/.barretenberg/' >> ~/.bashrc
source ~/.bashrc
