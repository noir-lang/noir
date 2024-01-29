---
title: Sandbox Features
---

The Aztec Sandbox is intended to provide developers with a lightweight and fast local node, running alongside a PXE.

Developers should be able to quickly spin up local, emulated instances of an Ethereum blockchain and an Aztec encrypted rollup, and start deploying private contracts and submitting private txs.

The sandbox allows developers to:

- Write and deploy Aztec contracts
- Leverage private and public state variables in contracts
- Write private and public functions in contracts
- Call private and public functions on other Aztec contracts (contract composability)
- Send messages between Aztec and Ethereum contracts
- Interact with the Aztec network using a familiar Typescript SDK ([aztec.js](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec.js))
- Start only a local PXE or Aztec node individually.
- Start a P2P bootstrap node for Aztec nodes to connect and discover each other.
