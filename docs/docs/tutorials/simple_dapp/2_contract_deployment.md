# Contract Deployment

To add contracts to your application, we'll start by creating a new `aztec-nargo` project. We'll then compile the contracts, and write a simple script to deploy them to our Sandbox.

:::info
Follow the instructions [here](../../reference/sandbox_reference/index.md) to install `aztec-nargo` if you haven't done so already.
:::

## Initialize Aztec project

Create a new `contracts` folder, and from there, initialize a new project called `token`:

```sh
mkdir contracts && cd contracts
aztec-nargo new --contract token
```

Then, open the `contracts/token/Nargo.toml` configuration file, and add the `aztec.nr` and `value_note` libraries as dependencies:

```toml
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/aztec" }
authwit = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/authwit"}
compressed_string = {git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/compressed-string"}
```

Last, copy-paste the code from the `Token` contract into `contracts/token/main.nr`:

#include_code token_all noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

### Helper files

The `Token` contract also requires some helper files. You can view the files [here](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/noir-projects/noir-contracts/contracts/token_contract/src). Copy the `types.nr` and the `types` folder into `contracts/token/src`.

## Compile your contract

We'll now use `aztec-nargo` to compile.

Now run the following from your contract folder (containing Nargo.toml):

```sh
aztec-nargo compile
```

## Deploy your contracts

Let's now write a script for deploying your contracts to the Sandbox. We'll create a Private eXecution Environment (PXE) client, and then use the `ContractDeployer` class to deploy our contracts, and store the deployment address to a local JSON file.

Create a new file `src/deploy.mjs`:

```js
// src/deploy.mjs
import { writeFileSync } from 'fs';
import { Contract, loadContractArtifact, createPXEClient } from '@aztec/aztec.js';
import { getInitialTestAccountsWallets } from '@aztec/accounts/testing';
import TokenContractJson from "../contracts/token/target/token_contract-Token.json" assert { type: "json" };


#include_code dapp-deploy yarn-project/end-to-end/src/sample-dapp/deploy.mjs raw

main().catch((err) => {
  console.error(`Error in deployment script: ${err}`);
  process.exit(1);
});
```

We import the contract artifacts we have generated plus the dependencies we'll need, and then we can deploy the contracts by adding the following code to the `src/deploy.mjs` file. Here, we are using the `ContractDeployer` class with the compiled artifact to send a new deployment transaction. The `wait` method will block execution until the transaction is successfully mined, and return a receipt with the deployed contract address.

Note that the token's `_initialize()` method expects an `owner` address to mint an initial set of tokens to. We are using the first account from the Sandbox for this.

:::info
If you are using the generated typescript classes, you can drop the generic `ContractDeployer` in favor of using the `deploy` method of the generated class, which will automatically load the artifact for you and type-check the constructor arguments. SEe the [How to deploy a contract](../../guides/smart_contracts/how_to_deploy_contract.md) page for more info.
:::

Run the snippet above as `node src/deploy.mjs`, and you should see the following output, along with a new `addresses.json` file in your project root:

```text
Token deployed to 0x2950b0f290422ff86b8ee8b91af4417e1464ddfd9dda26de8af52dac9ea4f869
```

## Next steps

Now that we have our contracts set up, it's time to actually [start writing our application that will be interacting with them](./3_contract_interaction.md).
