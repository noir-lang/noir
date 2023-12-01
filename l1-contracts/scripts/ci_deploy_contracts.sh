#!/bin/bash

FORCE_DEPLOY=${2:-"false"}

export ETHEREUM_HOST=$DEPLOY_TAG-mainnet-fork.aztec.network:8545/$FORK_API_KEY

# If we have previously successful commit, we can early out if nothing relevant has changed since.
if [[ $FORCE_DEPLOY == 'false' ]] && check_rebuild cache-"$CONTENT_HASH" $REPOSITORY; then
  echo "No contract deploy necessary."
  exit 0
fi

mkdir -p serve
# Contract addresses will be mounted in the serve directory
docker run \
  -v $(pwd)/serve:/usr/src/contracts/serve \
  -e ETHEREUM_HOST=$ETHEREUM_HOST -e PRIVATE_KEY=$CONTRACT_PUBLISHER_PRIVATE_KEY \
  aztecprotocol/l1-contracts:$DEPLOY_TAG \
  ./scripts/deploy_contracts.sh

# Write the contract addresses as terraform variables
for KEY in ROLLUP_CONTRACT_ADDRESS REGISTRY_CONTRACT_ADDRESS INBOX_CONTRACT_ADDRESS OUTBOX_CONTRACT_ADDRESS; do
  VALUE=$(jq -r .$KEY ./serve/contract_addresses.json)
  export TF_VAR_$KEY=$VALUE
done

# Write TF state variables
deploy_terraform l1-contracts ./terraform
