#!/usr/bin/env bash

set -eum pipefail

# Replace API_KEY in nginx config
echo "Replacing api key with $API_KEY in nginx config..."
sed -i 's/{{API_KEY}}/'$API_KEY'/' /etc/nginx/gateway.conf

# Run nginx and anvil alongside each other
trap 'kill $(jobs -p)' SIGTERM

# Anvil defaults - Nginx assumes these values to be as they are
HOST="0.0.0.0"
PORT=8544
ETHEREUM_HOST=$HOST:$PORT
# Stripping double quotations from the mnemonic seed phrase
echo "stripping double quotations from the mnemonic seed phrase: ${MNEMONIC:0:10}..."
MNEMONIC_STRIPPED=${MNEMONIC//\"/}
echo "result: ${MNEMONIC_STRIPPED:0:10}..."

# Data directory for anvil state
mkdir -p /data

# Run anvil silently
.foundry/bin/anvil --silent --host $HOST -p $PORT -m "$MNEMONIC_STRIPPED" -f=https://mainnet.infura.io/v3/$INFURA_API_KEY --chain-id=$L1_CHAIN_ID --fork-block-number=15918000 --block-base-fee-per-gas=10 -s=$SNAPSHOT_FREQUENCY --state=./data/state --balance=1000000000000000000 >/dev/null &

echo "Waiting for ethereum host at $ETHEREUM_HOST..."
while ! curl -s $ETHEREUM_HOST >/dev/null; do sleep 1; done

echo "Starting nginx..."
nginx &
wait
