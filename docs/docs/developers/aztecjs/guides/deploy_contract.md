---
title: How to Deploy a Contract
---

This guide explains how to deploy a smart contract using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#deploying-a-token-contract).

```typescript
import { Contract } from "@aztec/aztec.js";

const contract = await Contract.deploy(wallet, MyContractArtifact, [
  ...constructorArgs,
])
  .send()
  .deployed();
console.log(`Contract deployed at ${contract.address}`);
```