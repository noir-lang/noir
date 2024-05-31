---
title: Core Components
sidebar_position: 1
---

## Private Smart Contracts

A smart contract on Aztec is a collection of functions, written as ZK-SNARK circuits. These circuits can have different modes of execution:

1. Private Functions -- can read and write private state, read historical public state, consume or send messages to / from Ethereum, and read Ethereum state. They can call other private functions in the same contract, or other contracts, and can call public functions.
2. Public Functions -- can read and write public state, write private state, consume or send messages to / from Ethereum and read Ethereum state. They can call other public functions on the same or other contracts.
3. Portal Contracts -- these are contracts on Ethereum that can receive messages from Aztec or send messages to Aztec from Ethereum contracts.

Using these different modes of execution, developers can build applications with user privacy, data privacy and code privacy.

- User privacy - transactions may not reveal information about the sender or the recipient.
- Data privacy - transactions may not reveal information about the payload of the transaction, e.g., the asset or value being transacted.
- Code privacy - transactions may not reveal the program logic.

## High level network architecture

An overview of the Aztec network architecture will help contextualize the concepts introduced in this section.

<img src="/img/how-does-aztec-work.webp" alt="network architecture" />

### Aztec.js

A user of the Aztec network will interact with the network through Aztec.js. Aztec.js is a library that provides APIs for managing accounts and interacting with smart contracts (including account contracts) on the Aztec network. It communicates with the [Private eXecution Environment (PXE)](concepts/pxe/index.md) through a `PXE` implementation, allowing developers to easily register new accounts, deploy contracts, view functions, and send transactions.

### Private Execution Environment

The PXE provides a secure environment for the execution of sensitive operations, ensuring private information and decrypted data are not accessible to unauthorized applications. It hides the details of the [state model](concepts/state_model/index.md) from end users, but the state model is important for Aztec developers to understand as it has implications for [private/public execution](concepts/smart_contracts/communication/public_private_calls.md) and [L1/L2 communication](../protocol-specs/l1-smart-contracts/index.md). The PXE also includes the [ACIR Simulator](concepts/pxe/acir_simulator.md) for private executions and the KeyStore for secure key management.

Procedurally, the PXE sends results of private function execution and requests for public function executions to the [sequencer](concepts/nodes_clients/sequencer/index.md), which will update the state of the rollup.

### Sequencer

The sequencer aggregates transactions into a block, generates proofs of the state updates (or delegates proof generate to the prover network) and posts it to the rollup contract on Ethereum, along with any required public data for data availability.

## Further Reading

- [The state model](concepts/state_model/index.md)
- [Accounts](concepts/accounts/index.md)
- [Aztec Smart Contracts](concepts/smart_contracts/index.md)
- [Transactions](concepts/transactions.md)
- [Communication between network components](concepts/smart_contracts/communication/index.md)
