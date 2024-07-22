#!/bin/bash
set -e

INFRA=$1
DEPLOY_TAG=$2
API_KEY=$3

# When destroying and applying terraforms, they may not be
# ready for a while, as it must register with DNS etc.
# This script waits on a healthy status from the infra - a valid response to a request
# We retry every 20 seconds, and wait for a total of 5 minutes (15 times)

if [ "$INFRA" == "mainnet-fork" ]; then
  export ETHEREUM_HOST="https://$DEPLOY_TAG-mainnet-fork.aztec.network:8545/$API_KEY"
  curl -H "Content-Type: application/json" -X POST --data '{"method":"eth_chainId","params":[],"id":49,"jsonrpc":"2.0"}' \
    --connect-timeout 30 \
    --retry 15 \
    --retry-delay 20 \
    $ETHEREUM_HOST
elif [ "$INFRA" == "pxe" ]; then
  export PXE_URL="https://api.aztec.network/$DEPLOY_TAG/aztec-pxe/$API_KEY/status"
  curl \
    --connect-timeout 30 \
    --retry 15 \
    --retry-delay 20 \
    $PXE_URL
else
  echo "Invalid infra type"
  exit 1
fi
