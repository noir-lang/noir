#!/bin/bash
set -e
source ../utils/setup.sh

test_title "Basic flow"

AMOUNT=42

aztec-wallet create-account -a main
aztec-wallet deploy token_contract@Token --args accounts:main Test TST 18 -f main
aztec-wallet send mint_public -ca last --args accounts:main $AMOUNT -f main
RESULT=$(aztec-wallet simulate balance_of_public -ca last --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')

section "Main account public balance is ${RESULT}"

assert_eq ${RESULT} "${AMOUNT}n"
