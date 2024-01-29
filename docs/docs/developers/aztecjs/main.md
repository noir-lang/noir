---
title: Aztec.js
---

If you are looking for the API reference, go [here](../../apis/aztec-js/index.md).

## Introduction

Aztec.js is a library that provides APIs for managing accounts and interacting with contracts on the Aztec network. It communicates with the [Private eXecution Environment (PXE)](https://docs.aztec.network/apis/pxe/interfaces/PXE) through a `PXE` implementation, allowing developers to easily register new accounts, deploy contracts, view functions, and send transactions.

## Usage

### Create a new account

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

### Deploy a contract

```typescript
import { Contract } from "@aztec/aztec.js";

const contract = await Contract.deploy(wallet, MyContractArtifact, [
  ...constructorArgs,
])
  .send()
  .deployed();
console.log(`Contract deployed at ${contract.address}`);
```

### Send a transaction

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

### Call a view function

```typescript
import { Contract } from "@aztec/aztec.js";

const contract = await Contract.at(contractAddress, MyContractArtifact, wallet);
const balance = await contract.methods.getBalance(wallet.getAddress()).view();
console.log(`Account balance is ${balance}`);
```
