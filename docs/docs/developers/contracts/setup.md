---
title: Setup
---

import { AztecPackagesVersion } from "@site/src/components/Version";

## Introduction

This guide explains the set up required to write a contract using the Aztec.nr library.

:::info Prerequisite reading
If you haven't read about [Aztec.nr](./main.md), we recommend going there first.
:::

### Dependencies

#### Aztec Sandbox

You need to setup the [Aztec sandbox](../getting_started/quickstart.md).

<!-- TODO([#1056](https://github.com/AztecProtocol/aztec-packages/issues/1056)): Add a step for the starter kit -->

## Setup for Aztec.nr contracts

1. Inside the yarn project you created from the [Sandbox page](../getting_started/quickstart.md), create a sub-folder where the contracts will reside.

```bash
mkdir contracts
```

All contract projects will reside within this folder. Note that contracts don't actually have to live here and this is just a style choice.

1. Next, create an Aztec contract project using aztec-nargo by running the following in the terminal from the `contracts` folder

```bash
cd contracts
aztec-nargo new --contract example_contract
```

This creates `example_contract` directory within contracts which is a Noir project with:

- a Nargo.toml (which is the manifest file of the project) at `example_contract/Nargo.toml`.
- a main.nr file (the file where our contract will reside) at `example_contract/src/main.nr`.

Your folder should look like:

```tree
.
|-contracts
| |--example_contract
| |  |--src
| |  |  |--main.nr
| |  |--Nargo.toml
|-src
| |--index.ts
```

Before writing the contracts, we must add the aztec.nr library. This adds smart contract utility functions for interacting with the Aztec network.

3. Finally, add relevant aztec-nr dependencies that you might use such as `aztec.nr`, `value_note` and `safe_math` libraries.

Open Nargo.toml that is in the `contracts/example_contract` folder, and add the dependency section as follows

```toml
[package]
name = "example_contract"
authors = [""]
compiler_version = ">=0.18.0"
type = "contract"

[dependencies]
# Framework import
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }

# Utility dependencies
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/value-note"}
safe_math = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/safe-math"}
```

:::info
Note: currently the dependency name **_MUST_** be `aztec`. The framework expects this namespace to be available when compiling into contracts. This limitation may be removed in the future.
:::

You are now ready to write your own contracts!

## Next Steps

- Read up about how to [write a contract](./syntax/main.md) OR
- Follow a [tutorial](../tutorials/main.md)
