#!/bin/bash

# We use this script just for CI so we assume we're running on x86 linux

mkdir -p $HOME/.barretenberg
curl -o ./barretenberg-x86_64-linux-gnu.tar.gz -L https://github.com/AztecProtocol/aztec-packages/releases/download/aztec-packages-v0.41.0/barretenberg-x86_64-linux-gnu.tar.gz
tar -xvf ./barretenberg-x86_64-linux-gnu.tar.gz -C $HOME/.barretenberg/
echo 'export PATH=$PATH:$HOME/.barretenberg/' >> ~/.bashrc
source ~/.bashrc
