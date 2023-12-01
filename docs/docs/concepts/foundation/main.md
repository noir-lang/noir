---
title: Foundational Concepts
---

As a layer 2 rollup on Ethereum, the Aztec network includes components that look similar to other layer 2 networks, but since it handles private state it also includes many new components.

On this page we will introduce the high level network architecture for Aztec with an emphasis on the concepts that are core to understanding Aztec, including:

- [The state model](./state_model/main.md)
- [Accounts](./accounts/main.md)
- [Aztec Smart Contracts](./contracts.md)
- [Transactions](./transactions.md)
- [Communication between network components](./communication/main.md)

## High level network architecture

An overview of the Aztec network architecture will help contextualize the concepts introduced in this section.

<img src="/img/aztec_high_level_network_architecture.svg" alt="network architecture" />

### Aztec.js

A user of the Aztec network will interact with the network through Aztec.js. Aztec.js is a library that provides APIs for managing accounts and interacting with smart contracts (including account contracts) on the Aztec network. It communicates with the [Private eXecution Environment (PXE)](../../apis/pxe/interfaces/PXE) through a `PXE` implementation, allowing developers to easily register new accounts, deploy contracts, view functions, and send transactions.

### Private Execution Environment

The PXE provides a secure environment for the execution of sensitive operations, ensuring private information and decrypted data are not accessible to unauthorized applications. It hides the details of the [state model](./state_model/main.md) from end users, but the state model is important for Aztec developers to understand as it has implications for [private/public execution](./communication/public_private_calls/main.md) and [L1/L2 communication](./communication/cross_chain_calls.md). The PXE also includes the [ACIR Simulator](../advanced/acir_simulator.md) for private executions and the KeyStore for secure key management.

Procedurally, the PXE sends results of private function execution and requests for public function executions to the [sequencer](./nodes_clients/sequencer.md), which will update the state of the rollup.

### Sequencer

The sequencer aggregates transactions into a block, generates proofs of the state updates (or delegates proof generate to the prover network) and posts it to the rollup contract on Ethereum, along with any required public data for data availability.

## Further Reading

Here are links to pages with more information about the network components mentioned above:

- Aztec.js
  - [Dapp tutorial](../../dev_docs/tutorials/writing_dapp/main.md)
  - [API reference](../../apis/aztec-js)
- Private Execution Environment (PXE)
  - [Dapp tutorial](../../dev_docs/tutorials/writing_dapp/pxe_service.md)
  - [API reference](../../apis/pxe/index.md)
- [Private Kernel Circuit](../advanced/circuits/kernels/private_kernel.md)
- [Sequencer](./nodes_clients/sequencer.md)
- Prover Network (coming soon<sup>tm</sup>)
- [Rollup Circuit](../advanced/circuits/rollup_circuits/main.md) -- a component of the rollup contract
