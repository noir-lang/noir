# Contract Deployment

To add contracts to your application, we'll start by creating a new `nargo` project. We'll then compile the contracts, and write a simple script to deploy them to our Sandbox.

:::info
Follow the instructions [here](../../getting_started/noir_contracts.md) to install `nargo` if you haven't done so already.
:::

## Initialise nargo project

Create a new `contracts` folder, and from there, initialise a new project called `private_token`:

```sh
mkdir contracts && cd contracts
nargo new --contract private_token
```

Then, open the `contracts/private_token/Nargo.toml` configuration file, and add the `aztec.nr` and `value_note` libraries as dependencies:

```toml
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="master", directory="yarn-project/noir-libs/noir-aztec" }
value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="master", directory="yarn-project/noir-libs/value-note" }
```

Last, copy-paste the code from the `PrivateToken` contract into `contracts/private_token/main.nr`:

#include_code all yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr rust

## Compile your contract

We'll now use the [Aztec CLI](../../cli/main.md) to [compile](../../contracts/compiling.md) our project. If you haven't installed the CLI already, you can install it locally to your project running:

```sh
yarn add -D @aztec/cli
```

Now run the following from your project root:

```sh
yarn aztec-cli compile contracts/private_token
```

:::info
If you are using Typescript, consider including the `--typescript` option to [generate type-safe wrappers](../../contracts/compiling.md#typescript-interfaces) for your contracts.
:::

This should have created an artifact `contracts/private_token/target/private_token-Main.json` with the interface and bytecode for your contract.

## Adding a second contract

For the purposes of this tutorial, we'll set up a second contract: a public token contract. Follow the same steps as above for initialising a new Nargo project, include the dependencies, and copy-paste the following code into `contracts/public_token/main.nr`:

#include_code all yarn-project/noir-contracts/src/contracts/public_token_contract/src/main.nr rust

Compile the contract with the CLI:

```sh
yarn aztec-cli compile contracts/public_token
```

With both contracts ready, we'll now proceed to deployment.

## Deploy your contracts

Let's now write a script for deploying your contracts to the Sandbox. We'll create an RPC client, and then use the `ContractDeployer` class to deploy our contracts, and store the deployment address to a local JSON file.

Create a new file `src/deploy.mjs`, importing the contract artifacts we have generated plus the dependencies we'll need, and with a call to a `main` function that we'll populate in a second:

```js
// src/deploy.mjs
import { writeFileSync } from 'fs';
import { createAztecRpcClient, ContractDeployer } from '@aztec/aztec.js';
import PrivateTokenArtifact from '../contracts/private_token/target/PrivateToken.json' assert { type: 'json' };
import PublicTokenArtifact from '../contracts/public_token/target/PublicToken.json' assert { type: 'json' };

async function main() { }

main().catch(err => {
  console.error(`Error in deployment script: ${err}`);
  process.exit(1);
});
```

Now we can deploy the contracts by adding the following code to the `src/deploy.mjs` file. Here, we are using the `ContractDeployer` class with the compiled artifact to send a new deployment transaction. The `wait` method will block execution until the transaction is successfully mined, and return a receipt with the deployed contract address. 

#include_code dapp-deploy yarn-project/end-to-end/src/sample-dapp/deploy.mjs javascript

Note that the private token constructor expects an `owner` address to mint an initial set of tokens to. We are using the first account from the Sandbox for this.

:::info
If you are using the generated typescript classes, you can drop the generic `ContractDeployer` in favor of using the `deploy` method of the generated class, which will automatically load the artifact for you and type-check the constructor arguments:

```typescript
await PrivateToken.deploy(client, 100n, owner.address).send().wait();
```
:::

Run the snippet above as `node src/deploy.mjs`, and you should see the following output, along with a new `addresses.json` file in your project root:

```text
Private token deployed to 0x2950b0f290422ff86b8ee8b91af4417e1464ddfd9dda26de8af52dac9ea4f869
Public token deployed to 0x2b54f68fd1e18f7dcfa71e3be3c91bb06ecbe727a28d609e964c225a4b5549c8
```
## Next steps

Now that we have our contracts set up, it's time to actually [start writing our application that will be interacting with them](./contract_interaction.md).