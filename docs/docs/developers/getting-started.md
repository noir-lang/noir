---
title: Getting Started
---

## Building on Aztec

There are two common ways that developers may want to start building on Aztec.

1. Build a user facing application that connects to the Aztec network via the Typescript SDK.
2. Build an Aztec Connect bridge that connects the Aztec network to existing Ethereum smart contracts.

## Building with the SDK

The fastest way to get started developing on Aztec is using the public mainnet fork testnet.

1. Connect Metamask (or other Ethereum wallet) to the testnet
   1. Chain ID: `677868`
   2. RPC URL: `https://mainnet-fork.aztec.network`
2. Get testnet funds
   1. Ping [@critesjosh_](https://twitter.com/critesjosh_) for testnet ETH. We will have a public faucet published soon.
3. Install the SDK in your project.
   1. `npm install @aztec/sdk`
4. Interact with Aztec (optional, useful for cross-referencing)
   1. Use the testnet version of zk.money (https://aztec-connect-testnet.zk.money/)
   2. Use the `azteccli` command line tool. https://github.com/critesjosh/azteccli

Once you have testnet ETH and setup the SDK, you can start interacting with the Aztec network. You can start registering accounts, making deposits, doing transfers and withdrawals and other things supported by the [SDK](../sdk/overview.md).

You can see how to set up the SDK on the [setup page](../sdk/usage/setup.mdx) or in the command line repo (https://github.com/critesjosh/azteccli/blob/main/src/base.ts).

The [testnet block explorer](https://aztec-connect-testnet-explorer.aztec.network/) will be useful as you develop your application.

The [overview page](../sdk/overview.md) of the SDK section has more information about using the SDK.

## Building an Aztec Connect Bridge Contract

The [Aztec Connect bridges GitHub repository](https://github.com/AztecProtocol/aztec-connect-bridges) has the most up to date information about creating a bridge contract.
