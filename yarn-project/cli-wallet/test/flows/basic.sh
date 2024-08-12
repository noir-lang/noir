#!/bin/bash
set -e
source ../utils/setup.sh

echo "Test: Basic flow"
echo

aztec-wallet create-account -a main
aztec-wallet deploy token_contract@Token --args accounts:main Test TST 18 -f main
aztec-wallet send mint_public -ca contracts:last --args accounts:main 42 -f main
RESULT=$(aztec-wallet simulate balance_of_public -ca contracts:last --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')

if [ $RESULT = "42n" ]; then
    echo
    echo "Test passed"
else 
    exit 1
fi

echo
echo "---------------------------------"