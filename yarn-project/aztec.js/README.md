# Aztec.js

Aztec.js is a tool that provides APIs for interacting with contracts on the Aztec network. It communicates with the [AztecRPCServer](../aztec-rpc/) through an AztecRPCClient implementation, allowing developers to easily deploy contracts, view functions, and send transactions.

### Usage

#### Deploy a contract

```typescript
import { ContractDeployer } from '@aztec/aztec.js';

const deployer = new ContractDeployer(contractAbi, aztecRpcServer);
const tx = deployer.deploy(constructorArgs[0], constructorArgs[1]).send();
await tx.isMined();
console.log('Contract deployed!');

const receipt = await tx.getReceipt();
console.log(`Contract address: ${receipt.contractAddress}`);
```

#### Send a transaction

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.create(contractAddress, contractAbi, aztecRpcServer);
const tx = contract.methods
    .transfer(amount, recipientAddress))
    .send({ origin: senderAddress });
await tx.isMined();
console.log(`Transferred ${amount} to ${recipientAddress}!`);
```

#### Call a view function

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.create(contractAddress, contractAbi, aztecRpcServer);
const [balance] = contract.methods
    .getBalance(accountPublicKey))
    .view({ from: accountAddress });
console.log(`Account balance: ${balance}.`);
```
