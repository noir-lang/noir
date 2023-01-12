---
title: Glossary
---

### Account

An Aztec account is the user primitive on the network. See the [Accounts](how-aztec-works/accounts.mdx) page for more information.

### Account Key

See [Viewing Key](#viewing-key).

### Account Migration

Used when a user loses access to their [Account Keys](#account-key). This allows a user to keep their alias while setting new account key, spending key and recovery key.

**Accounts can only be migrated 1 time.**

### Account Note

The accounts registered by users on Aztec are represented by account notes. An account note associates spending keys and aliases with an account. The spending key is used to sign transactions.

### Account Recovery

Used when a user loses access to all of their registered [Spending keys](#spending-key). Note that a user must have the [viewing key](#viewing-key) in order to recover an account.

Read more about acccount recovery in the SDK docs [here](./sdk/usage/account-recovery.md).

### Account Registration

Registering an account on Aztec associates the account public key with an alias, a spending key and an optional recovery key. A recovery key must be added at registration in order to take advantage of [account recovery](#account-recovery).

Read more about account registration on the [accounts page](how-aztec-works/accounts.mdx#account-registration) and in the SDK docs [here](./sdk/usage/register.md).

### Alias

The account public key is associated with a human-readable alias when the account registers a new signing key (see below). The alias can be anything (20 alphanumeric, lowercase characters or less) as long as it hasn't been claimed yet.

### Asset Ids

Asset Ids are unique numbers that correspond to various assets in Aztec.

| Asset | Id |
| --- | --- |
| ETH | 0 |
| DAI | 1 |
| wstETH | 2 |
| yvDAI | 3 |
| yvWETH | 4 |
| weWETH | 5 |
| wewstETH | 6 |
| weDAI | 7 |
| wa2DAI | 8 |
| wa2WETH | 9 |
 
### Barretenberg

Aztec's cryptography back-end. Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Falafel

The Aztec client. See [Sequencer](#sequencer) for more info.

Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Halloumi

Aztec's Proof creation service. Refer to the graphic at the top of [this page](https://medium.com/aztec-protocol/explaining-the-network-in-aztec-network-166862b3ef7d) to see how it fits in the Aztec architecture.

### Privacy Key

See [Viewing Key](#viewing-key).

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

A specific private key registered to an account with permission to spend asset notes on behalf of that account. See the [Accounts](how-aztec-works/accounts.mdx) page for more information.

### Signing Key

See [Spending Key](#spending-key).

### Value Notes

Asset notes (or value notes) are representations of asset in Aztec. They are sent around the network via transactions.

### Viewing Key

Also called the Account key, the privacy key or the decryption key.

This is the private key that is associated with plain (unregistered) Aztec account. This key is used to decrypt notes associated with the account. For an unregistered Aztec account, it is also used to spend notes. It can be used to [register](#account-registration) an account 1 time.

See the [Accounts](how-aztec-works/accounts.mdx) page for more information.
