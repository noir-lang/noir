#!/bin/bash

set -e

echo "🚀 Deploying Private Token to Sepolia..."

# Load environment
if [ -f .env ]; then
    source .env
else
    echo "❌ .env file not found. Copy .env.example to .env and configure it."
    exit 1
fi

# Check required variables
if [ -z "$SEPOLIA_RPC_URL" ] || [ -z "$PRIVATE_KEY" ]; then
    echo "❌ Missing SEPOLIA_RPC_URL or PRIVATE_KEY in .env"
    exit 1
fi

cd contracts

echo ""
echo "📦 Building contracts..."
forge build

echo ""
echo "🔄 Deploying..."
forge script script/Deploy.s.sol \
    --rpc-url $SEPOLIA_RPC_URL \
    --broadcast \
    --verify \
    -vvv

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📋 Update your .env file with the deployed contract addresses."
echo ""
