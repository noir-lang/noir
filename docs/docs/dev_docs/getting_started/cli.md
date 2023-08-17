---
title: Aztec CLI
---

## Introduction

The Aztec CLI is a tool designed to enable a user to interact with the Aztec Network. Here we will provide a walk-through demonstrating how to use the CLI to deploy and test contracts on the [Aztec Sandbox](../sandbox/main.md).

## Requirements

The Aztec CLI is an npm package so you will need to have installed node.js >= 18. This tutorial will use the Aztec Sandbox so you should first set up the sandbox using the link above.

To install the Aztec CLI run:

`npm install -g @aztec/cli`

Or if you use yarn:

`yarn global add @aztec/cli`

Then verify that it is installed with:

`aztec-cli -h`

## I have the Sandbox running, now what?

Lets first establish that we are able to communicate with the Sandbox. Most commands will require the url to the Sandbox, which defaults in the CLI to `http://localhost:8080`. You can override this as an option with each command or by setting `AZTEC_RPC_HOST` environment variable.

To test communication with the Sandbox, let's run the command:

`% aztec-cli block-number
0`

You should see the current block number (0) printed to the screen!

## Contracts

We have shipped a number of example contracts in the `@aztec/noir-contracts` npm package. This is included with the cli by default so you are able to use these contracts to test with. To get a list of the names of the contracts run:

```
% aztec-cli example-contracts
ChildContractAbi
EasyPrivateTokenContractAbi
EcdsaAccountContractAbi
EscrowContractAbi
LendingContractAbi
NonNativeTokenContractAbi
ParentContractAbi
PendingCommitmentsContractAbi
PokeableTokenContractAbi
PrivateTokenAirdropContractAbi
PrivateTokenContractAbi
PublicTokenContractAbi
SchnorrAccountContractAbi
SchnorrSingleKeyAccountContractAbi
TestContractAbi
UniswapContractAbi
```

In the following sections there will be commands that require contracts as options. You can either specify the full directory path to the contract abi, or you can use the name of one of these examples as the option value. This will become clearer later on.

## Creating Accounts

The first thing we want to do is create a couple of accounts. We will use the `create-account` command which will generate a new private key for us, register the account on the sandbox, and deploy a simple account contract which [uses a single key for privacy and authentication](../../concepts/foundation/accounts/keys.md):

```
% aztec-cli create-account
Created new account:

Address:         0x20d3321707d53cebb168568e25c5c62a853ae1f0766d965e00d6f6c4eb05d599
Public key:      0x02d18745eadddd496be95274367ee2cbf0bf667b81373fb6bed715c18814a09022907c273ec1c469fcc678738bd8efc3e9053fe1acbb11fa32da0d6881a1370e
Private key:     0x2aba9e7de7075deee3e3f4ad1e47749f985f0f72543ed91063cc97a40d851f1e
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
```

Once the account is set up, the CLI returns the resulting address, its privacy key, and partial address. You can read more about these [here](../../concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys).

Alternatively, we can also manually generate a private key and use it for creating the account, either via a `-k` option or by setting the `PRIVATE_KEY` environment variable.
 
```
% aztec-cli generate-private-key

Private Key: 0x6622c828e9cd5adc86f10878765fe921d2b8cb2c79bdbc391157e43811ce88e3
Public Key: 0x08aad54f32f1b6621ee5f25267166e160147cd355a2dfc129fa646a651dd29471d814ac749c2cda831fcca361c830ba56db4b4bd5951d4953c81865d0ae0cbe7


% aztec-cli create-account --private-key 0x6622c828e9cd5adc86f10878765fe921d2b8cb2c79bdbc391157e43811ce88e3

Created new account:

Address:         0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c
Public key:      0x08aad54f32f1b6621ee5f25267166e160147cd355a2dfc129fa646a651dd29471d814ac749c2cda831fcca361c830ba56db4b4bd5951d4953c81865d0ae0cbe7
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
```

For all commands that require a user's private key, the CLI will look for the `PRIVATE_KEY` environment variable in absence of an optional argument.

Let's double check that the accounts have been registered with the sandbox using the `get-accounts` command:

```
% aztec-cli get-accounts
Accounts found:

Address:         0x20d3321707d53cebb168568e25c5c62a853ae1f0766d965e00d6f6c4eb05d599
Public key:      0x02d18745eadddd496be95274367ee2cbf0bf667b81373fb6bed715c18814a09022907c273ec1c469fcc678738bd8efc3e9053fe1acbb11fa32da0d6881a1370e
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed

Address:         0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c
Public key:      0x08aad54f32f1b6621ee5f25267166e160147cd355a2dfc129fa646a651dd29471d814ac749c2cda831fcca361c830ba56db4b4bd5951d4953c81865d0ae0cbe7
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
```

## Deploying a Token Contract

We will now deploy the private token contract using the `deploy` command, minting 1000000 initial tokens to address `0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c`. Make sure to replace this address with one of the two you created earlier.

```
% aztec-cli deploy PrivateTokenContractAbi --args 1000000 0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c

Contract deployed at 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f
```

:::info 
If you use a different address in the constructor above, you will get an error when running the deployment. This is because you need to register an account in the sandbox before it can receive private notes. When you create a new account, it gets automatically registered. Alternatively, you can register an account you do not own along with its public key using the `register-recipient` command.
:::

This command takes 1 mandatory positional argument which is the path to the contract ABI file in a JSON format (e.g. `contracts/target/PrivateToken.json`).
Alternatively you can pass the name of an example contract as exported by `@aztec/noir-contracts` (run `aztec-cli example-contracts` to see the full list of contracts available).

The command takes a few optional arguments while the most important one is:
- `--args` - Arguments to the constructor of the contract. In this case we have minted 1000000 initial tokens to the aztec address 0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c.

The CLI tells us that the contract was successfully deployed. We can use the `check-deploy` command to verify that a contract has been successfully deployed to that address:

```
% aztec-cli check-deploy --contract-address 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f

Contract found at 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f
```

## Calling a View Method

When we deployed the token contract, an initial supply of tokens was minted to the address provided in the constructor. We can now query the `getBalance()` method on the contract to retrieve the balance of that address. Make sure to replace the `contract-address` with the deployment address you got from the previous command, and the `args` with the account you used in the constructor.

```
% aztec-cli call getBalance \
  --args 0x2337f1d5cfa6c03796db5539b0b2d5a57e9aed42665df2e0907f66820cb6eebe \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f

View result:  [
  "{\"type\":\"bigint\",\"data\":\"1000000\"}"
]
```

The `call` command calls a read-only method on a contract, one that will not generate a transaction to be sent to the network. The arguments here are:

- `--args` - The address for which we want to retrieve the balance.
- `--contract-abi` - The abi of the contract we are calling.
- `--contract-address` - The address of the deployed contract

As you can see from the result, this address has a balance of 1000000, as expected.

## Sending a Transaction

We can now send a transaction to the network. We will transfer funds from the owner of the initial minted tokens to our other account. For this we will use the `send` command, which expects as arguments the quantity of tokens to be transferred, the sender's address, and the recipient's address. Make sure to replace all addresses in this command with the ones for your run.

```
% aztec-cli send transfer \
  --args 543 0x2337f1d5cfa6c03796db5539b0b2d5a57e9aed42665df2e0907f66820cb6eebe 0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f \
  --private-key 0x2aba9e7de7075deee3e3f4ad1e47749f985f0f72543ed91063cc97a40d851f1e

Transaction has been mined
Transaction hash: 15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c
Status: mined
Block number: 5
Block hash: 163697608599543b2bee9652f543938683e4cdd0f94ac506e5764d8b908d43d4
```

We called the `transfer` function of the contract and provided these arguments:

- `--args` - The list of arguments to the function call.
- `--contract-abi` - The abi of the contract to call.
- `--contract-address` - The deployed address of the contract to call.

The command output tells us the details of the transaction such as its hash and status. We can use this hash to query the receipt of the transaction at a later time:

```
% aztec-cli get-tx-receipt 15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c

Transaction receipt:
{
  "txHash": "15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c",
  "status": "mined",
  "error": "",
  "blockHash": "163697608599543b2bee9652f543938683e4cdd0f94ac506e5764d8b908d43d4",
  "blockNumber": 5,
  "origin": "0x2337f1d5cfa6c03796db5539b0b2d5a57e9aed42665df2e0907f66820cb6eebe"
}
```

Let's now call `getBalance()` on each of our accounts and we should see updated values:

```
% aztec-cli call getBalance -a 0x2337f1d5cfa6c03796db5539b0b2d5a57e9aed42665df2e0907f66820cb6eebe -c PrivateTokenContractAbi -ca 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f

View result:  [
  "{\"type\":\"bigint\",\"data\":\"999457\"}"
]

% aztec-cli call getBalance -a 0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c -c PrivateTokenContractAbi -ca 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f

View result:  [
  "{\"type\":\"bigint\",\"data\":\"543\"}"
]
```

## Logs

Finally, we can use the CLI's `get-logs` command to retrieve unencrypted logs emitted by the contract:

```
% aztec-cli get-logs 5 1
Logs found:

Coins transferred
```

Here we asked for the logs from block 5 (the block in which our call to `transfer` was mined) and to include a total of 1 block's worth of logs. The text `Coins Transferred` is emitted during the execution of the `transfer` function on the contract.
