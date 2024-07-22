#!/usr/bin/env bash

TO_EXTRACT=$1
FILE_PATH=$2

if [[ $TO_EXTRACT == "l1-contracts" ]]; then
  # Extract l1 contract addresses

  JSON_OUTPUT='{'

  # Read the file line by line
  while IFS= read -r line; do
    # Extract the hexadecimal address using awk
    address=$(echo "$line" | awk '{print $NF}')

    # Assign the address to the respective variable based on the line content
    if [[ $line == *"Rollup Address"* ]]; then
      export TF_VAR_ROLLUP_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "rollup_contract_address": "'$address'",'
    elif [[ $line == *"Registry Address"* ]]; then
      export TF_VAR_REGISTRY_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "registry_contract_address": "'$address'",'
    elif [[ $line == *"Inbox Address"* ]]; then
      export TF_VAR_INBOX_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "inbox_contract_address": "'$address'",'
    elif [[ $line == *"Outbox Address"* ]]; then
      export TF_VAR_OUTBOX_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "outbox_contract_address": "'$address'",'
    elif [[ $line == *"Oracle Address"* ]]; then
      export TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "availability_oracle_contract_address": "'$address'",'
    elif [[ $line == *"Gas Token Address"* ]]; then
      export TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "gas_token_contract_address": "'$address'",'
    elif [[ $line == *"Gas Portal Address"* ]]; then
      export TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$address
      JSON_OUTPUT+=' "gas_portal_contract_address": "'$address'",'
    else
      echo "Unknown contract address: $line"
    fi
  done <"$FILE_PATH"

  # Remove the last comma and close the JSON object
  JSON_OUTPUT=${JSON_OUTPUT%,}
  JSON_OUTPUT+=' }'

  # echo all addresses into github env
  echo "TF_VAR_ROLLUP_CONTRACT_ADDRESS=$TF_VAR_ROLLUP_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_REGISTRY_CONTRACT_ADDRESS=$TF_VAR_REGISTRY_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_INBOX_CONTRACT_ADDRESS=$TF_VAR_INBOX_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_OUTBOX_CONTRACT_ADDRESS=$TF_VAR_OUTBOX_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS" >>$GITHUB_ENV
  echo "TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS" >>$GITHUB_ENV

  # Output to JSON file
  echo $JSON_OUTPUT >./l1-contracts.json

elif [[ $TO_EXTRACT == "account" ]]; then
  # Extract aztec account private key
  OUTPUT=$(cat $FILE_PATH)

  AZTEC_PRIVATE_KEY=$(echo "$OUTPUT" | grep "Private key:" | awk '{print $NF}')
  AZTEC_ADDRESS=$(echo "$OUTPUT" | grep "Address:" | awk '{print $NF}')

  # Print the private key and address into github env
  echo "AZTEC_PRIVATE_KEY=$AZTEC_PRIVATE_KEY" >>$GITHUB_ENV
  echo "AZTEC_ADDRESS=$AZTEC_ADDRESS" >>$GITHUB_ENV

  # Export
  export AZTEC_PRIVATE_KEY=$AZTEC_PRIVATE_KEY
  export AZTEC_ADDRESS=$AZTEC_ADDRESS

elif [[ $TO_EXTRACT == "l2-bootstrap" ]]; then
  # Extract l2 bootstrap contract addresses

  # Read the log output from a file
  OUTPUT=$(cat $FILE_PATH)

  KEY_REGISTRY_ADDRESS=$(echo "$OUTPUT" | grep "Deployed Key Registry on L2 at" | awk '{print $NF}')

  AUTH_REGISTRY_ADDRESS=$(echo "$OUTPUT" | grep "Deployed Auth Registry on L2 at" | awk '{print $NF}')

  FEE_JUICE_ADDRESS=$(echo "$OUTPUT" | grep "Deployed Gas Token on L2 at" | awk '{print $NF}')

  # Print the extracted into github env
  echo "KEY_REGISTRY_ADDRESS=$KEY_REGISTRY_ADDRESS" >>$GITHUB_ENV
  echo "AUTH_REGISTRY_ADDRESS=$AUTH_REGISTRY_ADDRESS" >>$GITHUB_ENV
  echo "FEE_JUICE_ADDRESS=$FEE_JUICE_ADDRESS" >>$GITHUB_ENV

  # Export
  export KEY_REGISTRY_ADDRESS=$KEY_REGISTRY_ADDRESS
  export AUTH_REGISTRY_ADDRESS=$AUTH_REGISTRY_ADDRESS
  export FEE_JUICE_ADDRESS=$FEE_JUICE_ADDRESS

elif [[ $TO_EXTRACT == "l2-contract" ]]; then
  # Extract l2 contract addresses

  TOKEN_CONTRACT_NAME=$3

  OUTPUT=$(cat $FILE_PATH)

  CONTRACT_ADDRESS=$(echo "$OUTPUT" | grep "Contract deployed at" | awk '{print $NF}')

  echo "$TOKEN_CONTRACT_NAME=$CONTRACT_ADDRESS" >>$GITHUB_ENV

  # Export
  export $TOKEN_CONTRACT_NAME=$CONTRACT_ADDRESS
fi
