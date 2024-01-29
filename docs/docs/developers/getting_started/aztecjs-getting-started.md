---
title: Getting Started with Aztec.js
---

import Image from "@theme/IdealImage";

In this guide, we will retrieving the Sandbox and deploy a pre-written contract to it using Aztec.js.

This guide assumes you have followed the [quickstart](./quickstart.md).

## Prerequisites

- A running [Aztec sandbox](./quickstart.md)

## Set up the project

We will deploy a pre-compiled token contract, and send tokens privately, using the Sandbox.

:::info
Find the full code [here](https://github.com/AztecProtocol/dev-rel/tree/main/tutorials/sandbox-tutorial/token)
:::

We will create a `yarn` project called `token` (although `npm` works fine too).

1. Initialize a yarn project

```sh
mkdir token
cd token
yarn init -yp
```

2. Create a `src` folder inside your new `token` directory:

```sh
mkdir src
```

3. Add necessary yarn packages (and optionally add typescript too)

```sh
yarn add @aztec/aztec.js @aztec/accounts @aztec/noir-contracts typescript @types/node
```

4. [Optional] If creating a typescript file, add a `tsconfig.json` file into the project root, here is an example:

```json
{
  "compilerOptions": {
    "outDir": "dest",
    "rootDir": "src",
    "target": "es2020",
    "lib": ["dom", "esnext", "es2017.object"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "declaration": true,
    "allowSyntheticDefaultImports": true,
    "esModuleInterop": true,
    "downlevelIteration": true,
    "inlineSourceMap": true,
    "declarationMap": true,
    "importHelpers": true,
    "resolveJsonModule": true,
    "composite": true,
    "skipLibCheck": true
  },
  "references": [],
  "include": ["src", "src/*.json"]
}
```

5. Update `package.json` - Add a `scripts` section to `package.json` and set `"type": "module"`:

```json
{
  "name": "token",
  "version": "1.0.0",
  "description": "My first token contract",
  "main": "index.js",
  "author": "1000x Dev",
  "license": "MIT",
  "type": "module",
  "scripts": {
    "build": "yarn clean && tsc -b",
    "build:dev": "tsc -b --watch",
    "clean": "rm -rf ./dest tsconfig.tsbuildinfo",
    "start": "yarn build && DEBUG='token' node ./dest/index.js"
  },
  "dependencies": {
    "@aztec/accounts": "latest",
    "@aztec/aztec.js": "latest",
    "@aztec/noir-contracts": "latest",
    "@types/node": "^20.6.3",
    "typescript": "^5.2.2"
  }
}
```

6. Create an `index.ts` file in the `src` directory with the following sandbox connection setup:

```ts
#include_code imports /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts raw

async function main() {
#include_code setup /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts raw
}

main();
```

7. Finally, run the package:

In the project root, run

```sh
yarn start
```

A successful run should show something like this:

```
  token Aztec Sandbox Info  {
  token   sandboxVersion: '#include_aztec_short_version',
  token   chainId: 31337,
  token   protocolVersion: 1,
  token   l1ContractAddresses: {
  token     rollupAddress: EthAddress {
  token       buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
  token     },
  token     registryAddress: EthAddress {
  token       buffer: <Buffer 5f bd b2 31 56 78 af ec b3 67 f0 32 d9 3f 64 2f 64 18 0a a3>
  token     },
  token     inboxAddress: EthAddress {
  token       buffer: <Buffer e7 f1 72 5e 77 34 ce 28 8f 83 67 e1 bb 14 3e 90 bb 3f 05 12>
  token     },
  token     outboxAddress: EthAddress {
  token       buffer: <Buffer 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00>
  token     },
  token     contractDeploymentEmitterAddress: EthAddress {
  token       buffer: <Buffer 5f c8 d3 26 90 cc 91 d4 c3 9d 9d 3a bc bd 16 98 9f 87 57 07>
  token     }
  token   }
  token } +0ms
```

Great! The Sandbox is running and we are able to interact with it.

## Load accounts

The sandbox is preloaded with multiple accounts so you don't have to sit and create them. Let's load these accounts. Add this code to the `main()` function in `index.ts` below the code that's there:

#include_code load_accounts /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

An explanation on accounts on Aztec can be found [here](../../learn/concepts/accounts/main.md).

If you want more accounts, you can find instructions in the [Account creation section](../wallets/creating_schnorr_accounts.md).

## Deploy a contract

Now that we have our accounts loaded, let's move on to deploy our pre-compiled token smart contract. You can find the full code for the contract [here](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/contracts/token_contract/src). Add this to `index.ts` below the code you added earlier:

#include_code Deployment /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

`yarn start` will now give something like this:

```
  token Aztec Sandbox Info  {
  token   sandboxVersion: '#include_aztec_short_version',
  token   chainId: 31337,
  token   protocolVersion: 1,
  token   l1ContractAddresses: {
  token     rollupAddress: EthAddress {
  token       buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
  token     },
  token     registryAddress: EthAddress {
  token       buffer: <Buffer 5f bd b2 31 56 78 af ec b3 67 f0 32 d9 3f 64 2f 64 18 0a a3>
  token     },
  token     inboxAddress: EthAddress {
  token       buffer: <Buffer e7 f1 72 5e 77 34 ce 28 8f 83 67 e1 bb 14 3e 90 bb 3f 05 12>
  token     },
  token     outboxAddress: EthAddress {
  token       buffer: <Buffer 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00>
  token     },
  token     contractDeploymentEmitterAddress: EthAddress {
  token       buffer: <Buffer 5f c8 d3 26 90 cc 91 d4 c3 9d 9d 3a bc bd 16 98 9f 87 57 07>
  token     }
  token   }
  token } +0ms
  token Loaded alice's account at 0x25048e8c...70d0 +4s
  token Loaded bob's account at 0x115f123b...6483 +0ms
  token Deploying token contract... +0ms
  token Contract successfully deployed at address 0x11a03dce...afc7 +5s
  token Minting tokens to Alice... +18ms
  token 1000000 tokens were successfully minted and redeemed by Alice +10s
```

We can break this down as follows:

1. We create and send a contract deployment transaction to the network.
2. We wait for it to be successfully mined.
3. We retrieve the transaction receipt containing the transaction status and contract address.
4. We connect to the contract with Alice
5. Alice initialize the contract with herself as the admin and a minter.
6. Alice mints 1,000,000 tokens to be claimed by herself in private.
7. Alice redeems the tokens privately.

## View the balance of an account

A token contract wouldn't be very useful if you aren't able to query the balance of an account. As part of the deployment, tokens were minted to Alice. We can now call the contract's `balance_of_private()` function to retrieve the balances of the accounts.

Call the `balance_of_private` function using the following code (paste this):

#include_code Balance /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

Running now should yield output:

```
  token Aztec Sandbox Info  {
  token   sandboxVersion: '#include_aztec_short_version',
  token   chainId: 31337,
  token   protocolVersion: 1,
  token   l1ContractAddresses: {
  token     rollupAddress: EthAddress {
  token       buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
  token     },
  token     registryAddress: EthAddress {
  token       buffer: <Buffer 5f bd b2 31 56 78 af ec b3 67 f0 32 d9 3f 64 2f 64 18 0a a3>
  token     },
  token     inboxAddress: EthAddress {
  token       buffer: <Buffer e7 f1 72 5e 77 34 ce 28 8f 83 67 e1 bb 14 3e 90 bb 3f 05 12>
  token     },
  token     outboxAddress: EthAddress {
  token       buffer: <Buffer 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00>
  token     },
  token     contractDeploymentEmitterAddress: EthAddress {
  token       buffer: <Buffer 5f c8 d3 26 90 cc 91 d4 c3 9d 9d 3a bc bd 16 98 9f 87 57 07>
  token     }
  token   }
  token } +0ms
  token Loaded alice's account at 0x25048e8c...70d0 +4s
  token Loaded bob's account at 0x115f123b...6483 +0ms
  token Deploying token contract... +0ms
  token Contract successfully deployed at address 0x1b388d99...4b55 +4s
  token Minting tokens to Alice... +10ms
  token 1000000 tokens were successfully minted and redeemed by Alice +10s
  token Alice's balance 1000000 +80ms
  token Bob's balance 0 +31ms
```

Above, we created a second instance of the `TokenContract` contract class.
This time pertaining to Bob.
This class offers a TypeScript bindings of our `Token` contract..
We then call `balance_of_private()` as a `view` method.
View methods can be thought as read-only.
No transaction is submitted as a result but a user's state can be queried.

We can see that each account has the expected balance of tokens.

### Calling an unconstrained (view) function

<img src="/img/sandbox_unconstrained_function.svg" alt="Unconstrained function call" />

## Create and submit a transaction

Now lets transfer some funds from Alice to Bob by calling the `transfer` function on the contract. This function takes 4 arguments:

1. The sender.
2. The recipient.
3. The quantity of tokens to be transferred.
4. The nonce for the [authentication witness](../../learn//concepts/accounts/main.md#authorizing-actions), or 0 if msg.sender equal sender.

Here is the Typescript code to call the `transfer` function, add this to your `index.ts` at the bottom of the `main` function:

#include_code Transfer /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

Our output should now look like this:

```
  token Aztec Sandbox Info  {
  token   sandboxVersion: '#include_aztec_short_version',
  token   chainId: 31337,
  token   protocolVersion: 1,
  token   l1ContractAddresses: {
  token     rollupAddress: EthAddress {
  token       buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
  token     },
  token     registryAddress: EthAddress {
  token       buffer: <Buffer 5f bd b2 31 56 78 af ec b3 67 f0 32 d9 3f 64 2f 64 18 0a a3>
  token     },
  token     inboxAddress: EthAddress {
  token       buffer: <Buffer e7 f1 72 5e 77 34 ce 28 8f 83 67 e1 bb 14 3e 90 bb 3f 05 12>
  token     },
  token     outboxAddress: EthAddress {
  token       buffer: <Buffer 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00>
  token     },
  token     contractDeploymentEmitterAddress: EthAddress {
  token       buffer: <Buffer 5f c8 d3 26 90 cc 91 d4 c3 9d 9d 3a bc bd 16 98 9f 87 57 07>
  token     }
  token   }
  token } +0ms
  token Loaded alice's account at 0x25048e8c...70d0 +4s
  token Loaded bob's account at 0x115f123b...6483 +0ms
  token Deploying token contract... +0ms
  token Contract successfully deployed at address 0x01d8af7d...9a4d +5s
  token Minting tokens to Alice... +18ms
  token 1000000 tokens were successfully minted and redeemed by Alice +11s
  token Alice's balance 1000000 +59ms
  token Bob's balance 0 +33ms
  token Transferring 543 tokens from Alice to Bob... +0ms
  token Alice's balance 999457 +6s
  token Bob's balance 543 +39ms
```

Here, we used the same contract abstraction as was previously used for reading Alice's balance. But this time we called `send()` generating and sending a transaction to the network. After waiting for the transaction to settle we were able to check the new balance values.

Finally, the contract has 2 `mint` functions that can be used to generate new tokens for an account.
We will focus only on `mint_private`.
This function is public but it mints tokens privately.
This function takes:

1. A quantity of tokens to be minted.
2. A secret hash.

This function is public and it inserts a new note into the note hash tree and increases the total token supply by the amount minted.

To make the note spendable the note has to be redeemed. A user can do that by calling the `redeem_shield` function.

Let's now use these functions to mint some tokens to Bob's account using Typescript, add this to `index.ts`:

#include_code Mint /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

Our complete output should now be something like:

```
  token Aztec Sandbox Info  {
  token   sandboxVersion: '#include_aztec_short_version',
  token   chainId: 31337,
  token   protocolVersion: 1,
  token   l1ContractAddresses: {
  token     rollupAddress: EthAddress {
  token       buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
  token     },
  token     registryAddress: EthAddress {
  token       buffer: <Buffer 5f bd b2 31 56 78 af ec b3 67 f0 32 d9 3f 64 2f 64 18 0a a3>
  token     },
  token     inboxAddress: EthAddress {
  token       buffer: <Buffer e7 f1 72 5e 77 34 ce 28 8f 83 67 e1 bb 14 3e 90 bb 3f 05 12>
  token     },
  token     outboxAddress: EthAddress {
  token       buffer: <Buffer 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00>
  token     },
  token     contractDeploymentEmitterAddress: EthAddress {
  token       buffer: <Buffer 5f c8 d3 26 90 cc 91 d4 c3 9d 9d 3a bc bd 16 98 9f 87 57 07>
  token     }
  token   }
  token } +0ms
  token Loaded alice's account at 0x25048e8c...70d0 +4s
  token Loaded bob's account at 0x115f123b...6483 +0ms
  token Deploying token contract... +0ms
  token Contract successfully deployed at address 0x03a0bb2c...02c2 +7s
  token Minting tokens to Alice... +19ms
  token 1000000 tokens were successfully minted and redeemed by Alice +9s
  token Alice's balance 1000000 +43ms
  token Bob's balance 0 +31ms
  token Transferring 543 tokens from Alice to Bob... +0ms
  token Alice's balance 999457 +6s
  token Bob's balance 543 +36ms
  token Minting 10000 tokens to Bob... +5s
  token Alice's balance 999457 +9s
  token Bob's balance 10543 +43ms
```

That's it! We have successfully deployed a token contract to an instance of the Aztec network and mined private state-transitioning transactions. We have also queried the resulting state all via the interfaces provided by the contract. To see exactly what has happened here, you can learn about the transaction flow [here](../../learn/concepts/transactions.md).

## Next Steps

Learn more about writing Aztec.nr contracts in the [Aztec.nr getting started guide](./aztecnr-getting-started.md).
