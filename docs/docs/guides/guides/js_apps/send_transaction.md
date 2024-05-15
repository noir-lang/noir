---
title: How to Send a Transaction
---

This guide explains how to send a transaction using Aztec.js.

## Prerequisites

You should have a wallet to act as the transaction sender, and a contract that has been deployed.

You can learn how to create wallets from [this guide](./create_account.md).

You can learn how to deploy a contract [here](./deploy_contract.md).

## Relevant imports

You will need to import this library:

#include_code import_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

## Define contract

Get a previously deployed contract like this:

#include_code get_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

## Call method

#include_code send_transaction yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript
