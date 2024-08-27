#!/bin/bash
set -e
source ../utils/setup.sh

test_title "Shield and private transfer"

MINT_AMOUNT=42
TRANSFER_AMOUNT=21

source $TEST_FOLDER/token_utils/create_main_and_mint_private.sh $MINT_AMOUNT

aztec-wallet create-account -a recipient

aztec-wallet send transfer -ca token --args accounts:recipient $TRANSFER_AMOUNT -f main

RESULT_MAIN=$(aztec-wallet simulate balance_of_private -ca token --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')
RESULT_RECIPIENT=$(aztec-wallet simulate balance_of_private -ca token --args accounts:recipient -f recipient | grep "Simulation result:" | awk '{print $3}')

section "Main account private balance is ${RESULT_MAIN}, recipient account private balance is ${RESULT_RECIPIENT}"

assert_eq ${RESULT_MAIN} ${RESULT_RECIPIENT}
