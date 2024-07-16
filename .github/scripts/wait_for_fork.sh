#!/bin/bash
set -e

DEPLOY_TAG=$1
TEST_FORK_API_KEY=$2

# When destroying and applying mainnet fork terraform, it may not be
# ready for a while, as it must register with DNS etc.
# This script waits on a healthy status from the fork - a valid response to the chainid request
# We retry every 20 seconds, and wait for a total of 5 minutes (15 times)
export ETHEREUM_HOST="https://$DEPLOY_TAG-mainnet-fork.aztec.network:8545/$TEST_FORK_API_KEY"

curl -H "Content-Type: application/json" -X POST --data '{"method":"eth_chainId","params":[],"id":49,"jsonrpc":"2.0"}' \
  --connect-timeout 30 \
  --retry 15 \
  --retry-delay 20 \
  $ETHEREUM_HOST
