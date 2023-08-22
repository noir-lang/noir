# Contracts

## What is a Noir Contract?

**Noir** is a domain specific language for creating and verifying proofs. It's design choices are influenced heavily by Rust.

We've extended the Noir language to understand the notion of an **'Aztec smart contract'**.

- A **smart contract** is just a collection of persistent state variables, and a collection of functions which may edit those state variables.
- An **Aztec smart contract** is a smart contract with **private** state variables and **private** functions.
- A **Noir Contract** is just an Aztec smart contract, written in Noir syntax.


:::info
Throughout these docs, we'll refer to "Vanilla Noir" as being the version of Noir without the Noir Contract syntax.
:::

# Getting started

Please consider using the [TUTORIAL-TEMPLATE](../../TUTORIAL_TEMPLATE.md) for standalone guides / tutorials.

## Installing Noir

To write a Noir Contract, you need to write Noir, and to write Noir, you need to [install Nargo](https://noir-lang.org/getting_started/nargo_installation).

## Installing Noir tooling

There are a number of tools to make writing Noir Contracts more pleasant. See [here](https://github.com/noir-lang/awesome-noir#get-coding).

## Quick start

Download an Aztec Box. (Josh / Ze to build :) ).

@Josh what's the best way to enable a 'one-command' creation of a noir project? (akin to https://noir-lang.org/getting_started/hello_world). I wonder if @aztec/cli should have this functionality? 

@Josh I wonder if @aztec/cli would be a good place to hold 'Aztec Boxes'?
- `aztec-cli unbox`
- `aztec-cli unbox private_token`.

Or, if you don't want to do that, here's more detail on doing it all yourself:

:::danger TODO
TODO
:::


## Example Noir Contract

In keeping with the origins of blockchain, here's an example of a simple token contract. But here, the balances are private.

#include_code easy_private_token_storage /yarn-project/noir-contracts/src/contracts/easy_private_token_contract/src/storage.nr rust

#include_code easy_private_token_contract /yarn-project/noir-contracts/src/contracts/easy_private_token_contract/src/main.nr rust

:::info Disclaimer
Please note that any example contract set out herein is provided solely for informational purposes only and does not constitute any inducement to use or deploy. Any implementation of any such contract with an interface or any other infrastructure should be used in accordance with applicable laws and regulations.
:::

## Next steps
You can read more about writing contracts [here](./syntax.md), and then move to compiling it which is detailed [here](./compiling.md).