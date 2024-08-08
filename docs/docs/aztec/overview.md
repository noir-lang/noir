---
title: What is Aztec?
sidebar_position: 0
id: overview
tags: [protocol]
---

import Image from "@theme/IdealImage";

This page outlines Aztec's fundamental technical concepts.

## Aztec Overview

<Image img={require("/img/how-does-aztec-work.webp")} />

1. A user interacts with Aztec through Aztec.js (like web3js or ethersjs)
2. Private functions are executed in the PXE, which is client-side
3. They are rolled up and sent to the Public VM (running on an Aztec node)
4. Public functions are executed in the Public VM
5. The Public VM rolls up the private & public transaction rollups
6. These rollups are submitted to Ethereum

The PXE is unaware of the Public VM. And the Public VM is unaware of the PXE. They are completely separate execution environments. This means:

- The PXE and the Public VM cannot directly communicate with each other
- Private transactions in the PXE are executed first, followed by public transactions

### Private and public state

Private state works with UTXOs, or what we call notes. To keep things private, everything is stored in an [append-only UTXO tree](./concepts/storage/trees/index.md), and a nullifier is created when notes are invalidated. Nullifiers are then stored in their own [nullifier tree](./concepts/storage/trees/index.md).

Public state works similarly to other chains like Ethereum, behaving like a public ledger. Public data is stored in a [public data tree](./concepts/storage/trees/index.md#public-state-tree).

Aztec [smart contract](./smart_contracts_overview.md) developers should keep in mind that different types are used when manipulating private or public state. Working with private state is creating commitments and nullifiers to state, whereas working with public state is directly updating state.

## Accounts

Every account in Aztec is a smart contract (account abstraction). This allows implementing different schemes for transaction signing, nonce management, and fee payments.

Developers can write their own account contract to define the rules by which user transactions are authorized and paid for, as well as how user keys are managed.

Learn more about account contracts [here](./concepts/accounts/index.md).

## Smart contracts

Developers can write [smart contracts](./smart_contracts_overview.md) that manipulate both public and private state. They are written in a framework on top of Noir, the zero-knowledge domain-specific language developed specifically for Aztec. Outside of Aztec, Noir is used for writing circuits that can be verified on EVM chains.

Noir has its own doc site that you can find [here](https://noir-lang.org).

## Communication with Ethereum

Aztec allows private communications with Ethereum - ie no-one knows where the transaction is coming from, just that it is coming from somewhere on Aztec.

This is achieved through portals - these are smart contracts deployed on an EVM that are related to the Ethereum smart contract you want to interact with.

Learn more about portals [here](../protocol-specs/l1-smart-contracts/index.md).

## Circuits

Aztec operates on three types of circuits:

- [Private kernel circuits](../aztec/concepts/circuits/kernels/private_kernel.md), which are executed by the user on their own device and prove correct execution of a function
- [Public kernel circuits](../aztec/concepts/circuits/kernels/public_kernel.md), which are executed by the [sequencer](./network/sequencer/index.md) and ensure the stack trace of transactions adheres to function execution rules
- [Rollup circuits](../aztec/concepts/circuits/index.md), which bundle all of the Aztec transactions into a proof that can be efficiently verified on Ethereum

## What's next?

### Dive deeper into how Aztec works

Explore the Concepts for a deeper understanding into the components that make up Aztec:

<div className="card-container">

  <Card shadow='tl' link='/aztec/concepts/accounts'>
    <CardHeader>
      <h3>Accounts</h3>
    </CardHeader>
    <CardBody>
      Learn about Aztec's native account abstraction - every account in Aztec is a smart contract which defines the rules for whether a transaction is or is not valid
    </CardBody>
  </Card>

  <Card shadow='tl' link='/aztec/concepts/circuits'>
    <CardHeader>
      <h3>Circuits</h3>
    </CardHeader>
    <CardBody>
      Central to Aztec's operations are circuits in the core protocol and the developer-written Aztec.nr contracts
    </CardBody>
  </Card>

  <Card shadow='tl' link='/aztec/concepts/pxe'>
    <CardHeader>
      <h3>PXE (pronounced 'pixie')</h3>
    </CardHeader>
    <CardBody>
      The Private Execution Environment (or PXE) is a client-side library for the execution of private operations
    </CardBody>
  </Card>

   <Card shadow='tl' link='/aztec/concepts/state_model'>
    <CardHeader>
      <h3>State model</h3>
    </CardHeader>
    <CardBody>
      Aztec has a hybrid public/private state model
    </CardBody>
  </Card>

  <Card shadow='tl' link='/aztec/concepts/storage'>
    <CardHeader>
      <h3>Storage</h3>
    </CardHeader>
    <CardBody>
     In Aztec, private data and public data are stored in two trees: a public data tree and a note hashes tree
    </CardBody>
  </Card>

  <Card shadow='tl' link='/aztec/concepts/wallets'>
    <CardHeader>
      <h3>Wallets</h3>
    </CardHeader>
    <CardBody>
     Wallets expose to dapps an interface that allows them to act on behalf of the user, such as querying private state or sending transactions
    </CardBody>
  </Card>

</div>

### Start coding

<div>
 <Card shadow='tl' link='/guides/developer_guides/getting_started/quickstart'>
    <CardHeader>
      <h3>Developer quickstart</h3>
    </CardHeader>
    <CardBody>
      Follow the getting started guide to start developing with the Aztec Sandbox
    </CardBody>
  </Card>
</div>