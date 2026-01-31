#!/bin/bash

set -e

echo "🔧 Setting up Private Token project..."
echo ""

# Check for Noir
if ! command -v nargo &> /dev/null; then
    echo "📦 Installing Noir..."
    curl -L https://install.noire.rs | bash
    source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null || true
    noirup
else
    echo "✅ Noir already installed: $(nargo --version)"
fi

# Check for Foundry
if ! command -v forge &> /dev/null; then
    echo "📦 Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    source ~/.zshrc 2>/dev/null || source ~/.bashrc 2>/dev/null || true
    foundryup
else
    echo "✅ Foundry already installed: $(forge --version)"
fi

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    echo "✅ Rust already installed: $(rustc --version)"
fi

echo ""
echo "📁 Compiling Noir circuits..."

# Compile circuits
cd circuits/private_transfer
echo "   Compiling private_transfer..."
nargo compile || echo "   ⚠️  Compilation failed - check circuit code"

cd ../mint
echo "   Compiling mint..."
nargo compile || echo "   ⚠️  Compilation failed - check circuit code"

cd ../..

echo ""
echo "🔨 Building Solidity contracts..."

cd contracts

# Install forge dependencies
forge install foundry-rs/forge-std --no-commit 2>/dev/null || true

# Build contracts
forge build || echo "   ⚠️  Build failed - check contract code"

# Run tests
echo ""
echo "🧪 Running contract tests..."
forge test || echo "   ⚠️  Some tests failed"

cd ..

echo ""
echo "🦀 Building Rust client..."

cd client
cargo build --release || echo "   ⚠️  Build failed - check client code"

cd ..

echo ""
echo "✅ Setup complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Copy .env.example to .env and configure your keys"
echo "   2. Generate verifiers: cd circuits/mint && nargo codegen-verifier"
echo "   3. Deploy contracts to Sepolia"
echo "   4. Update CONTRACT_ADDRESS in .env"
echo "   5. Run the client: cd client && cargo run --release -- --help"
echo ""
