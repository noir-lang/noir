#!/bin/bash
set -e
source ../utils/setup.sh

test_title "Tx management"

aztec-wallet create-account -a main
aztec-wallet deploy counter_contract@Counter --init initialize --args 0 accounts:main accounts:main -a counter -f main
aztec-wallet send increment -ca counter --args accounts:main accounts:main -f main

TX_LIST=$(aztec-wallet get-tx)

echo "${TX_LIST}"

TX_HASH=$(echo "${TX_LIST}" | grep "transactions:last" | awk '{print $3}')

section Last transaction hash is ${TX_HASH}

TX_STATUS=$(aztec-wallet get-tx ${TX_HASH} | grep "Status: " | awk '{print $2}')

assert_eq ${TX_STATUS} "success"



