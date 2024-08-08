---
title: How to Simulate a Function Call
tags: [functions]
---

This guide explains how to `simulate` a function call using Aztec.js.

## Prerequisites

You should have a wallet to act as the caller, and a contract that has been deployed.

You can learn how to create wallets from [this guide](./create_account.md).

You can learn how to deploy a contract [here](./deploy_contract.md).

## Relevant imports

You will need to import this from Aztec.js:

#include_code import_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

## Define contract

Get a previously deployed contract like this:

#include_code get_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

## Simulating function calls

Call the `simulate` function on the typescript contract wrapper like this:

#include_code simulate_function yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

:::info Note
- If the simulated function is `unconstrained` you will get a properly typed value.
- If the simulated function is `public` or `private` it will return a Field array of size 4.
:::
