---
title: How to Deploy a Contract
sidebar_position: 4
---

# Deploying contracts

Once you have [compiled](how_to_compile_contract.md) your contracts you can proceed to deploying them using aztec.js which is a Typescript client to interact with the sandbox.

## Prerequisites

- `aztec-nargo` installed (go to [Sandbox section](../../reference/sandbox_reference/index.md) for installation instructions)
- contract artifacts ready (go to [How to Compile Contract](how_to_compile_contract.md) for instructions on how to compile contracts)
- Aztec Sandbox running (go to [Sandbox section](../../getting_started.md) for instructions on how to install and run the sandbox)

## Deploy

Contracts can be deployed using the `aztec.js` library.

Compile the contract:

```bash
aztec-nargo compile
```

Generate the typescript class:

```bash
aztec-builder codegen ./aztec-nargo/output/target/path -o src/artifacts
```

This would create a typescript file like `Example.ts` in `./src/artifacts`. Read more on the [compiling page](how_to_compile_contract.md).

You can use the `Contract` class to deploy a contract:

#include_code dapp-deploy yarn-project/end-to-end/src/sample-dapp/deploy.mjs typescript

Or you can use the generated contract class. See [below](#deploying-token-contract) for more details.

### Deploy Arguments

There are several optional arguments that can be passed:

The `deploy(...)` method is generated automatically with the typescript class representing your contract.

Additionally the `.send()` method can have a few optional arguments too, which are specified in an optional object:

#include_code deploy_options yarn-project/aztec.js/src/contract/deploy_method.ts typescript

### Deploying token contract

To give you a more complete example we will deploy a `Token` contract whose artifacts are included in the `@aztec/noir-contracts.js` package.

```ts
#include_code create_account_imports yarn-project/end-to-end/src/composed/docs_examples.test.ts raw
#include_code import_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts raw
#include_code import_token_contract yarn-project/end-to-end/src/composed/docs_examples.test.ts raw

async function main(){

    #include_code full_deploy yarn-project/end-to-end/src/composed/docs_examples.test.ts raw

}
```

:::note
You can try running the deployment with the same salt the second time in which case the transaction will fail because the address has been already deployed to.
:::
