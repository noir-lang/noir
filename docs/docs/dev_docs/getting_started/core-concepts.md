---
title: Core Concepts
---

import Image from '@theme/IdealImage';

This page outlines Aztec concepts that are essential for developers to understand. Reading and understanding these concepts will help you massively when you start to dive deeper into smart contracts.

A little bit of time here can save a lot down the road.

# Aztec Overview

<Image img={require('/img/aztec_high_level_network_architecture.png')} />

To sum this up:
1. A user interacts with Aztec through Aztec.js (like web3js or ethersjs) or Aztec CLI
2. Private functions are executed in the PXE, which is client-side
3. They are rolled up and sent to the Public VM
4. Public functions are executed in the Public VM
5. The Public VM rolls up the private & public transaction rollups
6. These rollups are submitted to Ethereum

## Composability between private and public state

The PXE is unaware of the Public VM. And the Public VM is unaware of the PXE. They are completely separate execution environments. This means:

* The PXE and the Public VM cannot directly communicate with each other
* Private transactions in the PXE are executed first, followed by public transactions

You can call a public function from a private function by using `context.call_public_function`, like this:

#include_code call_public_function yarn-project/noir-contracts/contracts/card_game_contract/src/main.nr rust

You cannot call a private function from a public function, but you can use a slow updates tree to read historical public state and stage writes to public state from a private function. 

### Data types

Private state works with UTXOs, or what we call notes. To keep things private, everything is stored in an append-only UTXO tree, and a nullifier is created when notes are spent.

Public state works similarly to other chains like Ethereum, behaving more like a public ledger. 

Working with private state is like creating commitments and nullifiers to state, whereas working with public state is like directly updating state.

We have abstractions for working with private state so you don't have to worry about these commitments and nullifiers. However, it is important to understand that the types and libraries you use will be different when working with private state and public state.

For example, let's say you're trying to work with an integer. We have a library called `EasyPrivateUint` that acts like an integer but in the background is actually updating notes in private state. For the public side, we instead have something called `SafeU120`. You cannot use EasyPrivateUint in a public environment, and you cannot use SafeU120 in a private environment.

# Storage

Currently, when writing Aztec.nr smart contracts, you will need to define two things when initiating storage:

1. The storage struct, ie what you are storing and their types
2. A storage `impl` block with `init` function that will be called when you use the storage in a function

The storage struct looks like this:

#include_code storage_struct yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

The storage impl block looks like this:

#include_code storage_init yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

The `init` function must declare the storage struct with an instantiation defining how variables are accessed and manipulated. Each variable must be given a storage slot, which can be anything except 0.

The impl block is likely to be abstracted away at a later date.

Learn more about how to use storage [here](../contracts/syntax/storage/main.md).

# Portals

Aztec allows you to interact with Ethereum privately - ie no-one knows where the transaction is coming from, just that it is coming from somewhere on Aztec.

This is achieved through portals - these are smart contracts written in Solidity that are related to the Ethereum smart contract you want to interact with.

A portal can be associated with multiple Aztec contracts, but an Aztec contract can only be associated with one portal. 

Learn more about how to work with portals [here](../contracts/portals/main.md).

# Account Abstraction

Every account in Aztec is a smart contract. This allows implementing different schemes for transaction signing, nonce management, and fee payments.

You can write your own account contract to define the rules by which user transactions are authorized and paid for, as well as how user keys are managed.

Learn more about account contracts [here](../../concepts/foundation/accounts/main.md).

# Noir Language

Aztec smart contracts are written in a framework on top of Noir, the zero-knowledge domain-specific language developed specifically for Aztec. Its syntax is similar to Rust. Outside of Aztec, Noir is used for writing circuits that can be verified in Solidity.

A cursory understanding of Noir is sufficient for writing Aztec contracts. The [Noir docs](https://noir-lang.org) will be a helpful reference when you start writing more advanced contracts.

Now you're ready to dive into coding:

1. [Learn how to interact with a smart contract in Aztec.js](./aztecjs-getting-started.md)
2. [Write your first Aztec smart contract](./aztecnr-getting-started.md)