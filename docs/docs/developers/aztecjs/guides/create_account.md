---
title: How to Create a New Account
---

This guide explains how to create a new account using [Aztec.js](../main.md).

To do this from the CLI, go [here](../../sandbox/references/cli-commands.md#creating-accounts).

```typescript
import { getSchnorrAccount } from "@aztec/aztec.js";
import { GrumpkinPrivateKey } from "@aztec/circuit-types";

const encryptionPrivateKey = GrumpkinPrivateKey.random();
const signingPrivateKey = GrumpkinPrivateKey.random();
const wallet = getSchnorrAccount(
  pxe,
  encryptionPrivateKey,
  signingPrivateKey
).waitDeploy();
console.log(`New account deployed at ${wallet.getAddress()}`);
```