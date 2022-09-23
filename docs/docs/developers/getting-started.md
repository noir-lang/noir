---
title: Getting Started
---

## Building on Aztec

There are two common ways that developers can start building on Aztec.

1. Build a user facing application that connects to the Aztec network via the Typescript SDK.
2. Build an Aztec Connect bridge that connects the Aztec network to Ethereum smart contracts.

We are also working on Noir, a domain specific language for creating and verifying proofs. There are some resources to learn more about this project at the bottom of the page.

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

The [overview page](../sdk/overview.md) of the SDK section has more information about using the SDK.

### Aztec SDK Resources

- [Aztec SDK on npm](https://www.npmjs.com/package/@aztec/sdk)
- [Aztec SDK on GitHub](https://github.com/AztecProtocol/aztec-connect/tree/master/sdk)
- [Getting Started with Aztec CLI / SDK](https://hackmd.io/NOtgWFSxS-Ko5mLlqt5GRw)
- [Aztec CLI](https://github.com/critesjosh/azteccli)
- [Testnet zk.money](https://aztec-connect-testnet.zk.money/)
- [Testnet block explorer](https://aztec-connect-testnet-explorer.aztec.network/)

## Building an Aztec Connect Bridge Contract

The [Aztec Connect bridges GitHub repository](https://github.com/AztecProtocol/aztec-connect-bridges) has the most up to date information about creating a bridge contract.

## Noir

Noir is a Domain Specific Language for SNARK proving systems. It can be used outside of Aztec or blockchain contexts. Noir will be used to create future versions of Aztec and as an integral part of the developer stack for building applications on Aztec.

It has been designed to use any ACIR compatible proving system. It's design choices are influenced heavily by Rust.

### Noir Resources

- [Official GitHub repo](https://github.com/noir-lang/noir)
- [The Noir Programming Language Book](https://noir-lang.github.io/book/index.html)
- [Getting Started with Noir Guide](https://hackmd.io/8jmyfuuTRWKr2w6rxr8HBw)