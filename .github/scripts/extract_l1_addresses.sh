#!/usr/bin/env bash

FILE_PATH=$1

# Read the file line by line
while IFS= read -r line; do
  # Extract the hexadecimal address using awk
  address=$(echo "$line" | awk '{print $NF}')

  # Assign the address to the respective variable based on the line content
  if [[ $line == *"Rollup Address"* ]]; then
    export TF_VAR_ROLLUP_CONTRACT_ADDRESS=$address
    echo "TF_VAR_ROLLUP_CONTRACT_ADDRESS=$TF_VAR_ROLLUP_CONTRACT_ADDRESS"
  elif [[ $line == *"Registry Address"* ]]; then
    export TF_VAR_REGISTRY_CONTRACT_ADDRESS=$address
    echo "TF_VAR_REGISTRY_CONTRACT_ADDRESS=$TF_VAR_REGISTRY_CONTRACT_ADDRESS"
  elif [[ $line == *"Inbox Address"* ]]; then
    export TF_VAR_INBOX_CONTRACT_ADDRESS=$address
    echo "TF_VAR_INBOX_CONTRACT_ADDRESS=$TF_VAR_INBOX_CONTRACT_ADDRESS"
  elif [[ $line == *"Outbox Address"* ]]; then
    export TF_VAR_OUTBOX_CONTRACT_ADDRESS=$address
    echo "TF_VAR_OUTBOX_CONTRACT_ADDRESS=$TF_VAR_OUTBOX_CONTRACT_ADDRESS"
  elif [[ $line == *"Oracle Address"* ]]; then
    export TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$address
    echo "TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS"
  elif [[ $line == *"Gas Token Address"* ]]; then
    export TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$address
    echo "TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS"
  elif [[ $line == *"Gas Portal Address"* ]]; then
    export TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$address
    echo "TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS"
  else
    echo "Unknown contract address: $line"
  fi
done <"$FILE_PATH"

# echo all addresses into github env
echo "TF_VAR_ROLLUP_CONTRACT_ADDRESS=$TF_VAR_ROLLUP_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_REGISTRY_CONTRACT_ADDRESS=$TF_VAR_REGISTRY_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_INBOX_CONTRACT_ADDRESS=$TF_VAR_INBOX_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_OUTBOX_CONTRACT_ADDRESS=$TF_VAR_OUTBOX_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS" >>$GITHUB_ENV
echo "TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS" >>$GITHUB_ENV

# Set global variable for redeployment of contracts
echo "CONTRACTS_DEPLOYED=1" >>$GITHUB_ENV
