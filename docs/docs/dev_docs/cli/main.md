---
title: Aztec CLI
---

## Introduction

The Aztec CLI is a command-line tool allowing the user to interact directly with the Aztec Network.

It aims to provide all of the functionality required to deploy and invoke contracts and query system state such as contract data, transactions and emitted logs.

## Requirements

You should have [node.js](https://nodejs.org/en/download) installed with version >= 18.

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

Once installed it is invoked via:

`aztec-cli [options] [command]`

## I have the Sandbox running, now what?

Lets first establish that we are able to communicate with the Sandbox. Most commands will require the url to the Sandbox, which defaults in the CLI to `http://localhost:8080`. You can override this as an option with each command or by setting `PXE_URL` environment variable.

To test communication with the Sandbox, let's run the command:

#include_code block-number yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

You should see the current block number (1) printed to the screen!

## Contracts

We have shipped a number of example contracts in the `@aztec/noir-contracts` npm package. This is included with the cli by default so you are able to use these contracts to test with. To get a list of the names of the contracts run:

#include_code example-contracts yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

You can see all of our example contracts in the monorepo [here](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/src/contracts).

In the following sections there will be commands that require contracts as options. You can either specify the full directory path to the contract artifact, or you can use the name of one of these examples as the option value. This will become clearer later on.

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

You will see a that a number of accounts exist that we did not create. The Sandbox initializes itself with 3 default accounts. Save one of the printed accounts (not the one that you generated above) in an environment variable. We will use it later.

```bash
export ADDRESS2=<Account address printed by the above command>
```

## Deploying a Token Contract

We will now deploy a token contract using the `deploy` command, and set an address of the admin via a constructor argument.
Make sure to replace this address with one of the two you created earlier.

#include_code deploy yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Save the contract address as an environment variable. We will use it later.

```bash
export CONTRACT_ADDRESS=<Your new contract address>
```

- `--args` - Arguments to the constructor of the contract. In this case we have set an address as admin.

The CLI tells us that the contract was successfully deployed. We can use the `check-deploy` command to verify that a contract has been successfully deployed to that address:

#include_code check-deploy yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

## Sending a Transaction

We can now send a transaction to the network. We will mint funds in the public domain.
To form and submit the transaction we will use the `send` command of `aztec-cli`.
The `send` command expect the function name as the first unnamed argument and the following named arguments:

- `--args` - The list of arguments to the function call.
- `--contract-artifact` - The artifact of the contract to call.
- `--contract-address` - The deployed address of the contract to call.
- `--private-key` - The private key of the sender.

#include_code send yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

We called the `mint_public` function and provided it with the 2 arguments it expects: the recipient's address and the amount to be minted. Make sure to replace all addresses in this command with yours.

The command output tells us the details of the transaction such as its hash and status. We can use this hash to query the receipt of the transaction at a later time:

#include_code get-tx-receipt yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

## Calling an Unconstrained (View) Function

Now that the `mint_public` tx has been settled we can call the `balance_of_public` unconstrained function:

#include_code call yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

The `call` command calls a read-only method on a contract, one that will not generate a transaction to be sent to the network. The arguments here are:

- `--args` - The address for which we want to retrieve the balance.
- `--contract-artifact` - The artifact of the contract we are calling.
- `--contract-address` - The address of the deployed contract

As you can see from the result, this address has a public balance of 543, as expected.

## Compute Function Selector
`aztec-cli --compute-selector <signature e.g. foo(Field,Field)>` gives the function selector.  

## Inspect Contract
`aztec-cli --compute-selector <json artifact file e.g. artifacts/token_contract.json>` gives the list of all callable functions along with their function signature and selector.