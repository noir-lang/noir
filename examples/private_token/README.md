# Private Token with Noir ZK Proofs

A privacy-preserving token system on Ethereum Sepolia using Zero-Knowledge Proofs built with Noir.

## Features

- **Hidden Balances**: Token balances are never revealed on-chain
- **Private Addresses**: Sender and recipient addresses are hidden using commitments
- **Anonymous Transactions**: Transaction amounts and parties are private
- **Double-Spend Protection**: Nullifiers prevent reuse of commitments
- **EVM Compatible**: Runs on Ethereum Sepolia testnet

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Rust Client   │────▶│  Noir Circuits  │────▶│   ZK Proof      │
│   (private)     │     │  (prove)        │     │   (public)      │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Local State    │     │  Sepolia Chain  │◀────│  Verifier       │
│  (private)      │     │  (public)       │     │  Contract       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Privacy Model

1. **Commitments**: Each UTXO is represented as `Hash(address, balance, nonce)`
2. **Nullifiers**: To spend, publish `Hash(secret, nonce)` - prevents double-spend
3. **ZK Proofs**: Prove ownership and valid computation without revealing values

## Project Structure

```
noir_tests/
├── circuits/                    # Noir ZK circuits
│   ├── private_transfer/       # Transfer proof circuit
│   │   ├── src/main.nr
│   │   ├── Nargo.toml
│   │   └── Prover.toml
│   └── mint/                   # Mint proof circuit
│       ├── src/main.nr
│       ├── Nargo.toml
│       └── Prover.toml
├── contracts/                  # Solidity smart contracts
│   ├── src/
│   │   ├── PrivateToken.sol   # Main token contract
│   │   └── UltraVerifier.sol  # Placeholder verifier
│   ├── script/
│   │   └── Deploy.s.sol       # Deployment script
│   ├── test/
│   │   └── PrivateToken.t.sol # Contract tests
│   └── foundry.toml
├── client/                     # Rust CLI client
│   ├── src/
│   │   ├── main.rs            # CLI entry point
│   │   ├── lib.rs             # Library exports
│   │   ├── crypto.rs          # Cryptographic utilities
│   │   ├── state.rs           # Local state management
│   │   ├── prover.rs          # Proof generation
│   │   ├── contract.rs        # Contract interaction
│   │   └── error.rs           # Error types
│   └── Cargo.toml
├── .env.example
├── .gitignore
└── README.md
```

## Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Noir** (0.29+): `curl -L https://install.noire.rs | bash && noirup`
- **Foundry**: `curl -L https://foundry.paradigm.xyz | bash && foundryup`
- **Node.js** (optional, for additional tooling)

## Quick Start

### 1. Compile Noir Circuits

```bash
# Compile transfer circuit
cd circuits/private_transfer
nargo compile

# Compile mint circuit
cd ../mint
nargo compile
```

### 2. Generate Solidity Verifiers

```bash
# Generate verifier for transfer circuit
cd circuits/private_transfer
nargo codegen-verifier

# Generate verifier for mint circuit
cd ../mint
nargo codegen-verifier
```

Copy the generated verifiers to `contracts/src/`.

### 3. Deploy Contracts

```bash
cd contracts

# Install dependencies
forge install foundry-rs/forge-std

# Build contracts
forge build

# Run tests
forge test

# Deploy to Sepolia
cp ../.env.example ../.env
# Edit .env with your keys
source ../.env
forge script script/Deploy.s.sol --rpc-url $SEPOLIA_RPC_URL --broadcast --verify
```

### 4. Build and Run the Rust Client

```bash
cd client

# Build
cargo build --release

# Run CLI
cargo run --release -- --help
```

## Usage

### Create a New Account

```bash
cargo run --release -- new-account
```

Output:
```
✅ New account created!
   Address: 0x1234...
   Secret:  0xabcd...

⚠️  IMPORTANT: Save your secret key securely!
```

### Mint Tokens

```bash
cargo run --release -- mint --secret 0xYOUR_SECRET --amount 100
```

### Transfer Tokens

```bash
cargo run --release -- transfer \
    --from-secret 0xSENDER_SECRET \
    --to-address 0xRECIPIENT_ADDRESS \
    --amount 25
```

### Check Balance

```bash
cargo run --release -- balance --address 0xYOUR_ADDRESS
```

### List All Accounts

```bash
cargo run --release -- accounts
```

## How It Works

### Minting

1. User generates a secret key locally
2. Address is derived: `address = Hash(secret)`
3. Commitment is created: `commitment = Hash(address, amount, nonce)`
4. ZK proof proves:
   - Commitment is correctly formed
   - Amount is positive
5. On-chain: commitment is added to the set

### Transferring

1. Find an unspent commitment with sufficient balance
2. Compute nullifier: `nullifier = Hash(secret, nonce)`
3. Create output commitments for sender (change) and recipient
4. ZK proof proves:
   - Sender knows the secret for the input commitment
   - Input commitment exists
   - Balance is sufficient
   - Output commitments are correctly formed
5. On-chain:
   - Nullifier is recorded (prevents double-spend)
   - New commitments are added
   - No amounts or addresses are revealed

## Security Considerations

⚠️ **This is a demo project for educational purposes.**

- **Secret Management**: Keep secret keys secure. Loss = loss of funds.
- **State Backup**: Back up `private_state.json` regularly.
- **Pedersen Hash**: The demo uses SHA256 as a placeholder. Use actual Pedersen hash in production.
- **Audit Required**: Do not use on mainnet without thorough security audits.
- **Verifier Placeholder**: The included verifier always returns true. Replace with generated verifier.

## Testing

### Noir Circuit Tests

```bash
cd circuits/private_transfer
nargo test

cd ../mint
nargo test
```

### Solidity Contract Tests

```bash
cd contracts
forge test -vvv
```

### Rust Client Tests

```bash
cd client
cargo test
```

## Roadmap

- [ ] Integrate actual Pedersen hash from Barretenberg
- [ ] Implement proper proof generation using noir_rs
- [ ] Add Merkle tree for commitment management
- [ ] Support for multiple denominations
- [ ] Withdrawal/unwrap functionality
- [ ] Web interface
- [ ] Hardware wallet support

## Resources

- [Noir Documentation](https://noir-lang.org/docs)
- [Foundry Book](https://book.getfoundry.sh/)
- [Alloy Documentation](https://alloy-rs.github.io/alloy/)
- [Zero-Knowledge Proofs MOOC](https://zk-learning.org/)

## License

MIT
