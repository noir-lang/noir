---
title: How to Send a Transaction
---

This guide explains how to send a transaction using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#sending-a-transaction).

```typescript
import { Contract } from "@aztec/aztec.js";

const contract = await Contract.at(contractAddress, MyContractArtifact, wallet);
const tx = await contract.methods
  .transfer(amount, recipientAddress)
  .send()
  .wait();
console.log(
  `Transferred ${amount} to ${recipientAddress} on block ${tx.blockNumber}`
);
```