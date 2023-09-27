---
title: Quickstart
---

Get started with the Aztec Sandbox.

## Introduction

The Aztec Sandbox is an environment for local development on the Aztec Network. It's easy to get setup with just a single, simple command, and contains all the components needed to develop and test Aztec contracts and applications.

This is a 1 page introduction to getting started with running the sandbox, and interacting with it via the CLI. We will go over how to deploy a token contract to the sandbox, mint tokens and transfer them between accounts. You will find more in depth information on the following pages in this Getting Started section.

### Background

Aztec's Layer 2 network is a fully programmable combined private/public ZK rollup. To achieve this, the network contains the following primary components:

- Aztec Node - Aggregates all of the 'backend' services necessary for the building and publishing of rollups. This packages is currently in development and much of the functionality is mocked.
- [Private Execution Environment (PXE)](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/pxe) - Normally residing with the end client, this decrypts and stores a client's private state, executes simulations and submits transactions to the Aztec Node.
- [Aztec.js](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec.js) - Aztec's client library for interacting with the PXE (think Ethers.js). See the getting started guide [here](./sandbox.md).

All of this is included in the Sandbox, with the exception of Aztec.js which you can use to interact with it.

With the help of Aztec.js you will be able to:

- Create an account
- Deploy a contract
- Call view methods on contracts
- Simulate the calling of contract functions
- Send transactions to the network
- Be notified when transactions settle
- Query chain state such as chain id, block number etc.

This quickstart walks you through installing the Sandbox, deploying your first Noir contract, and verifying its execution!

## Requirements

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))
- Docker and Docker Compose (Docker Desktop under WSL2 on windows)

## Installation

You can run the Sandbox using either Docker or npm.

### With Docker

To install and start the Sandbox paste the line below in a macOS Terminal or Linux shell prompt. You will need to have Docker installed and running on your machine.

```bash
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

This will attempt to run the Sandbox on localhost:8080, so you will have to make sure nothing else is running on that port or change the port defined in `./.aztec/docker-compose.yml`. Running the command again will overwrite any changes made to the docker-compose.yml.

To install a specific version of the sandbox, you can set the environment variable `SANDBOX_VERSION`

```bash
SANDBOX_VERSION=<version> /bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

NOTE: If `SANDBOX_VERSION` is not defined, the script will pull the latest release of the sandbox.

### With npm

You can download and run the Sandbox package directly if you have nodejs 18 or higher installed.

You will also need an Ethereum node like Anvil or Hardhat running locally on port 8545.

```bash
npx @aztec/aztec-sandbox
```

### CLI

To interact with the sandbox now that it's running locally, install the [Aztec CLI](https://www.npmjs.com/package/@aztec/cli):

```bash
npm install -g @aztec/cli
```

## Deploying a contract

The sandbox is preloaded with multiple accounts. Let's assign them to shell variables. Run the following in your terminal, so we can refer to the accounts as $ALICE and $BOB from now on:

:::note
The default accounts that come with sandbox will likely change over time. Save two of the "Initial accounts" that are printed in the terminal when you started the sandbox.
:::

#include_code declare-accounts yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Start by deploying a token contract. After it is deployed, we check that the deployment succeeded, export the deployment address to use in future commands and then call the `_initialize` function. For more detail on how the token contract works, see the [token contract tutorial](../tutorials/writing_token_contract.md).

#include_code deploy yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Note that the deployed contract address is exported, so we can use it as `$CONTRACT` later on.

## Calling a contract

Alice is set up as the contract admin and token minter in the `_initialize` function. Let's get Alice some private tokens.

We need to export the `SECRET` and `SECRET_HASH` values in order to privately mint tokens. Private tokens are claimable by anyone with the pre-image to a provided hash, see more about how the token contract works in the [token contract tutorial](../tutorials/writing_token_contract.md). Once the tokens have been minted, Alice can claim them with the `redeem_shield` function. After this, Alice should have 1000 tokens in their private balance.

#include_code mint-private yarn-project/end-to-end/src/guides/up_quick_start.sh bash

We can have Alice privately transfer tokens to Bob. Only Alice and Bob will know what's happened. Here, we use Alice's private key to send a transaction to transfer tokens to Bob. Once they are transferred, we can verify that it worked as expected by checking Alice's and Bob's balances:

#include_code transfer yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Alice and Bob should have 500 tokens.

Congratulations! You are all set up with the Aztec sandbox!

## Great, but what can I do with it?

Aztec's Layer 2 network is a fully programmable combined private/public ZK rollup. To achieve this, the network contains the following primary components:

- Aztec Node - Aggregates all of the 'backend' services necessary for the building and publishing of rollups.
- Private Execution Environment (PXE) - Normally residing with the end client, this decrypts and stores a client's private state, executes simulations and submits transactions to the Aztec Node.
- [Aztec.js](./sandbox) - Aztec's client library for interacting with the PXE (think Ethers.js).
- [Aztec.nr](../contracts/main.md) - Aztec's smart contract framework

