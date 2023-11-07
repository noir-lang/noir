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

## Install Noir

To write an Aztec.nr contract, you need to write Noir, [aztec-cli](../cli/cli-commands) comes with a built-in compiler for Noir contracts.

:::info
For those coming from vanilla Noir, the version used for aztec.nr is tracked separately to nargo for vanilla noir, so be sure to use the nargo version shown above
:::

## Install `nargo` (recommended)

The CLI comes with the Noir compiler, so installing `nargo` is not required, however it is recommended as it provides a better developer experience for writing contracts. You will need nargo installed to take advantage of the [Noir Language Server](https://noir-lang.org/nargo/language_server), which provides syntax highlighting and formatting for your Aztec contracts.

You will also need `nargo` if you want to run unit tests in Noir.

You can install `nargo` with the following commands:

<InstallNargoInstructions />

## Install Noir tooling

There are a number of tools to make writing Aztec.nr contracts more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Tutorials

See the [Token Contract tutorial](../tutorials/writing_token_contract.md) for more info on getting set up to write contracts.

## Learn more

<DocCardList />
