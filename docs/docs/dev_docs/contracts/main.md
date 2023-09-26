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

To write an Aztec.nr contract, you need to write Noir, which requires a build tool called Nargo:

<InstallNargoInstructions />

:::info
For those coming from vanilla Noir, the nargo version used for aztec.nr is tracked seaprately to nargo for vanilla noir, so be sure to use the nargo version shown above
:::

## Install Noir tooling

There are a number of tools to make writing Aztec.nr contracts more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Tutorials

See the [Token Contract tutorial](../tutorials/writing_token_contract.md) for more info on getting set up to write contracts.

## Learn more

<DocCardList />
