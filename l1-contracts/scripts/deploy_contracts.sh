#!/usr/bin/env bash
set -eu

# Sets up defaults then runs the E2E Setup script to perform contract deployments
#
# Expected enviornment variables
# -ROLLUP_CONTRACT_ADDRESS
# -CONTRACT_DEPLOYMENT_EMITTER_ADDRESS
# -INBOX_CONTRACT_ADDRESS
# -OUTBOX_CONTRACT_ADDRESS
# -REGISTRY_CONTRACT_ADDRESS

# Create serve directory in which we save contract_addresses.json.
mkdir -p serve/
echo "Created output directory"

extract_deployed_to() {
  grep 'Deployed to:' | awk '{print $3}'
}

deploy_contract() {
  CONTRACT="$1"
  CONSTRUCTOR_ARGS=(${2-})
  if [ ${#CONSTRUCTOR_ARGS[@]} -gt 0 ]; then
    forge create --rpc-url $ETHEREUM_HOST --private-key $PRIVATE_KEY $CONTRACT --constructor-args $CONSTRUCTOR_ARGS
  else
    forge create --rpc-url $ETHEREUM_HOST --private-key $PRIVATE_KEY $CONTRACT
  fi
}

export REGISTRY_CONTRACT_ADDRESS=$(deploy_contract ./src/core/messagebridge/Registry.sol:Registry | extract_deployed_to)
export INBOX_CONTRACT_ADDRESS=$(deploy_contract ./src/core/messagebridge/Inbox.sol:Inbox "$REGISTRY_CONTRACT_ADDRESS" | extract_deployed_to)
export OUTBOX_CONTRACT_ADDRESS=$(deploy_contract ./src/core/messagebridge/Outbox.sol:Outbox "$REGISTRY_CONTRACT_ADDRESS" | extract_deployed_to)
export ROLLUP_CONTRACT_ADDRESS=$(deploy_contract ./src/core/Rollup.sol:Rollup "$REGISTRY_CONTRACT_ADDRESS" | extract_deployed_to)
export CONTRACT_DEPLOYMENT_EMITTER_ADDRESS=$(deploy_contract ./src/periphery/ContractDeploymentEmitter.sol:ContractDeploymentEmitter | extract_deployed_to)

# Store contract addresses in a JSON file
jq -n \
  --arg registryAddress "$REGISTRY_CONTRACT_ADDRESS" \
  --arg inboxAddress "$INBOX_CONTRACT_ADDRESS" \
  --arg outboxAddress "$OUTBOX_CONTRACT_ADDRESS" \
  --arg rollupAddress "$ROLLUP_CONTRACT_ADDRESS" \
  --arg emitterAddress "$CONTRACT_DEPLOYMENT_EMITTER_ADDRESS" \
  '{
        REGISTRY_CONTRACT_ADDRESS: $registryAddress,
        INBOX_CONTRACT_ADDRESS: $inboxAddress,
        OUTBOX_CONTRACT_ADDRESS: $outboxAddress,
        ROLLUP_CONTRACT_ADDRESS: $rollupAddress,
        CONTRACT_DEPLOYMENT_EMITTER_ADDRESS: $emitterAddress
    }' >serve/contract_addresses.json

cat serve/contract_addresses.json

echo "Contract addresses have been written to serve/contract_addresses.json"
