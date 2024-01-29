---
title: Quickstart
---

In this guide, you will

1. Set up the Aztec sandbox (local development environment) locally
2. Install the Aztec development kit
3. Use the CLI to deploy an example contract that comes with the sandbox
4. Use the CLI to interact with the contract you just deployed

... in less than 10 minutes.

## Prerequisites

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))

## Install Docker

See [this page of the Docker docs](https://docs.docker.com/get-docker/) for instructions on how to install Docker Desktop for your operating system.

Once you have Docker installed, make sure it is running by opening the Docker Desktop application.

### Note on Linux

If you are running Linux, you will need to set the context (because Docker Desktop runs in a VM by default). See [this page](https://docs.docker.com/desktop/faqs/linuxfaqs/#what-is-the-difference-between-docker-desktop-for-linux-and-docker-engine) for more information. You can do this by running:

```bash
docker context use default
```

## Install the Sandbox

You can run the Sandbox using Docker.

To install the latest Sandbox version, run:

```bash
bash -i <(curl -s install.aztec.network)
```

This will install the following:

- **aztec** - launches various infrastructure subsystems (sequencer, prover, pxe, etc).
- **aztec-cli** - a command line tool for interfacing and experimenting with infrastructure.
- **aztec-nargo** - aztec's build of nargo, the noir compiler toolchain.
- **aztec-sandbox** - a wrapper around docker-compose that launches services needed for sandbox testing.
- **aztec-up** - a tool to upgrade the aztec toolchain to the latest, or specific versions.

Once these have been installed, to start the sandbox, run:

```bash
aztec-sandbox
```

This will attempt to run the Sandbox on ` localhost:8080`, so you will have to make sure nothing else is running on that port or change the port defined in `./.aztec/docker-compose.yml`. Running the command again will overwrite any changes made to the `docker-compose.yml`.

This command will also install the CLI if a node package version of the CLI isn't found locally.

## Deploy a contract using the CLI

The sandbox is preloaded with multiple accounts. Let's assign them to shell variables. Run the following in your terminal, so we can refer to the accounts as $ALICE and $BOB from now on:

:::note
The default accounts that come with sandbox will likely change over time. Save two of the "Initial accounts" that are printed in the terminal when you started the sandbox.
:::

#include_code declare-accounts yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Start by deploying a token contract. After it is deployed, we check that the deployment succeeded, and export the deployment address to use in future commands. For more detail on how the token contract works, see the [token contract tutorial](../tutorials/writing_token_contract.md).

#include_code deploy yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Note that the deployed contract address is exported, so we can use it as `$CONTRACT` later on.

## Call a contract with the CLI

Alice is set up as the contract admin and token minter in the `_initialize` function. Let's get Alice some private tokens.

We need to export the `SECRET` and `SECRET_HASH` values in order to privately mint tokens. Private tokens are claimable by anyone with the pre-image to a provided hash, see more about how the token contract works in the [token contract tutorial](../tutorials/writing_token_contract.md). After the tokens have been minted, the notes will have to added to the [Private Execution Environment](../../apis/pxe/interfaces/PXE.md) (PXE) to be consumed by private functions. Once added, Alice can claim them with the `redeem_shield` function. After this, Alice should have 1000 tokens in their private balance.

#include_code mint-private yarn-project/end-to-end/src/guides/up_quick_start.sh bash

We can have Alice privately transfer tokens to Bob. Only Alice and Bob will know what's happened. Here, we use Alice's private key to send a transaction to transfer tokens to Bob. Once they are transferred, we can verify that it worked as expected by checking Alice's and Bob's balances:

#include_code transfer yarn-project/end-to-end/src/guides/up_quick_start.sh bash

Alice and Bob should have 500 tokens.

Congratulations! You are all set up with the Aztec sandbox!

## What's next?

To start writing your first Aztec.nr smart contract, go to the [next page](aztecnr-getting-started.md).

You can also dig more into the sandbox and CLI [here](../cli/main.md).
