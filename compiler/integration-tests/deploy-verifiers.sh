#!/bin/bash

cd ../..

anvil > /dev/null 2>&1 &
sleep 10

cd foundry-project

forge build

forge create --rpc-url http://127.0.0.1:8545 --mnemonic "test test test test test test test test test test test junk" src/1_mul.sol:UltraVerifier --json > mul_output.json
forge create --rpc-url http://127.0.0.1:8545 --mnemonic "test test test test test test test test test test test junk" src/main.sol:UltraVerifier  --json > main_output.json
