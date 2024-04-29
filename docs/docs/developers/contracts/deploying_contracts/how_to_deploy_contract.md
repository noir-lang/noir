---
title: How to deploy a contract
---

# Deploying contracts

Once you have [compiled](../compiling_contracts/how_to_compile_contract.md) your contracts you can proceed to deploying them using aztec.js which is a Typescript client to interact with the sandbox.

## Prerequisites

- `aztec-nargo` installed (go to [Sandbox and CLI section](../../sandbox/main.md) for installation instructions)
- contract artifacts ready (go to [How to Compile Contract](../compiling_contracts/how_to_compile_contract.md) for instructions on how to compile contracts)
- Aztec Sandbox running (go to [Sandbox section](../../getting_started/quickstart.md) for instructions on how to install and run the sandbox)

## Deploy

Contracts can be deployed using the `aztec.js` library.

Compile the contract:

```bash
aztec-nargo compile
```

Generate the typescript class:

```bash
aztec-builder ./aztec-nargo/output/target/path -o src/artifacts
```

This would create a typescript file like `Example.ts` in `./src/artifacts`. Read more on the [compiling page](../compiling_contracts/how_to_compile_contract.md).

Now you can import it to easily deploy and interact with the contract.

```ts
import { ExampleContract } from "./target/Example.js";

const tx = ExampleContract.deploy(pxe).send();
await tx.wait({ interval: 0.5 });
const receipt = await tx.getReceipt();
const exampleContract = await ExampleContract.at(
  receipt.contractAddress!,
  myWallet
);
```
### Deploy Arguments

There are several optional arguments that can be passed:

The `deploy(...)` method is generated automatically with the typescript class representing your contract.
Its arguments are `PXE` client and contract constructor arguments.

Additionally the `.send()` method can have a few optional arguments too, which are specified in an optional object:

- `contractAddressSalt?: Fr`: A salt which is one of the inputs when computing a contract address of the contract to be deployed.
  By default is set to a random value.
  Set it, if you need a deterministic contract address (same functionality as Ethereum's `CREATE2` opcode).

```ts
const tx = ExampleContract.deploy(pxe).send({
  contractAddressSalt: new Fr(3n),
});
```

### Deploying token contract

To give you a more complete example we will deploy a `Token` contract whose artifacts are included in the `@aztec/noir-contracts.js` package.

The contract has `admin` as a constructor argument.
We will deploy the contract and pass the `admin` address as an argument.

```ts
const admin = AztecAddress.from(
  "0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529"
);
// TokenContract is the TS interface that is automatically generated when compiling the contract with the `-ts` flag.
const contract = await TokenContract.deploy(wallet, admin).send().deployed();
logger(`Contract deployed at ${contract.address}`);
```

If everything went as expected you should see the following output (with a different address):

> Contract deployed at `0x151de6120ae6628129ee852c5fc7bcbc8531055f76d4347cdc86003bbea96906`

If we pass the salt as an argument:

```ts
const contract = await TokenContract.deploy(wallet, admin)
  .send({ contractAddressSalt: Fr.fromString("0x123") })
  .deployed();
```

the resulting address will be deterministic.

> **NOTE**: You can try running the deployment with the same salt the second time in which case the transaction will fail because the address has been already deployed to.
