---
title: SDK Overview
---

The Aztec SDK is the gateway for developers to access the Aztec network and benefit from low gas fees and privacy on Ethereum. The SDK connects to Ethereum and our zk-rollup service and can be integrated with a few lines of code.

The SDK is designed to abstract away the complexities of zero-knowledge proofs from the developer and end users. It provides a simple API for creating accounts, depositing and withdrawing tokens and interacting with Ethereum smart contracts anonymously. Core transfers inside the SDK are private by default.

Under the hood the SDK keeps track of a users private balances across multiples assets and provides easy to use helper methods to the developer to create seamless private UI's.

The SDK is written in Typescript and requires a Javascript context (web browser, web view or Node.js). This makes it well suited for developing web interfaces for maximum accessibility.

## What can you do with the SDK?

The Aztec SDK has many capabilities, many of which are associated with the following user actions:

- Connect to the Aztec network
- Add accounts to decrypt notes and show balances
- Register new accounts
- Handle signing keys to spend notes or withdraw assets to Ethereum
- Query Aztec transaction fees and specify settlement time
- Lookup user aliases associated with spending account to make transfering assets easier
- Migrate accounts to a new public key
- Recover accounts for which a spending key was lost
- Initiate interactions with smart contracts on Ethereum (i.e. defi deposits, token swaps, or unique interactions with custom [bridge contracts](https://github.com/AztecProtocol/aztec-connect-bridges))

## Controllers

Most of the above capabilities are possible through the use of `Controllers`. Controllers make it easy to prompt users for relevant transactions when an action requires an Ethereum transaction or signature and create proofs or send transactions to the Aztec network.

For example, the `RegisterController` is used for registering accounts, the `DepositController` for sending Ether or tokens to an Aztec account from an Ethereum account and the `TransferController` for spending notes within the Aztec network.

## Accounts

Aztec accounts may have two or more associated private keys. A new Aztec account should register a unique spending key before handling funds. Please review the [account overview page](../how-aztec-works/accounts) for a high level overview or dig into the code in the [Add Accounts to the SDK page](./usage/add-account).

## SDK Flavours

The SDK comes in 3 flavours. The users environment will typically automatically determine which flavour is used, but it is good to be aware of them because they do have tradeoffs.

In [zk.money](https://zk.money), the SDK is set up as `PLAIN`.

### Plain

`PLAIN` is used with a node backend or gets used in the iframe if shared workers are not supported in the execution context.

### Shared worker

An application can make the SDK available to users via `SHARED_WORKER` if the SDK at the `HOSTED` endpoint is unavailable.

This option is preferred over `PLAIN` because it allows similar work to be shared. For example, if you were using `PLAIN` in multiple tabs, each tab would be doing the same work. The `SHARED_WORKER` is more efficient in terms of networking and storage.

### Hosted

`HOSTED` does everything that the `SHARED_WORKER` does plus more, except in the case that your browser does not support shared workers. You can check shared worker browser compatibility [here](https://developer.mozilla.org/en-US/docs/Web/API/SharedWorker#browser_compatibility).

The `HOSTED` flavour is dependent on being able to access the SDK via a URL endpoint. The endpoint is `https://sdk.aztec.network`, where the Aztec team hosts an instance of the SDK.

When accessing the SDK via this method, users are trusting the provider at that URL with proper management of their Aztec keys within their browsers. The host also chooses which rollup provider the SDK talks to (ie mainnet vs a testnet).

This flavor needs more testing before it is considered production grade.
