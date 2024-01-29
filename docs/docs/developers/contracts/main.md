import DocCardList from '@theme/DocCardList';

# Aztec.nr

## What is Aztec.nr?

**Aztec.nr** is a framework for writing Aztec smart contracts.

## Nomenclature

[**Noir**](https://noir-lang.org/) is a domain specific language for creating and verifying proofs. It's design choices are influenced heavily by Rust.

A **smart contract** is just a collection of persistent state variables, and a collection of functions which may edit those state variables.

An **Aztec smart contract** is a smart contract with **private** state variables and **private** functions.

**Aztec.nr** is a framework for writing Aztec smart contracts, written in Noir.

# Getting started

## Install aztec-nargo

To write an Aztec.nr contract, you need to the compiler, `aztec-nargo` which is installed when you install the sandbox. See install instructions [here](../cli/sandbox-reference.md).

:::info
For those coming from vanilla Noir, the version used for aztec.nr is tracked separately to nargo for vanilla Noir. Be sure to use `aztec-nargo` to compile your contracts.
:::

## Install Noir LSP (recommended)

Install the [Noir Language Support extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir) to get syntax highlighting, syntax error detection and go-to definitions for your Aztec contracts.

Once the extension is installed, go to your VSCode settings, search for "noir" and update the `Noir: Nargo Path` field to point to your `aztec-nargo` executable.

You can print the path of your `aztec-nargo` executable by running:

```bash
which aztec-nargo
```

## Install Noir tooling

There are a number of tools to make writing Aztec.nr contracts in Noir more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Tutorials

See the [Token Contract tutorial](../tutorials/writing_token_contract.md) for more info on getting set up to write contracts.

## Learn more

<DocCardList />
