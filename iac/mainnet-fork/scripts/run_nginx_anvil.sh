#!/usr/bin/env bash

set -eum pipefail

# Replace API_KEYs in nginx config
echo "Replacing api keys in nginx config..."
sed -i 's/{{PUBLIC_API_KEY}}/'$API_KEY'/g' /etc/nginx/gateway.conf
sed -i 's/{{ADMIN_API_KEY}}/'$FORK_ADMIN_API_KEY'/g' /etc/nginx/gateway.conf

# Resulting config
cat /etc/nginx/gateway.conf
echo

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

# Log directory for anvil
mkdir -p /var/log/anvil/

# Run anvil logging to stdout
.foundry/bin/anvil --block-time 12 --host $HOST -p $PORT -m "$MNEMONIC_STRIPPED" -f=https://mainnet.infura.io/v3/$INFURA_API_KEY --chain-id=$L1_CHAIN_ID --fork-block-number=15918000 --block-base-fee-per-gas=10 -s=$SNAPSHOT_FREQUENCY --state=./data/state --balance=1000000000000000000 &

echo "Waiting for ethereum host at $ETHEREUM_HOST..."
while ! curl -s $ETHEREUM_HOST >/dev/null; do sleep 1; done

# Fix anvil's fork timestamp
curl -s -H "Content-Type: application/json" -XPOST -d"{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"evm_setNextBlockTimestamp\",\"params\":[\"$(date +%s | xargs printf '0x%x')\"]}" $ETHEREUM_HOST > /dev/null
curl -s -H "Content-Type: application/json" -XPOST -d"{\"id\":2,\"jsonrpc\":\"2.0\",\"method\":\"evm_mine\",\"params\":[]}" $ETHEREUM_HOST > /dev/null

echo "Starting nginx..."
nginx &
wait
