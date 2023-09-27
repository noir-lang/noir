# Deploying contracts

Once you have [compiled](./compiling.md) your contracts you can proceed to deploying them using the aztec-cli or using aztec.js which is a Typescript client to interact with the sandbox.

## Prerequisites
- aztec-cli installed (go to [CLI main section](../cli/main.md) for installation instructions)
- contract artifacts ready (go to [Compiling contracts section](./compiling.md) for instructions on how to compile contracts)
- aztec-sandbox running (go to [Sandbox section](../getting_started/sandbox.md) for instructions on how to install and run the sandbox)

## Deploy

Contracts can be deployed using the `aztec-cli` or using the `aztec.js` library. 


import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli deploy /path/to/contract/abi.json
```

</TabItem>
<TabItem value="js" label="Aztec.js">

Pre-requisite - Generate type-safe typescript classes for your contract when compiling using the `@aztec/noir-compiler` package. You can install the package by running `npm install @aztec/noir-compiler`.

```ts
import { readFileSync, writeFileSync } from 'fs';
import { compileUsingNargo, generateTypescriptContractInterface} from '@aztec/noir-compiler';

const compiled: ContractAbi[] = await compileUsingNargo(projectPathToContractFolder);
const abiImportPath = "../target/Example.json";
writeFileSync(tsInterfaceDestFilePath, generateTypescriptContractInterface(compiled[0], abiImportPath));
```
This would create a typescript file like `Example.ts` in the path specified. More details in the [compiling page](./compiling.md)

Now you can import it to easily deploy and interact with the contract.
```ts
import { ExampleContract } from './Example.js';

const tx = ExampleContract.deploy(pxe).send();
await tx.wait({ interval: 0.5 });
const receipt = await tx.getReceipt();
const exampleContract = await ExampleContract.at(receipt.contractAddress!, myWallet);
```
</TabItem>
</Tabs>

### Deploy Arguments
There are several optional arguments that can be passed:
<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

`aztec-cli deploy` takes 1 mandatory argument which is the path to the contract ABI file in a JSON format (e.g. `contracts/target/PrivateToken.json`). Alternatively you can pass the name of an example contract as exported by `@aztec/noir-contracts` (run `aztec-cli example-contracts` to see the full list of contracts available).

The command also takes the following optional arguments:
- `-args <constructorArgs...>` (default: `[]`): Arguments to pass to the contract constructor.
- `--rpc-url <string>` (default: `http://localhost:8080`): URL of the PXE to connect to.
- `--public-key <string>` (default: `undefined`): Optional encryption public key for this contract.
Set this only if this contract is expected to receive private notes (in such a case the public key is used during the note encryption).
- `--salt <string>` (default: random value): Hexadecimal string used when computing the contract address of the contract being deployed.
By default is set to a random value.
Set it, if you need a deterministic contract address (same functionality as Ethereum's `CREATE2` opcode).

</TabItem>
<TabItem value="js" label="Aztec.js">

The `deploy(...)` method is generated automatically with the typescript class representing your contract.
Its arguments are `PXE` client and contract constructor arguments.

Additionally the `.send()` method can have a few optional arguments too, which are specified in an optional object:
- `portalContract?: EthAddress`: The L1 portal address to link the contract to. See the section on [Portals to learn more about them](./portals/main.md).
- `contractAddressSalt?: Fr`: A salt which is one of the inputs when computing a contract address of the contract to be deployed. 
By default is set to a random value.
Set it, if you need a deterministic contract address (same functionality as Ethereum's `CREATE2` opcode).

```ts
const tx = ExampleContract.deploy(pxe).send({ 
    portalContract: EthAddress.from("0x1234..."),
    contractAddressSalt: new Fr(3n),
});
```

</TabItem>
</Tabs>

### Deploying private token contract
To give you a more complete example we will deploy the `PrivateToken` contract whose artifacts are included in the `@aztec/noir-contracts` package.

The contract has `initial_supply` and `owner` as constructor arguments.
Because the contract sends a note to the owner specified inside the constructor, we need their public key to encrypt the note with. For this, we first need to register the owner as a recipient inside the PXE with the following command:

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli register-recipient --address 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529 --public-key 0x26e193aef4f83c70651485b5526c6d01a36d763223ab24efd1f9ff91b394ac0c20ad99d0ef669dc0dde8d5f5996c63105de8e15c2c87d8260b9e6f02f72af622 --partial-address 0x200e9a6c2d2e8352012e51c6637659713d336405c29386c7c4ac56779ab54fa7
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
const aztecAddress = AztecAddress.fromString("0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529");
const publicKey = Point.fromString("0x26e193aef4f83c70651485b5526c6d01a36d763223ab24efd1f9ff91b394ac0c20ad99d0ef669dc0dde8d5f5996c63105de8e15c2c87d8260b9e6f02f72af622");
const partialAddress = Fr.fromString("0x200e9a6c2d2e8352012e51c6637659713d336405c29386c7c4ac56779ab54fa7");

const completeAddress = CompleteAddress.create(aztecAddress, publicKey, partialKey); 
await pxe.registerRecipient(completeAddress);
```


</TabItem>
</Tabs>

When you create a new account, it gets automatically registered. It can be verified by calling `aztec-cli get-accounts` OR in aztec.js by using `await pxe.getRegisteredAccounts()`

> **NOTE 1**: If we didn't register owner as a recipient we could not encrypt a note for the owner and the contract deployment would fail because constructor execution would fail (we need owner's public key to encrypt a note).

> **NOTE 2**: If a note recipient is one of the accounts inside the PXE, we don't need to register it as a recipient because we already have the public key available.

Once the recipient is registered we can deploy the contract:

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli deploy PrivateTokenContractAbi --args 1000 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
// PrivateTokenContract is the TS interface that is automatically generated when compiling the contract with the `-ts` flag.
const initialBalance = 1000n;
const owner = AztecAddress.from("0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529");
const contract = await PrivateTokenContract.deploy(wallet, initialBalance, owner).send().deployed();
logger(`Contract deployed at ${contract.address}`);
```

</TabItem>
</Tabs>

If everything went as expected you should see the following output (with a different address):
> Contract deployed at `0x151de6120ae6628129ee852c5fc7bcbc8531055f76d4347cdc86003bbea96906`

If we pass the salt as an argument:

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli deploy PrivateTokenContractAbi --args 1000 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529 --salt 0x123
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
const contract = await PrivateTokenContract.deploy(wallet, initialBalance, owner).send({ contractAddressSalt: Fr.fromString("0x123") }).deployed();
```

</TabItem>
</Tabs>

the resulting address will be deterministic.

> **NOTE**: You can try running the deployment with the same salt the second time in which case the transaction will fail because the address has been already deployed to.
