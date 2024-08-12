#!/bin/bash
set -e
source ../utils/setup.sh

echo "Test: Basic flow, no aliases"

ACCOUNT_ADDRESS=$(aztec-wallet create-account -a main | grep "Address:" | awk '{print $2}')
TOKEN_ADDRESS=$(aztec-wallet deploy ./target/token_contract-Token.json --args $ACCOUNT_ADDRESS Test TST 18 -f $ACCOUNT_ADDRESS | grep "Contract deployed at" | awk '{print $4}')
aztec-wallet send mint_public -c ./target/token_contract-Token.json -ca $TOKEN_ADDRESS --args $ACCOUNT_ADDRESS 42 -f $ACCOUNT_ADDRESS
RESULT=$(aztec-wallet simulate balance_of_public -c ./target/token_contract-Token.json -ca $TOKEN_ADDRESS --args $ACCOUNT_ADDRESS -f $ACCOUNT_ADDRESS | grep "Simulation result:" | awk '{print $3}')

if [ $RESULT = "42n" ]; then
    echo "Test passed"
else 
    exit 1
fi

echo
echo "---------------------------------"
