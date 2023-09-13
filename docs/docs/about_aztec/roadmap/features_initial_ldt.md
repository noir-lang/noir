---
title: Initial Sandbox Features
---

The Aztec Sandbox is intended to provide developers with a lightweight & fast node, with features similar to Ethereum's Ganache or Anvil 'local node' packages.

Devs should be able to quickly spin up local, emulated instances of an Ethereum blockchain and an Aztec encrypted rollup, and start deploying private contracts and submitting private txs.

Here's a summary of the features we intend to support with the first release of the Aztec Sandbox.

## Aztec.nr Contracts

- Noir `contract` scopes.
  - Declare a `contract`, containing a collection of state variables and functions.
- private state variables:
  - `read`, `write`, and `delete` private state variables within private functions.
- public (non-private) state variables:
  - Manipulate 'public' state in a familiar way to Ethereum state.
- private functions
  - May read and modify private state.
- public functions
  - May read and modify public state.
- `constructor` functions, for initialising contract state.
- `import` other Aztec.nr contracts, so their functions may be called.
- Nested function calls, for contract composability
  - private functions can call private functions of other contracts, and receive return values.
  - private functions can call public functions any contract.
  - public functions can call private functions of any contract.
  - public functions can call public functions of other contracts, and receive return values.
  - private functions can be called recursively.
  - public functions can be called recursively.
- Send messages from Aztec.nr contracts to Ethereum L1, for consumption by L1 smart contracts.
  - Useful, for example, if writing an app to withdraw funds from L2 to L1.
- Consume messages which have been sent by:
  - L1 functions.
    - Useful, for example, if writing an app to deposit funds from L1 to L2.
  - public L2 functions.
- Emit `event` data from a Aztec.nr Contract.
  - Allows applications to subscribe to events which have been emitted by a Aztec.nr contract's functions, for example.
- Write `unconstrained` functions.
  - These allow developers to write `pure` and `view` functions, which can perform calculations and retrieve state. E.g. for fetching contract-specific information, which may then be consumed by a dapp, without having to generate a zero-knowledge proof or interact with the 'network'.

## `aztec.js`

A typescript wrapper for making RPC calls to an Aztec LDT node.

- Similar in purpose to `web3.js`/`ethers.js`/`viem`, but for interacting with Aztec Network nodes. The RPC interface for an Aztec node is necessarily different from that of an Ethereum node, because it deals with encrypted transactions and state variables.
- A library for public/private key management.
- Construct `Contract` instances from a Aztec.nr contract's JSON ABI.
- Deploy new contracts to the Aztec LDT.
- Construct tx requests, passing arguments to a function of a contract.
- Sign tx requests.
- Send txs to the LDT node, for simulating.
- Send txs to the LDT node, to be sent to the LDT network.
- Call `unconstrained` functions of a Aztec.nr contract, to perform `pure` calculations or retrieve state.

## Aztec Local Developer Testnet Node

A bundle of packages which emulate the actions of all eventual Aztec network participants. The goal is for developer experience to be akin to Ganache / Anvil.

- Aztec RPC Client
  - Simulate and/or execute private functions locally.
- Aztec Public Node
  - Broadcasts a user's txs to the tx pool.
  - Simulate public functions locally.
- Tx Pool
  - An in-memory emulation of a tx pool. By default, a user's txs will be rolled-up into an L2 block immediately.
- Sequencer Node
  - Reads the tx pool and bundles pending txs into a rollup block immediately.
  - Orders txs.
  - Executes public functions.
  - Passes messages between L1 and L2.
- L1 Rollup smart contract
  - Verifies the rollup's snark.
  - Reconciles calldata with snark public inputs.
  - Updates the rollup's state hash.
- L1 data archiver
  - Gobbles up and stores all calldata, events, and state changes from L1
- World state DB
  - Reconstructs the Aztec Network's various trees.
  - Allows tree state to be queried.

## Participate

Keep up with the latest discussion and join the conversation in the [Aztec forum](https://discourse.aztec.network).


import Disclaimer from "../../misc/common/\_disclaimer.mdx";
<Disclaimer/>