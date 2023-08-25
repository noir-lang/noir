# Architecture

Wallets expose to dapps an interface that allows them to act on behalf of the user, such as querying private state or sending transactions. Bear mind that, as in Ethereum, wallets should require user confirmation whenever carrying out a potentially sensitive action requested by a dapp.

## Overview

Architecture-wise, a wallet is an instance of an **Aztec RPC Server** which manages user keys and private state. The RPC server also communicates with an **Aztec Node** for retrieving public information or broadcasting transactions. Note that the RPC server requires a local database for keeping private state, and is also expected to be continuously syncing new blocks for trial-decryption of user notes.

Additionally, a wallet must be able to handle one or more [account contract implementations](../../concepts/foundation/accounts/main.md#account-contracts-and-wallets). When a user creates a new account, the account is represented on-chain by an account contract. The wallet is responsible for deploying and interacting with this contract. A wallet may support multiple flavours of accounts, such as an account that uses ECDSA signatures, or one that relies on WebAuthn, or one that requires multi-factor authentication. For a user, the choice of what account implementation to use is then determined by the wallet they interact with.

In code, this translates to a wallet implementing an **Entrypoint** interface that defines [how to create an _execution request_ out of an array of _function calls_](./main.md#transaction-lifecycle) for the specific implementation of an account contract. Think of the entrypoint interface as the Javascript counterpart of an account contract, or the piece of code that knows how to format and authenticate a transaction based on the rules defined in Noir by the user's account.

## Entrypoint interface

The entrypoint interface is used for creating an _execution request_ out of one or more _function calls_ requested by a dapp. Account contracts are expected to handle multiple function calls per transaction, since dapps may choose to batch multiple actions into a single request to the wallet.

#include_code entrypoint-interface /yarn-project/aztec.js/src/account/entrypoint/index.ts typescript

Refer to the page on [writing an account contract](./writing_an_account_contract.md) for an example on how to implement this interface.

## RPC interface

A wallet exposes the RPC interface to dapps by running an [Aztec RPC Server instance](https://github.com/AztecProtocol/aztec-packages/blob/95d1350b23b6205ff2a7d3de41a37e0bc9ee7640/yarn-project/aztec-rpc/src/aztec_rpc_server/aztec_rpc_server.ts). The Aztec RPC Server requires a keystore and a database implementation for storing keys, private state, and recipient encryption public keys.

#include_code rpc-interface /yarn-project/types/src/interfaces/aztec_rpc.ts typescript





