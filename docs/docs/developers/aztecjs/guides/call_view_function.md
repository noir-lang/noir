---
title: How to Call a View Function
---

This guide explains how to call a `view` function using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#calling-an-unconstrained-view-function).

```typescript
import { Contract } from "@aztec/aztec.js";

const contract = await Contract.at(contractAddress, MyContractArtifact, wallet);
const balance = await contract.methods.getBalance(wallet.getAddress()).view();
console.log(`Account balance is ${balance}`);
```
