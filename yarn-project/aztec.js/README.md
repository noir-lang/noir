# Aztec.js

Aztec.js is a library that provides APIs for managing accounts and interacting with contracts on the Aztec network. It communicates with the [Private eXecution Environment (PXE)](https://docs.aztec.network/reference/aztecjs/pxe) through a `PXE` implementation, allowing developers to easily register new accounts, deploy contracts, view functions, and send transactions.

## Installing

```
npm install @aztec/aztec.js
```

## Usage

Use the `@aztec/accounts` package in order to create and manage accounts, and acquire a `Wallet` object needed to send transactions and interact with the network.

### Deploy a contract

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.deploy(wallet, MyContractArtifact, [...constructorArgs])
  .send()
  .deployed();
console.log(`Contract deployed at ${contract.address}`);
```

### Send a transaction

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.at(contractAddress, MyContractArtifact, wallet);
const tx = await contract.methods.transfer(amount, recipientAddress).send().wait();
console.log(`Transferred ${amount} to ${recipientAddress} on block ${tx.blockNumber}`);
```

### Simulate a function

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.at(contractAddress, MyContractArtifact, wallet);
const balance = await contract.methods.get_balance(wallet.getAddress()).simulate();
console.log(`Account balance is ${balance}`);
```
