#!/usr/bin/env bash

set -euo pipefail

export ETHEREUM_HOST=https://$DEPLOY_TAG-mainnet-fork.aztec.network:8545/$FORK_API_KEY

REPOSITORY="l1-contracts"

CONTENT_HASH=$(calculate_content_hash $REPOSITORY)

echo "Last successfully published commit: $CONTENT_HASH"

# Check if image hash has alredy been deployed.
if check_rebuild "cache-$CONTENT_HASH-$DEPLOY_TAG-deployed" $REPOSITORY; then
  echo "No changes detected, no contract deploy necessary."
  # Set global variable for redeployment of contracts
  echo export CONTRACTS_DEPLOYED=0 >>$BASH_ENV
  exit 0
fi

# Login to pull our ecr images with docker.
retry ecr_login

# Contract addresses will be saved in the serve directory
mkdir -p serve
FILE_PATH=./serve/contract_addresses.json
CLI_IMAGE=$(calculate_image_uri cli)
retry docker pull $CLI_IMAGE

# remove 0x prefix from private key
PRIVATE_KEY=${CONTRACT_PUBLISHER_PRIVATE_KEY#0x}
docker run \
  $CLI_IMAGE \
  deploy-l1-contracts -u $ETHEREUM_HOST -p $PRIVATE_KEY | tee ./serve/contract_addresses.json

## Result format is:
# Rollup Address: 0xe33d37702bb94e83ca09e7dc804c9f4c4ab8ee4a
# Registry Address: 0xf02a70628c4e0d7c41f231f9af24c1678a030438
# L1 -> L2 Inbox Address: 0xdf34a07c7da15630d3b5d6bb17651d548a6e9d8f
# L2 -> L1 Outbox address: 0xf6b1b3c2c393fe55fe577a1f528bd72a76589ab0
# Contract Deployment Emitter Address: 0xf3ecc6e9428482a74687ee5f7b96f4dff8781454
# Availability Oracle Address: 0x610178da211fef7d417bc0e6fed39f05609ad788
# Gas Token Address: 0x9e4b815648c4a98a9bce6a899cecbaf3758cf23c
# Gas Portal Address: 0xda5dea39534f67f33deb38ec3b1e438fa893bf2c

# Read the file line by line
while IFS= read -r line; do
  # Extract the hexadecimal address using awk
  address=$(echo "$line" | awk '{print $NF}')

  # Assign the address to the respective variable based on the line content
  if [[ $line == *"Rollup"* ]]; then
    export TF_VAR_ROLLUP_CONTRACT_ADDRESS=$address
    echo "TF_VAR_ROLLUP_CONTRACT_ADDRESS=$TF_VAR_ROLLUP_CONTRACT_ADDRESS"
  elif [[ $line == *"Registry"* ]]; then
    export TF_VAR_REGISTRY_CONTRACT_ADDRESS=$address
    echo "TF_VAR_REGISTRY_CONTRACT_ADDRESS=$TF_VAR_REGISTRY_CONTRACT_ADDRESS"
  elif [[ $line == *"Inbox"* ]]; then
    export TF_VAR_INBOX_CONTRACT_ADDRESS=$address
    echo "TF_VAR_INBOX_CONTRACT_ADDRESS=$TF_VAR_INBOX_CONTRACT_ADDRESS"
  elif [[ $line == *"Outbox"* ]]; then
    export TF_VAR_OUTBOX_CONTRACT_ADDRESS=$address
    echo "TF_VAR_OUTBOX_CONTRACT_ADDRESS=$TF_VAR_OUTBOX_CONTRACT_ADDRESS"
  elif [[ $line == *"Oracle"* ]]; then
    export TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$address
    echo "TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS=$TF_VAR_AVAILABILITY_ORACLE_CONTRACT_ADDRESS"
  elif [[ $line == *"Gas Token"* ]]; then
    export TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$address
    echo "TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS=$TF_VAR_GAS_TOKEN_CONTRACT_ADDRESS"
  elif [[ $line == *"Gas Portal"* ]]; then
    export TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$address
    echo "TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS=$TF_VAR_GAS_PORTAL_CONTRACT_ADDRESS"
  else
    echo "Unknown contract address: $line"
  fi
done <"$FILE_PATH"

if [ "$DRY_DEPLOY" -eq 1 ]; then
  echo "DRY_DEPLOY: deploy_terraform l1-contracts ./terraform"
  echo "DRY_DEPLOY: tag_remote_image $REPOSITORY cache-$CONTENT_HASH cache-$CONTENT_HASH-$DEPLOY_TAG-deployed"
else
  # Write TF state variables
  deploy_terraform l1-contracts ./terraform

  # Tag the image as deployed.
  retry tag_remote_image $REPOSITORY cache-$CONTENT_HASH cache-$CONTENT_HASH-$DEPLOY_TAG-deployed
fi

# Set global variable for redeployment of contracts
echo export CONTRACTS_DEPLOYED=1 >>$BASH_ENV
