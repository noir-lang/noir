#!/bin/bash

export ETHEREUM_HOST=https://$DEPLOY_TAG-mainnet-fork.aztec.network:8545/$FORK_API_KEY

REPOSITORY="l1-contracts"

CONTENT_HASH=$(calculate_content_hash $REPOSITORY)

echo "Last successfully published commit: $CONTENT_HASH"

# Check if image hash has alredy been deployed.
if check_rebuild "cache-$CONTENT_HASH-$DEPLOY_TAG-deployed" $REPOSITORY; then
  echo "No changes detected, no contract deploy necessary."
  exit 0
fi

# Login to pull our ecr images with docker.
ecr_login

mkdir -p serve
# Contract addresses will be mounted in the serve directory
docker run \
  -v $(pwd)/serve:/usr/src/l1-contracts/serve \
  -e ETHEREUM_HOST=$ETHEREUM_HOST -e PRIVATE_KEY=$CONTRACT_PUBLISHER_PRIVATE_KEY \
  "$ECR_URL/l1-contracts:cache-$CONTENT_HASH" \
  ./scripts/deploy_contracts.sh

# Write the contract addresses as terraform variables
for KEY in ROLLUP_CONTRACT_ADDRESS REGISTRY_CONTRACT_ADDRESS INBOX_CONTRACT_ADDRESS OUTBOX_CONTRACT_ADDRESS; do
  VALUE=$(jq -r .$KEY ./serve/contract_addresses.json)
  export TF_VAR_$KEY=$VALUE
done

# Write TF state variables
deploy_terraform l1-contracts ./terraform

# Tag the image as deployed.
retry tag_remote_image $REPOSITORY cache-$CONTENT_HASH cache-$CONTENT_HASH-$DEPLOY_TAG-deployed
