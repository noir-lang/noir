# Deploying contracts

Once you have [compiled](./compiling.md) your contracts you can proceed to deploying them using the aztec-cli or using aztec.js which is a Typescript client to interact with the sandbox.

## Prerequisites

- `aztec-cli` and `aztec-nargo` installed (go to [CLI main section](../cli/main.md) for installation instructions)
- contract artifacts ready (go to [Compiling contracts section](./compiling.md) for instructions on how to compile contracts)
- Aztec Sandbox running (go to [Sandbox section](../getting_started/quickstart.md) for instructions on how to install and run the sandbox)

## Deploy

Contracts can be deployed using the `aztec-cli` or using the `aztec.js` library.

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli deploy /path/to/contract/artifact.json
```

</TabItem>
<TabItem value="js" label="Aztec.js">

Pre-requisite - Compile the contract and generate a type-safe typescript class for it.

Compile the contract:

```bash
aztec-nargo compile
```

Generate the typescript class:

```bash
aztec-cli codegen ./aztec-nargo/output/target/path -o src/artifacts --ts
```

This would create a typescript file like `Example.ts` in `./src/artifacts`. Read more on the [compiling page](./compiling.md).

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

</TabItem>
</Tabs>

### Deploy Arguments

There are several optional arguments that can be passed:
<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

`aztec-cli deploy` takes 1 mandatory argument which is the path to the contract artifact file in a JSON format (e.g. `contracts/target/PrivateToken.json`). Alternatively you can pass the name of an example contract as exported by `@aztec/noir-contracts` (run `aztec-cli example-contracts` to see the full list of contracts available).

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

### Deploying token contract

To give you a more complete example we will deploy a `Token` contract whose artifacts are included in the `@aztec/noir-contracts` package.

The contract has `admin` as a constructor argument.
We will deploy the contract with the `aztec-cli` and pass the `admin` address as an argument.

<Tabs groupId="deployment-methods">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli deploy TokenContractArtifact --args 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
const admin = AztecAddress.from(
  "0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529"
);
// TokenContract is the TS interface that is automatically generated when compiling the contract with the `-ts` flag.
const contract = await TokenContract.deploy(wallet, admin).send().deployed();
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
aztec-cli deploy TokenContractArtifact --args 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529 --salt 0x123
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
const contract = await TokenContract.deploy(wallet, admin)
  .send({ contractAddressSalt: Fr.fromString("0x123") })
  .deployed();
```

</TabItem>
</Tabs>

the resulting address will be deterministic.

> **NOTE**: You can try running the deployment with the same salt the second time in which case the transaction will fail because the address has been already deployed to.
