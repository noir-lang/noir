# Architecture

Wallets expose to dapps an interface that allows them to act on behalf of the user, such as querying private state or sending transactions. Bear mind that, as in Ethereum, wallets should require user confirmation whenever carrying out a potentially sensitive action requested by a dapp.

## Overview

Architecture-wise, a wallet is an instance of an **Private Execution Environment (PXE)** which manages user keys and private state.
The PXE also communicates with an **Aztec Node** for retrieving public information or broadcasting transactions.
Note that the PXE requires a local database for keeping private state, and is also expected to be continuously syncing new blocks for trial-decryption of user notes.

Additionally, a wallet must be able to handle one or more [account contract implementations](../../learn/concepts/accounts/main.md#account-contracts-and-wallets). When a user creates a new account, the account is represented on-chain by an account contract. The wallet is responsible for deploying and interacting with this contract. A wallet may support multiple flavours of accounts, such as an account that uses ECDSA signatures, or one that relies on WebAuthn, or one that requires multi-factor authentication. For a user, the choice of what account implementation to use is then determined by the wallet they interact with.

In code, this translates to a wallet implementing an **AccountInterface** interface that defines [how to create an _execution request_ out of an array of _function calls_](./main.md#transaction-lifecycle) for the specific implementation of an account contract and [how to generate an _auth witness_](./main.md#authorizing-actions) for authorizing actions on behalf of the user. Think of this interface as the Javascript counterpart of an account contract, or the piece of code that knows how to format a transaction and authenticate an action based on the rules defined by the user's account contract implementation.

## Account interface

The account interface is used for creating an _execution request_ out of one or more _function calls_ requested by a dapp, as well as creating an _auth witness_ for a given message hash. Account contracts are expected to handle multiple function calls per transaction, since dapps may choose to batch multiple actions into a single request to the wallet.

#include_code account-interface yarn-project/aztec.js/src/account/interface.ts typescript

Refer to the page on [writing an account contract](./writing_an_account_contract.md) for an example on how to implement this interface.

## PXE interface

A wallet exposes the PXE interface to dapps by running an PXE instance. The PXE requires a keystore and a database implementation for storing keys, private state, and recipient encryption public keys.

#include_code pxe-interface /yarn-project/circuit-types/src/interfaces/pxe.ts typescript





