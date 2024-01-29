---
title: Creating Schnorr Accounts
---

## Introduction

This section shows how to create schnorr account wallets on the Aztec Sandbox.

An in-depth explaining about accounts on aztec can be found [here](../../learn/concepts/accounts/main.md). But creating an account on the Sandbox does 2 things:

1. Deploys an account contract -- representing you -- allowing you to perform actions on the network (deploy contracts, call functions etc).
2. Adds your encryption keys to the Private eXecution Environment (PXE) allowing it to decrypt and manage your private state.

## Pre-requisites

Have a running Sandbox and a repository that interacts with it as explained [here](../getting_started/quickstart.md).

Let's assume you have a file `src/index.ts` from the example used in the Sandbox page.

## Create accounts on the sandbox

1. Import relevant modules:

#include_code imports /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

2. Code to create an account. You must run this inside of a function:

#include_code create_accounts /yarn-project/end-to-end/src/e2e_sandbox_example.test.ts typescript

3. Running `yarn start` should now output:

```
  token Aztec Sandbox Info  {
    version: 1,
    chainId: 31337,
    rollupAddress: EthAddress {
      buffer: <Buffer cf 7e d3 ac ca 5a 46 7e 9e 70 4c 70 3e 8d 87 f6 34 fb 0f c9>
    },
    client: 'pxe@0.1.0',
    compatibleNargoVersion: '0.11.1-aztec.0'
  }
  token Creating accounts using schnorr signers... +3ms
  token Created Alice's account at 0x1509b252...0027 +10s
  token Created Bob's account at 0x031862e8...e7a3 +0ms
```

That might seem like a lot to digest but it can be broken down into the following steps:

1. We create 2 `Account` objects in Typescript. This object heavily abstracts away the mechanics of configuring and deploying an account contract and setting up a 'wallet' for signing transactions. If you aren't interested in building new types of account contracts or wallets then you don't need to be too concerned with it. In this example we have constructed account contracts and corresponding wallets that sign/verify transactions using schnorr signatures.
2. We wait for the deployment of the 2 account contracts to complete.
3. We retrieve the expected account addresses from the `Account` objects and ensure that they are present in the set of account addresses registered on the Sandbox.

Note, we use the `getRegisteredAccounts` API to verify that the addresses computed as part of the
account contract deployment have been successfully added to the Sandbox.

If you were looking at your terminal that is running the Sandbox you should have seen a lot of activity.
This is because the Sandbox will have simulated the deployment of both contracts, executed the private kernel circuit for each account deployment and later on submitted the 2 transactions to the pool.
The sequencer will have picked them up and inserted them into an L2 block and executed the recursive rollup circuits before publishing the L2 block on L1 (in our case Anvil).
Once this has completed, the L2 block is retrieved and pulled down to the PXE so that any new account state can be decrypted.

## Next Steps

Check out our section on [Writing your own Account Contract](./writing_an_account_contract.md) leveraging our account abstraction
