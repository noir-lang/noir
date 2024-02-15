---
title: How to Create a New Account
---

This guide explains how to create a new account using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#creating-accounts).

## Relevant imports

You will need to import these libraries:

#include_code create_account_imports yarn-project/end-to-end/src/docs_examples.test.ts typescript

## Define arguments needed

#include_code define_account_vars yarn-project/end-to-end/src/docs_examples.test.ts typescript

## Create the wallet with these args

#include_code create_wallet yarn-project/end-to-end/src/docs_examples.test.ts typescript

Now you have a new wallet in your PXE! To learn how to use this wallet to deploy a contract, read [this guide](./deploy_contract.md).

