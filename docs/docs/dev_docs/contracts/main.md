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

To write an Aztec.nr contract, you need to write Noir, `aztec-nargo` comes with a built-in compiler for Aztec contracts written in Noir. See install instructions [here](../cli/sandbox-reference.md).

:::info
For those coming from vanilla Noir, the version used for aztec.nr is tracked separately to nargo for vanilla Noir. Be sure to use `aztec-nargo` to compile your contracts.
:::

## Install `nargo` (recommended)

`aztec-nargo` comes with the Noir compiler, so installing `nargo` is not required, however it is recommended as it provides a better developer experience for writing contracts. You will need nargo installed to take advantage of the [Noir Language Server](https://noir-lang.org/nargo/language_server), which provides syntax highlighting and formatting for your Aztec contracts.

You can install `nargo` with the following commands:

<InstallNargoInstructions />

## Install Noir tooling

There are a number of tools to make writing Aztec.nr contracts in Noir more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Tutorials

See the [Token Contract tutorial](../tutorials/writing_token_contract.md) for more info on getting set up to write contracts.

## Learn more

<DocCardList />
