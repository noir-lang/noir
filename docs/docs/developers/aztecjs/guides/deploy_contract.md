---
title: How to Deploy a Contract
---

This guide explains how to deploy a smart contract using [Aztec.js](../main.md).

## Prerequisites

You should have a wallet to act as the deployer, and a contract artifact ready to be deployed.

You can learn how to create wallets from [this guide](./create_account.md).

You can read about contract artifacts [here](../../contracts/compiling_contracts/artifacts.md).

## Import the contract artifact

In this guide we are using a Token contract artifact. This comes from the [token contract tutorial](../../tutorials/writing_token_contract.md).

#include_code import_token_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

## Deploy contract

#include_code deploy_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts typescript

To learn how to send a transaction from Aztec.js read [this guide](./send_transaction.md). You can also call a `view` function from Aztec.js by reading [this guide](./call_view_function.md).
