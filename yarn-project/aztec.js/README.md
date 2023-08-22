# Aztec.js

Aztec.js is a tool that provides APIs for interacting with contracts on the Aztec network. It communicates with the [AztecRPCServer](../aztec-rpc/) through an AztecRPCClient implementation, allowing developers to easily deploy contracts, view functions, and send transactions.

### Usage

#### Deploy a contract

```typescript
import { ContractDeployer } from '@aztec/aztec.js';

const deployer = new ContractDeployer(contractAbi, aztecRpcServer);
const tx = deployer.deploy(constructorArgs[0], constructorArgs[1]).send();
// wait for tx to be mined
const receipt = await tx.wait();
console.log(`Contract deployed at ${receipt.contractAddress}`);
```

#### Send a transaction

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.at(contractAddress, contractAbi, aztecRpcServer);
const tx = contract.methods
    .transfer(amount, recipientAddress)
    .send({ origin: senderAddress });

// wait for tx to be mined
await tx.wait();
console.log(`Transferred ${amount} to ${recipientAddress}!`);
```

#### Call a view function

```typescript
import { Contract } from '@aztec/aztec.js';

const contract = await Contract.at(contractAddress, contractAbi, aztecRpcServer);
const balance = contract.methods
    .getBalance(accountPublicKey))
    .view({ from: accountAddress });
console.log(`Account balance: ${balance}.`);
```
