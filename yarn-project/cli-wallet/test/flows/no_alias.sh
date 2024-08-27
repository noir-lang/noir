#!/bin/bash
set -e
source ../utils/setup.sh

test_title "Basic flow, no aliases"

AMOUNT=42

ACCOUNT_ADDRESS=$(aztec-wallet create-account -a main | grep "Address:" | awk '{print $2}')
TOKEN_ADDRESS=$(aztec-wallet deploy ./target/token_contract-Token.json --args $ACCOUNT_ADDRESS Test TST 18 -f $ACCOUNT_ADDRESS | grep "Contract deployed at" | awk '{print $4}')
aztec-wallet send mint_public -c ./target/token_contract-Token.json -ca $TOKEN_ADDRESS --args $ACCOUNT_ADDRESS $AMOUNT -f $ACCOUNT_ADDRESS
RESULT=$(aztec-wallet simulate balance_of_public -c ./target/token_contract-Token.json -ca $TOKEN_ADDRESS --args $ACCOUNT_ADDRESS -f $ACCOUNT_ADDRESS | grep "Simulation result:" | awk '{print $3}')

section "Main account public balance is ${RESULT}"

assert_eq ${RESULT} "${AMOUNT}n"
