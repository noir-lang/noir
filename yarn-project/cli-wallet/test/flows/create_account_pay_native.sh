#!/bin/bash
set -e
source ../utils/setup.sh

test_title "Create an account and deploy using native fee payment with bridging"

echo
warn ///////////////////////////////////////////////////////////////////
warn // WARNING: this test requires protocol contracts to be deployed //
warn //         > aztec deploy-protocol-contracts                     //
warn ///////////////////////////////////////////////////////////////////
echo

aztec-wallet create-account -a main --register-only
aztec-wallet bridge-fee-juice 100000000000000000 main --mint --no-wait


section "Create a bootstrapping account just to force block creation"

aztec-wallet create-account -a bootstrap
aztec-wallet deploy counter_contract@Counter --init initialize --args 0 accounts:main accounts:main -f bootstrap -a counter
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f bootstrap
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f bootstrap


section "Deploy main account claiming the fee juice, use it later"

aztec-wallet deploy-account -f main --payment method=native,claim
# These should use --payment method=native
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f main
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f main

RESULT=$(aztec-wallet simulate get_counter -ca counter --args accounts:main -f main | grep "Simulation result:" | awk '{print $3}')

section "Counter is ${RESULT}"

assert_eq ${RESULT} "4n"