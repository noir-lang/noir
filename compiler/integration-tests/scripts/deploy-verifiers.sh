#!/bin/bash

self_path=$(dirname "$(readlink -f "$0")")
cd $self_path/../foundry-project

forge build

CI=${CI:-false}
if [[ $CI = true ]]; then
    anvil > /dev/null 2>&1 &
else
    anvil &
    pid=$!
fi

sleep 10

forge create --rpc-url http://127.0.0.1:8545 --mnemonic "test test test test test test test test test test test junk" src/1_mul.sol:UltraVerifier --json > mul_output.json
forge create --rpc-url http://127.0.0.1:8545 --mnemonic "test test test test test test test test test test test junk" src/main.sol:UltraVerifier  --json > main_output.json

if [[ $CI = false ]]; then
    wait $pid
fi
