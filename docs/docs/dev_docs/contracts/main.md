# Aztec.nr

## What is Aztec.nr?

**Aztec.nr** is a library for writing Aztec smart contracts.

## Nomenclature

[**Noir**](https://noir-lang.org/) is a domain specific language for creating and verifying proofs. It's design choices are influenced heavily by Rust.

A **smart contract** is just a collection of persistent state variables, and a collection of functions which may edit those state variables.

An **Aztec smart contract** is a smart contract with **private** state variables and **private** functions.

**Aztec.nr** is a library for writing Aztec smart contracts, written in Noir.

# Getting started

## Install Noir

To write a Noir Contract, you need to write Noir, and to write Noir, you need to [install Nargo](https://noir-lang.org/getting_started/nargo_installation).

## Install Noir tooling

There are a number of tools to make writing Noir Contracts more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Quick start

:::danger TODO
Starter kit
:::

## Example Noir Contract

In keeping with the origins of blockchain, here's an example of a simple private token contract. Everyone's balances are private.

#include_code easy_private_token_contract /yarn-project/noir-contracts/src/contracts/easy_private_token_contract/src/main.nr rust

:::info Disclaimer
Please note that any example contract set out herein is provided solely for informational purposes only and does not constitute any inducement to use or deploy. Any implementation of any such contract with an interface or any other infrastructure should be used in accordance with applicable laws and regulations.
:::
