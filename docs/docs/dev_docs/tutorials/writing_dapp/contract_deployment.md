# Contract Deployment

To add contracts to your application, we'll start by creating a new `nargo` project. We'll then compile the contracts, and write a simple script to deploy them to our Sandbox.

:::info
Follow the instructions [here](../../contracts/setup.md) to install `nargo` if you haven't done so already.
:::

## Initialise nargo project

Create a new `contracts` folder, and from there, initialise a new project called `token`:

```sh
mkdir contracts && cd contracts
nargo new --contract token
```

Then, open the `contracts/token/Nargo.toml` configuration file, and add the `aztec.nr` and `value_note` libraries as dependencies:

```toml
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/value-note"}
safe_math = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/safe-math"}
```

Last, copy-paste the code from the `Token` contract into `contracts/token/main.nr`:

#include_code token_all yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

The `Token` contract also requires two helper files. Copy-them too:

Create `contracts/token/types.nr` and copy-paste the following:

#include_code token_types_all yarn-project/noir-contracts/src/contracts/token_contract/src/types.nr rust

Finally, create `contracts/token/util.nr` and copy-paste the following:

#include_code token_util_all yarn-project/noir-contracts/src/contracts/token_contract/src/util.nr rust

## Compile your contract

We'll now use the [Aztec CLI](../../cli/main.md) to [compile](../../contracts/compiling.md) our project. If you haven't installed the CLI already, you can install it locally to your project running:

```sh
yarn add -D @aztec/cli
```

Now run the following from your project root:

```sh
yarn aztec-cli compile contracts/token
```

:::info
If you are using Typescript, consider including the `--typescript` option to [generate type-safe wrappers](../../contracts/compiling.md#typescript-interfaces) for your contracts.
:::

This should have created an artifact `contracts/token/target/Token.json` with the interface and bytecode for your contract.

## Deploy your contracts

Let's now write a script for deploying your contracts to the Sandbox. We'll create an RPC client, and then use the `ContractDeployer` class to deploy our contracts, and store the deployment address to a local JSON file.

Create a new file `src/deploy.mjs`:

```js
// src/deploy.mjs
import { writeFileSync } from 'fs';
import { Contract, ContractDeployer, createAztecRpcClient, getSandboxAccountsWallets } from '@aztec/aztec.js';
import TokenContractAbi from "../contracts/token/target/Token.json" assert { type: "json" };

#include_code dapp-deploy yarn-project/end-to-end/src/sample-dapp/deploy.mjs raw

main().catch((err) => {
  console.error(`Error in deployment script: ${err}`);
  process.exit(1);
});
```

We import the contract artifacts we have generated plus the dependencies we'll need, and then we can deploy the contracts by adding the following code to the `src/deploy.mjs` file. Here, we are using the `ContractDeployer` class with the compiled artifact to send a new deployment transaction. The `wait` method will block execution until the transaction is successfully mined, and return a receipt with the deployed contract address.

Note that the token's `_initialize()` method expects an `owner` address to mint an initial set of tokens to. We are using the first account from the Sandbox for this.

:::info
If you are using the generated typescript classes, you can drop the generic `ContractDeployer` in favor of using the `deploy` method of the generated class, which will automatically load the artifact for you and type-check the constructor arguments:

```typescript
await Token.deploy(client).send().wait();
```

:::

Run the snippet above as `node src/deploy.mjs`, and you should see the following output, along with a new `addresses.json` file in your project root:

```text
Token deployed to 0x2950b0f290422ff86b8ee8b91af4417e1464ddfd9dda26de8af52dac9ea4f869
```

## Next steps

Now that we have our contracts set up, it's time to actually [start writing our application that will be interacting with them](./contract_interaction.md).
