---
title: How to Call a View Function
---

This guide explains how to call a `view` function using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#calling-an-unconstrained-view-function).

## Prerequisites

You should have a wallet to act as the caller, and a contract that has been deployed.

You can learn how to create wallets from [this guide](./create_account.md).

You can learn how to deploy a contract [here](./deploy_contract.md).

## Relevent imports

You will need to import this from Aztec.js:

#include_code import_contract yarn-project/end-to-end/src/docs_examples.test.ts typescript

## Define contract

Get a previously deployed contract like this:

#include_code get_contract yarn-project/end-to-end/src/docs_examples.test.ts typescript

## Call view function

Call the `view` function on the contract like this:

#include_code call_view_function yarn-project/end-to-end/src/docs_examples.test.ts typescript

