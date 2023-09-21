---
title: Aztec.nr Contracts Quickstart
---

## Introduction

This guide explains the set up required to write a contract using the Aztec.nr library; then deploy it to the sandbox. Aztec.nr is a library on top of [Noir](https://noir-lang.org/) that can be used to write smart contracts for Aztec. Since Noir files use the `.nr` extension, we are calling this library "Aztec.nr".

:::info Prerequisite reading
If you haven't read [Aztec Sandbox](./sandbox.md), we recommend going there first.
:::

### Dependencies

#### `nargo`

Nargo is Noir's build tool. On your terminal, run:

<InstallNargoInstructions />

#### Aztec Sandbox

You need to setup the [Aztec sandbox](./sandbox.md).

<!-- TODO([#1056](https://github.com/AztecProtocol/aztec-packages/issues/1056)): Add a step for the starter kit -->

## Set up for aztec.nr contracts

1. Inside the yarn project you created from the [Aztec.js](./sandbox.md) page, create a sub-folder where the contracts will reside.

```bash
mkdir contracts
```

All contract projects will reside within this folder. Note that contracts don't actually have to live here and this is just a style choice.

2. Next, create a noir project using nargo by running the following in the terminal from the `contracts` folder

```bash
cd contracts
nargo new example_contract
```

This creates a noir project with a Nargo.toml (which is the manifest file of the project). This file is found at `example_contract/src/main.nr`, where we will write our contract.

Your folder should look like:

```
.
|-contracts
| |--example_contract
| |  |--src
| |  |  |--main.nr
|-src
| |--index.ts
```

Before writing the contracts, we must add the aztec.nr library. This adds smart contract utility functions for interacting with the Aztec network.

3. Add aztec.nr library as a dependency to your noir project. Open Nargo.toml that is in the `contracts/example_contract` folder, and add the dependency section as follows:

```toml
[package]
name = "example_contract"
authors = [""]
compiler_version = "0.1"
type = "contract"
 
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
```

:::note
You may need to update your dependencies depending on the contract that you are writing. For example, the token contract [imports more](../getting_started/token_contract_tutorial#project-setup).
:::

You are now ready to write your own contracts!

You can replace the content of the generated file `example_contract/src/main.nr` with your contract code.

## Next Steps

- You can learn more about writing contracts from the [Contracts section](../contracts/main.md).
- You can find more example Aztec contracts [here](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/src/contracts).
- Consider going through the [token contract tutorial](./token_contract_tutorial.md) for a deep dive on writing more advanced Aztec contracts and an introduction to some of the concepts that underpin the Aztec network architecture.
- After writing the contract, you have to compile it. Details can be found [here](../contracts/compiling.md).
- After compiling, you can deploy your contract to the Aztec network. Relevant instructions and explanations can be found [here](../contracts/deploying.md).
- Thereafter, you can interact with the contracts similar to how it was shown in the the [Creating and submitting transactions section on the Sandbox page](./sandbox.md#creating-and-submitting-transactions).
