---
title: CLI Commands
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

Here you will find a reference to the commands available in the Aztec CLI.

## Installation

### Docker

The CLI will be installed automatically via Docker by running the command to install and start the sandbox, [instructions here](./sandbox-reference.md#with-docker).

## Update

The CLI comes with an update command.

```bash
aztec-cli update . --contract src/contract1 --contract src/contract2
```

This command does a few things to manage updates:

- It looks for a `package.json` and updates all `@aztec/` dependencies to the versions the sandbox expects.
- It looks for `Nargo.toml` at the `--contract` paths specified and updates all `aztec.nr` dependencies to the versions the sandbox expects.
- It outputs the changes.

You can specify a version to update to with the `--aztec-version` flag, but it defaults to `latest` so this is typically not necessary.

:::info

The update command won't update the CLI itself. To update these follow the [updating instructions which point to our curl command](./sandbox-reference.md#with-docker).

:::

## Compile

You can find more information about compiling contracts [on this page](../contracts/compiling.md).

## Creating Accounts

The first thing we want to do is create a couple of accounts. We will use the `create-account` command which will generate a new private key for us, register the account on the sandbox, and deploy a simple account contract which [uses a single key for privacy and authentication](../../learn/concepts/accounts/keys.md):

#include_code create-account yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Once the account is set up, the CLI returns the resulting address, its privacy key, and partial address. You can read more about these [here](../../learn/concepts/accounts/keys.md#addresses-partial-addresses-and-public-keys).

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

You will see that a number of accounts exist that we did not create. The Sandbox initializes itself with 3 default accounts. Save one of the printed accounts (not the one that you generated above) in an environment variable. We will use it later.

```bash
export ADDRESS2=<Account address printed by the above command>
```

## Deploying a Token Contract

We will now deploy a token contract using the `deploy` command, and set an address of the admin via a constructor argument. You can find the contract we are deploying [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/contracts/token_contract/src/main.nr) (or write it for yourself in [this tutorial!](../tutorials/writing_token_contract.md))
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

We called the [`mint_public`](https://github.com/AztecProtocol/aztec-packages/blob/87fa621347e55f82e36c70515c1824161eee5282/yarn-project/noir-contracts/contracts/token_contract/src/main.nr#L157C10-L157C10) function and provided it with the 2 arguments it expects: the recipient's address and the amount to be minted. Make sure to replace all addresses in this command with yours.

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
