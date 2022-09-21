---
title: Glossary
---

### Account

An Aztec account is the user primitive on the network. An account is identified by a public key or an alias and controlled by an account key and optional spending keys. An account key can decrypt value notes or register an account 1 time. See the [Accounts](how-aztec-works/accounts.md) page for more information.

### Account Key

The private key associated with an account can be used to decrypt notes. The private key can also be used to register a distinct spending keys 1 time. See the [Accounts](how-aztec-works/accounts.md) page for more information.

### Account Migration

Used when a user loses access to their [Account Keys](#account-key). This allows a user to keep their alias while setting new account key, spending key and recovery key.

**Accounts can only be migrated 1 time.**

### Account Note

The accounts registered by users on Aztec are represented by account notes. An account note associates spending keys and aliases with an account. The spending key is used to sign transactions.

### Account Recovery

Used when a user loses access to all of their registered [Spending keys](#spending-key).

### Asset Ids

Asset Ids are unique numbers that correspond to various assets in Aztec.

| Asset | Id |
| --- | --- |
| ETH | 0 |
| DAI | 1 |
| wstETH | 2 |

### Barretenberg

Aztec's cryptography back-end. Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Falafel

The Aztec client. See [Sequencer](#sequencer) for more info.

Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Halloumi

Aztec's Proof creation service. Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Rollup Processor Contract

This is the smart contract on Ethereum that holds user deposits, facilitates interactions with other Ethereum contracts from Aztec and processes Aztec rollup blocks. You can find the contract on Etherscan [here](https://etherscan.io/address/0xff1f2b4adb9df6fc8eafecdcbf96a2b351680455).

### Sequencer

This is also called the Rollup Processor.

This service is responsible for:

- Watching for rollup blocks on Ethereum and updating the representation of Aztec state accordingly
- Listening for and storing transactions from users, verifying they're valid, have correct fees, etc.
- Constructing new rollups at the appropriate time or when enough transactions are received
- Publishing of rollups to an Ethereum chain

You can find the Typescript reference implementation called Falafel [here](https://github.com/AztecProtocol/aztec-connect/tree/master/falafel).

Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Spending Key

A specific private key registered to an account with permission to spend asset notes on behalf of that account. See the [Accounts](how-aztec-works/accounts.md) page for more information.

### Signing Key

See [Spending Key](#spending-key).

### Value Notes

Asset notes (or value notes) are representations of asset in Aztec. They are sent around the network via transactions.
