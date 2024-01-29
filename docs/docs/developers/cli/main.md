---
title: Sandbox and CLI
---

The Aztec Sandbox is an environment for local development on the Aztec Network. It's easy to get setup with just a single, simple command, and contains all the components needed to develop and test Aztec contracts and applications. The Aztec CLI will allow you to interact with the sandbox.

## Components of the Aztec network

Aztec's Layer 2 network is a fully programmable combined private/public ZK rollup. To achieve this, the network contains the following primary components:

- Aztec Node - Aggregates all of the 'backend' services necessary for the building and publishing of rollups. This package is currently in development and much of the functionality is mocked.
- [Private Execution Environment (PXE)](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/pxe) - Normally residing with the end client, this decrypts and stores a client's private state, executes simulations and submits transactions to the Aztec Node.
- [Aztec.js](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec.js) - Aztec's client library for interacting with the PXE (think Ethers.js). See the getting started guide [here](../getting_started/aztecjs-getting-started.md).

All of this is included in the Sandbox, with the exception of Aztec.js which you can use to interact with it.

With the help of Aztec.js you will be able to:

- Create an account
- Deploy a contract
- Call view methods on contracts
- Simulate the calling of contract functions
- Send transactions to the network
- Be notified when transactions settle
- Query chain state such as chain id, block number etc.

## What's in the Sandbox?

The sandbox contains a local Ethereum instance running [Anvil](https://book.getfoundry.sh/anvil/), a local instance of the Aztec rollup and an aztec private execution client for handling user transactions and state.

These provide a self contained environment which deploys Aztec on a local (empty) Ethereum network, creates 3 smart contract wallet accounts on the rollup, and allows transactions to be processed on the local Aztec sequencer.

The current sandbox does not generate or verify proofs, but provides a working end to end developer flow for writing and interacting with Aztec.nr smart contracts.

## Aztec CLI and aztec-nargo

The Aztec CLI is a command-line tool allowing you to interact directly with the Aztec network and sandbox.

It aims to provide all of the functionality required to deploy, and invoke contracts and query system state such as contract data, transactions and emitted logs.

Use `aztec-nargo` for compiling contracts. See the [compiling contracts](../contracts/compiling.md) page for more information.
