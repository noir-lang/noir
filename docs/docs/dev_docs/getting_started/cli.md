---
title: Aztec CLI
---

## Introduction

The Aztec CLI is a tool designed to enable a user to interact with the Aztec Network.

This tutorial will use the Aztec Sandbox so you should first set it up by following [the Aztec Sandbox instructions](../sandbox/main.md).

## Requirements

You should also have [node.js](https://nodejs.org/en/download) installed with version >= 18.

To install the Aztec CLI run:

```bash
npm install -g @aztec/cli
```

Or if you use yarn:

```bash
yarn global add @aztec/cli
```

Then verify that it is installed with:

```bash
aztec-cli -h
```

## I have the Sandbox running, now what?

Lets first establish that we are able to communicate with the Sandbox. Most commands will require the url to the Sandbox, which defaults in the CLI to `http://localhost:8080`. You can override this as an option with each command or by setting `AZTEC_RPC_HOST` environment variable.

To test communication with the Sandbox, let's run the command:

#include_code block-number yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

You should see the current block number (1) printed to the screen!

## Contracts

We have shipped a number of example contracts in the `@aztec/noir-contracts` npm package. This is included with the cli by default so you are able to use these contracts to test with. To get a list of the names of the contracts run:

#include_code example-contracts yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

You can see all of our example contracts in the monorepo [here](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/src/contracts).

In the following sections there will be commands that require contracts as options. You can either specify the full directory path to the contract abi, or you can use the name of one of these examples as the option value. This will become clearer later on.

## Creating Accounts

The first thing we want to do is create a couple of accounts. We will use the `create-account` command which will generate a new private key for us, register the account on the sandbox, and deploy a simple account contract which [uses a single key for privacy and authentication](../../concepts/foundation/accounts/keys.md):

#include_code create-account yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Once the account is set up, the CLI returns the resulting address, its privacy key, and partial address. You can read more about these [here](../../concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys).

Save the Address and Private key as environment variables. We will be using them later.

```bash
export ADDRESS=<Address printed when you run the command>
export PRIVATE_KEY=<Private key printed when you run the command>
```

Alternatively, we can also manually generate a private key and use it for creating the account, either via a `-k` option or by setting the `PRIVATE_KEY` environment variable.

#include_code create-account-from-private-key yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

For all commands that require a user's private key, the CLI will look for the `PRIVATE_KEY` environment variable in absence of an optional argument.

Let's double check that the accounts have been registered with the sandbox using the `get-accounts` command:

#include_code get-accounts yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Save one of the printed accounts (not the one that you generated above) in an environment variable. We will use it later.

```bash
export ADDRESS2=<Account address printed by the above command>
```

## Deploying a Token Contract

We will now deploy the private token contract using the `deploy` command, minting 1000000 initial tokens to address `0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c`. Make sure to replace this address with one of the two you created earlier.

#include_code deploy yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Save the contract address as an environment variable. We will use it later.

```bash
export CONTRACT_ADDRESS=<Your new contract address>
```

:::info
If you use a different address in the constructor above, you will get an error when running the deployment. This is because you need to register an account in the sandbox before it can receive private notes. When you create a new account, it gets automatically registered. Alternatively, you can register an account you do not own along with its public key using the `register-recipient` command.
:::

This command takes 1 mandatory positional argument which is the path to the contract ABI file in a JSON format (e.g. `contracts/target/PrivateToken.json`).
Alternatively you can pass the name of an example contract as exported by `@aztec/noir-contracts` (run `aztec-cli example-contracts` to see the full list of contracts available).

The command takes a few optional arguments while the most important one is:

- `--args` - Arguments to the constructor of the contract. In this case we have minted 1000000 initial tokens to the aztec address 0x20d3321707d53cebb168568e25c5c62a853ae1f0766d965e00d6f6c4eb05d599.

The CLI tells us that the contract was successfully deployed. We can use the `check-deploy` command to verify that a contract has been successfully deployed to that address:

#include_code check-deploy yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

## Calling a View Method

When we deployed the token contract, an initial supply of tokens was minted to the address provided in the constructor. We can now query the `getBalance()` method on the contract to retrieve the balance of that address. Make sure to replace the `contract-address` with the deployment address you got from the previous command, and the `args` with the account you used in the constructor.

#include_code call yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

The `call` command calls a read-only method on a contract, one that will not generate a transaction to be sent to the network. The arguments here are:

- `--args` - The address for which we want to retrieve the balance.
- `--contract-abi` - The abi of the contract we are calling.
- `--contract-address` - The address of the deployed contract

As you can see from the result, this address has a balance of 1000000, as expected.

## Sending a Transaction

We can now send a transaction to the network. We will transfer funds from the owner of the initial minted tokens to our other account. For this we will use the `send` command, which expects as arguments the quantity of tokens to be transferred, the sender's address, and the recipient's address. Make sure to replace all addresses in this command with the ones for your run.

#include_code send yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

We called the `transfer` function of the contract and provided these arguments:

- `--args` - The list of arguments to the function call.
- `--contract-abi` - The abi of the contract to call.
- `--contract-address` - The deployed address of the contract to call.
- `--private-key` - The private key of the sender

The command output tells us the details of the transaction such as its hash and status. We can use this hash to query the receipt of the transaction at a later time:

#include_code get-tx-receipt yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Let's now call `getBalance()` on each of our accounts and we should see updated values:

#include_code calls yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash
