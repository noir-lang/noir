#!/bin/bash
set -e
source ../utils/setup.sh

echo "Test: Create an account and deploy using native fee payment with bridging" 
echo

echo
echo ///////////////////////////////////////////////////////////////////
echo // WARNING: this test requires protocol contracts to be deployed //
echo //         > aztec deploy-protocol-contracts                     //
echo ///////////////////////////////////////////////////////////////////
echo

aztec-wallet create-account -a main --register-only
aztec-wallet bridge-fee-juice 100000000000000000 accounts:main --mint

echo
echo "Create a bootstrapping account just to force block creation"
echo
aztec-wallet create-account -a bootstrap
aztec-wallet deploy counter_contract@Counter --init initialize --args 0 accounts:main accounts:main -f bootstrap -a counter
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f bootstrap
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f bootstrap

echo
echo "Deploy main account claiming the fee juice, use it later"
echo
aztec-wallet deploy-account -f main --payment method=native,claim
# These should use --payment method=native
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f main 
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f main

RESULT=$(aztec-wallet simulate get_counter -ca counter --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')

if [ $RESULT = "4n" ]; then
    echo
    echo "Test passed"
else 
    exit 1
fi


echo
echo "---------------------------------"